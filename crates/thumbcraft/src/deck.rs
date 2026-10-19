//! Implementação de `Render` com o DeckCraft (pptx, potx, ppsx).
//!
//! Segue o dc-min, com uma diferença deliberada: o DeckCraft descarta slides que não consegue
//! ler (XML quebrado) e renderiza o seguinte no lugar. Aqui, pedir um slide danificado é ERRO
//! (código 4), e os slides depois dele são traduzidos para o índice certo.

use std::io::{Cursor, Read};
use std::path::Path;

use deckcraft_fonts::FontDb;
use deckcraft_render::RenderOpts;
use thumbcraft_core::{Error, Raster, Render, Res};

/// Apresentação aberta.
pub struct Deck {
    pres: deckcraft_model_alias::Presentation,
    /// Posições (a partir de 0, na ordem original do arquivo) dos slides danificados.
    lost: Vec<usize>,
}

// O tipo vem do `deckcraft-pptx` sem exigir o crate de modelo como dependência direta.
mod deckcraft_model_alias {
    pub use deckcraft_pptx::Presentation;
}

impl Deck {
    pub fn open(path: &Path) -> Res<Deck> {
        let bytes = std::fs::read(path).map_err(|e| Error::Invalid(format!("não foi possível ler o arquivo: {e}")))?;
        let (mut pres, lost_paths) = if deckcraft_pptx::sniff(&bytes) {
            let (mut p, lost) = deckcraft_pptx::import_checked(&bytes).map_err(|e| Error::Invalid(format!("não foi possível abrir como apresentação: {e}")))?;
            deckcraft_format::repair(&mut p);
            (p, lost)
        } else if deckcraft_format::sniff(&bytes) {
            let (p, _) = deckcraft_format::load_checked(&bytes).map_err(|e| Error::Invalid(format!("não foi possível abrir como apresentação: {e}")))?;
            (p, Vec::new())
        } else {
            return Err(Error::Invalid("o arquivo não é uma apresentação que o DeckCraft consiga ler (pptx, potx ou ppsx)".into()));
        };
        let db = FontDb::global();
        for font in &pres.embedded_fonts {
            for id in font.faces().into_iter().flatten() {
                if let Some(item) = pres.media(id) {
                    if let Some(s) = deckcraft_fonts::embedded_sfnt(&item.data) {
                        db.add_font(s.to_vec());
                    }
                }
            }
        }
        let _ = &mut pres;
        let lost = if lost_paths.is_empty() { Vec::new() } else { positions(&bytes, &lost_paths).ok_or_else(|| {
            Error::Invalid(format!(
                "a apresentação tem {} slide(s) danificado(s) e não foi possível saber quais são; renderizar aqui mostraria o slide errado",
                lost_paths.len()
            ))
        })? };
        Ok(Deck { pres, lost })
    }

    /// Traduz o índice do arquivo original para o da lista que o motor guardou.
    fn resolve(&self, page: usize) -> Res<usize> {
        if self.lost.contains(&page) {
            return Err(Error::Invalid(format!("o slide {} está danificado no arquivo (XML ilegível); não dá para gerar a miniatura dele", page + 1)));
        }
        Ok(page - self.lost.iter().filter(|l| **l < page).count())
    }

    fn scale_for(&self, scale: f64) -> f64 {
        let size = self.pres.slide_size;
        let longest = size.width.max(size.height).max(1.0);
        let s = if scale.is_finite() { scale.clamp(0.01, 16.0) } else { 1.0 };
        s.min(f64::from(deckcraft_render::MAX_SIDE) / longest)
    }
}

