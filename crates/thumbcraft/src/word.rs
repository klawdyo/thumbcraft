//! Implementação de `Render` com o WordCraft (docx, doc, odt, rtf, html, md, tex, txt).

use std::path::Path;
use std::sync::Arc;

use thumbcraft_core::{Error, Raster, Render, Res};
use wordcraft_engine::layout::DocLayout;
use wordcraft_engine::render::{RenderOptions, render_page};
use wordcraft_engine::{Region, Session};

/// Documento Word aberto, com o layout de impressão já calculado.
pub struct Word {
    sess: Session,
    layout: Arc<DocLayout>,
}

impl Word {
    pub fn open(path: &Path) -> Res<Word> {
        let region = Region::from_locale_vars(|n| std::env::var(n).ok(), || None);
        let (doc, _) = wordcraft_engine::io::open_path_in(path, None, &region).map_err(|e| Error::Invalid(format!("não foi possível abrir como documento Word: {e}")))?;
        let sess = Session::new(doc);
        // O mesmo layout que `file.exportPng` usa (sem marcas de revisão).
        let layout = sess.export_layout_with(false);
        Ok(Word { sess, layout })
    }

    /// Passo comum de `render` e `native_png`: renderiza a página como o `file.exportPng`.
    fn draw(&self, page: usize, scale: f32) -> Res<wordcraft_engine::render::Rendered> {
        let pg = self.layout.pages.get(page).ok_or_else(|| Error::Invalid(format!("a página {} não existe", page + 1)))?;
        // Uma página de imagem é a página impressa: cabeçalho e rodapé com força total.
        let mut opts = RenderOptions::default();
        opts.display.dim_header = false;
        Ok(render_page(&self.sess.doc, pg, scale, &opts))
    }
}

impl Render for Word {
    fn unit(&self) -> &'static str {
        "página"
    }

    fn pages(&self) -> usize {
        self.layout.pages.len()
    }

    fn size(&mut self, page: usize) -> Res<(f64, f64)> {
        let pg = self.layout.pages.get(page).ok_or_else(|| Error::Invalid(format!("a página {} não existe", page + 1)))?;
        Ok((f64::from(pg.w), f64::from(pg.h)))
    }

    fn render(&mut self, page: usize, scale: f64) -> Res<Raster> {
        let img = self.draw(page, scale as f32)?;
        Ok(Raster { w: img.width, h: img.height, px: img.pixels })
    }

    fn native_png(&mut self, page: usize, scale: f64) -> Res<Vec<u8>> {
        // Mesma faixa que o comando do motor aplica à escala.
        let scale = (scale as f32).clamp(0.1, 8.0);
        let png = self.draw(page, scale)?.to_png();
        if png.is_empty() {
            return Err(Error::Invalid("o WordCraft não conseguiu gerar o PNG da página".into()));
        }
        Ok(png)
    }
}
