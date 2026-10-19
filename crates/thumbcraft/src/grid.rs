//! Implementação de `Render` com o GridCraft patchado (xlsx, xls, ods, csv, tsv).
//!
//! Tudo passa pelos comandos públicos do engine (`file.open`, `pageLayout.pageSetup`,
//! `file.printPreview`, `file.exportPng`), como o gc-min de viabilidade. A página impressa é
//! A4 na orientação da própria planilha, sem zoom e sem ajuste à página.

use std::path::Path;

use gridcraft_engine::Session;
use serde_json::{Value, json};
use thumbcraft_core::{Error, Raster, Render, Res};

/// Planilha aberta, com a lista de páginas de impressão.
pub struct Grid {
    sess: Session,
    /// Tamanho de cada página de impressão em pontos.
    pages: Vec<(f64, f64)>,
}

fn run(s: &mut Session, id: &str, p: Value) -> Result<Value, String> {
    s.execute_headless(id, p).map_err(|e| e.to_string())
}

impl Grid {
    pub fn open(path: &Path) -> Res<Grid> {
        let bad = |e: String| Error::Invalid(format!("não foi possível abrir como planilha: {e}"));
        let mut s = Session::new();
        run(&mut s, "file.open", json!({"path": path.to_string_lossy()})).map_err(bad)?;
        // Primeira planilha visível.
        let first = {
            let d = s.doc().map_err(|e| bad(e.to_string()))?;
            (0..d.wb.sheets.len()).find(|i| d.wb.sheet(*i).is_some_and(|sh| format!("{:?}", sh.visibility) == "Visible")).unwrap_or(0)
        };
        run(&mut s, "sheet.activate", json!({"sheet": first})).map_err(bad)?;
        run(&mut s, "pageLayout.pageSetup", json!({"paper": "A4", "fitToPage": false})).map_err(bad)?;
        let prev = run(&mut s, "file.printPreview", json!({})).map_err(bad)?;
        let pages = prev
            .get("pageList")
            .and_then(Value::as_array)
            .map(|l| l.iter().map(|p| (p["width"].as_f64().unwrap_or(0.0), p["height"].as_f64().unwrap_or(0.0))).collect())
            .unwrap_or_default();
        Ok(Grid { sess: s, pages })
    }

    /// PNG da página `page` (a partir de 0) na escala dada, pelos bytes que o comando devolve.
    fn png(&mut self, page: usize, scale: f64) -> Res<Vec<u8>> {
        let r = run(&mut self.sess, "file.exportPng", json!({"scale": scale, "page": page + 1}))
            .map_err(|e| Error::Invalid(format!("falha ao renderizar a página {}: {e}", page + 1)))?;
        let b64 = r.get("base64").and_then(Value::as_str).ok_or_else(|| Error::Internal("o GridCraft não devolveu a imagem".into()))?;
        gridcraft_engine::io::base64_decode(b64).ok_or_else(|| Error::Internal("imagem do GridCraft ilegível (base64)".into()))
    }
}

impl Render for Grid {
    fn unit(&self) -> &'static str {
        "página de impressão"
    }

    fn pages(&self) -> usize {
        self.pages.len()
    }

    fn size(&mut self, page: usize) -> Res<(f64, f64)> {
        self.pages.get(page).copied().ok_or_else(|| Error::Invalid(format!("a página {} não existe", page + 1)))
    }

    fn render(&mut self, page: usize, scale: f64) -> Res<Raster> {
        let bytes = self.png(page, scale)?;
        decode_png(&bytes)
    }

    fn native_png(&mut self, page: usize, scale: f64) -> Res<Vec<u8>> {
        self.png(page, scale)
    }
}

/// Decodifica um PNG de 8 bits (RGB ou RGBA) para RGBA pré-multiplicado.
fn decode_png(bytes: &[u8]) -> Res<Raster> {
    let bad = |e: &dyn std::fmt::Display| Error::Internal(format!("PNG do GridCraft ilegível: {e}"));
    let mut rd = png::Decoder::new(std::io::Cursor::new(bytes)).read_info().map_err(|e| bad(&e))?;
    let mut buf = vec![0u8; rd.output_buffer_size().ok_or_else(|| Error::Internal("imagem grande demais".into()))?];
    let info = rd.next_frame(&mut buf).map_err(|e| bad(&e))?;
    let n = info.width as usize * info.height as usize;
    let px = match (info.color_type, info.bit_depth) {
        (png::ColorType::Rgba, png::BitDepth::Eight) => {
            let mut v = buf;
            v.truncate(n * 4);
            for p in v.chunks_exact_mut(4) {
                let a = p[3] as u32;
                if a != 255 {
                    for c in &mut p[..3] {
                        *c = ((*c as u32 * a + 127) / 255) as u8;
                    }
                }
            }
            v
        }
        (png::ColorType::Rgb, png::BitDepth::Eight) => {
            let mut v = Vec::with_capacity(n * 4);
            for p in buf[..n * 3].chunks_exact(3) {
                v.extend_from_slice(&[p[0], p[1], p[2], 255]);
            }
            v
        }
        other => return Err(Error::Internal(format!("formato de PNG inesperado: {other:?}"))),
    };
    Ok(Raster { w: info.width, h: info.height, px })
}
