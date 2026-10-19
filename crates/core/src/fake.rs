//! Implementação falsa de `Render`, para testar o pós-processamento sem motores.

use crate::error::{Error, Res};
use crate::raster::Raster;
use crate::render::Render;

/// O que a página falsa desenha sobre o fundo.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Pattern {
    /// Só o fundo.
    Blank,
    /// Retângulo preto, em pontos: x0, y0, x1, y1.
    Box(f64, f64, f64, f64),
    /// Quatro quadrantes coloridos: vermelho, verde (cima), azul, amarelo (baixo).
    Quadrants,
}

/// Documento falso: todas as páginas têm o mesmo tamanho e desenho.
pub struct Fake {
    pub pages: usize,
    pub size: (f64, f64),
    pub pattern: Pattern,
    /// Fundo transparente em vez de branco.
    pub transparent: bool,
    /// Falha ao renderizar (para testar propagação de erro).
    pub fail: bool,
    /// Chamadas a `render`: (página, escala).
    pub calls: Vec<(usize, f64)>,
}

impl Fake {
    pub fn new(pages: usize, w: f64, h: f64, pattern: Pattern) -> Fake {
        Fake { pages, size: (w, h), pattern, transparent: false, fail: false, calls: Vec::new() }
    }
}

impl Render for Fake {
    fn unit(&self) -> &'static str {
        "página"
    }

    fn pages(&self) -> usize {
        self.pages
    }

    fn size(&mut self, page: usize) -> Res<(f64, f64)> {
        if page >= self.pages {
            return Err(Error::Invalid("página fora do documento".into()));
        }
        Ok(self.size)
    }

    fn render(&mut self, page: usize, scale: f64) -> Res<Raster> {
        self.calls.push((page, scale));
        if self.fail {
            return Err(Error::Invalid("falha simulada".into()));
        }
        let w = (self.size.0 * scale).ceil().max(1.0) as u32;
        let h = (self.size.1 * scale).ceil().max(1.0) as u32;
        let bg = if self.transparent { [0, 0, 0, 0] } else { [255, 255, 255, 255] };
        let mut r = Raster::filled(w, h, bg);
        match self.pattern {
            Pattern::Blank => {}
            Pattern::Box(x0, y0, x1, y1) => {
                let (px0, py0) = ((x0 * scale).round() as u32, (y0 * scale).round() as u32);
                let (px1, py1) = (((x1 * scale).round() as u32).min(w), ((y1 * scale).round() as u32).min(h));
                for y in py0..py1 {
                    for x in px0..px1 {
                        r.set(x, y, [0, 0, 0, 255]);
                    }
                }
            }
            Pattern::Quadrants => {
                for y in 0..h {
                    for x in 0..w {
                        let c = match (x < w / 2, y < h / 2) {
                            (true, true) => [255, 0, 0, 255],
                            (false, true) => [0, 255, 0, 255],
                            (true, false) => [0, 0, 255, 255],
                            (false, false) => [255, 255, 0, 255],
                        };
                        r.set(x, y, c);
                    }
                }
            }
        }
        Ok(r)
    }
}