impl Render for Deck {
    fn unit(&self) -> &'static str {
        "slide"
    }

    fn pages(&self) -> usize {
        self.pres.slides.len() + self.lost.len()
    }

    fn size(&mut self, page: usize) -> Res<(f64, f64)> {
        self.resolve(page)?;
        Ok((self.pres.slide_size.width, self.pres.slide_size.height))
    }

    fn render(&mut self, page: usize, scale: f64) -> Res<Raster> {
        let idx = self.resolve(page)?;
        let opts = RenderOpts { scale: self.scale_for(scale), ..Default::default() };
        let img = deckcraft_render::render_slide(&self.pres, idx, &opts);
        Ok(Raster { w: img.width, h: img.height, px: img.pixels })
    }

    fn native_png(&mut self, page: usize, scale: f64) -> Res<Vec<u8>> {
        let idx = self.resolve(page)?;
        let scale = self.scale_for(scale);
        let opts = RenderOpts { scale, ..Default::default() };
        let img = deckcraft_render::render_slide(&self.pres, idx, &opts);
        Ok(img.to_png_at(deckcraft_render::dpi_for_scale(scale)))
    }
}

/// Valor do atributo `name` dentro de um trecho de tag (`<... name="valor" ...>`).
fn attr<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let key = format!("{name}=\"");
    let mut from = 0;
    while let Some(i) = tag[from..].find(&key) {
        let at = from + i;
        // O nome precisa começar o atributo (depois de espaço), não ser sufixo de outro.
        if at == 0 || tag.as_bytes()[at - 1].is_ascii_whitespace() {
            let rest = &tag[at + key.len()..];
            return rest.split('"').next();
        }
        from = at + key.len();
    }
    None
}

/// Lê uma entrada do ZIP como texto (até 8 MB).
fn read_text(z: &mut zip::ZipArchive<Cursor<&[u8]>>, name: &str) -> Option<String> {
    let f = z.by_name(name).ok()?;
    let mut s = String::new();
    f.take(8 << 20).read_to_string(&mut s).ok()?;
    Some(s)
}

/// Posições originais (na ordem de `sldIdLst`) dos slides cujos caminhos estão em `lost`.
/// `None` se a ordem não pôde ser reconstruída ou algum caminho não foi encontrado.
fn positions(bytes: &[u8], lost: &[String]) -> Option<Vec<usize>> {
    let mut z = zip::ZipArchive::new(Cursor::new(bytes)).ok()?;
    let pres = read_text(&mut z, "ppt/presentation.xml")?;
    let rels = read_text(&mut z, "ppt/_rels/presentation.xml.rels")?;
    // r:id de cada <p:sldId>, na ordem.
    let mut ids = Vec::new();
    for part in pres.split('<').skip(1) {
        let tag = part.split('>').next().unwrap_or("");
        let name = tag.split(|c: char| c.is_whitespace() || c == '/').next().unwrap_or("");
        if name == "sldId" || name.ends_with(":sldId") {
            // O atributo do relacionamento tem prefixo (r:id); o numérico é só `id`.
            let rid = tag.split_whitespace().find_map(|a| a.strip_prefix("r:id=\"").or_else(|| a.split_once(":id=\"").map(|(_, v)| v)))?;
            ids.push(rid.trim_end_matches(['"', '/']).to_string());
        }
    }
    let norm = |t: &str| t.trim_start_matches('/').to_string();
    let target_of = |rid: &str| -> Option<String> {
        rels.split('<').skip(1).find_map(|part| {
            let tag = part.split('>').next().unwrap_or("");
            if !tag.starts_with("Relationship ") || attr(tag, "Id")? != rid {
                return None;
            }
            let t = attr(tag, "Target")?;
            Some(if t.starts_with('/') { norm(t) } else { format!("ppt/{}", t.trim_start_matches("./")) })
        })
    };
    let order: Vec<Option<String>> = ids.iter().map(|r| target_of(r)).collect();
    let mut pos = Vec::new();
    for l in lost {
        let l = norm(l);
        let i = order.iter().position(|t| t.as_deref().is_some_and(|t| t == l || t.ends_with(&l) || l.ends_with(t)))?;
        pos.push(i);
    }
    pos.sort_unstable();
    pos.dedup();
    Some(pos)
}
