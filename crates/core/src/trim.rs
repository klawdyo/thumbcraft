//! Remoção automática de margens uniformes brancas ou transparentes.

use crate::raster::{Raster, Region};

/// Tolerância: um pixel conta como vazio se, composto sobre branco, todos os canais passam disso.
const BLANK: u16 = 250;

/// O pixel pré-multiplicado `p` é branco ou transparente (composto sobre branco)?
fn blank(p: &[u8]) -> bool {
    let back = 255 - p[3] as u16;
    p[0] as u16 + back >= BLANK && p[1] as u16 + back >= BLANK && p[2] as u16 + back >= BLANK
}

/// Menor retângulo, dentro de `r`, que contém todo o conteúdo não vazio. `None` se tudo é vazio.
pub fn content(img: &Raster, r: Region) -> Option<Region> {
    let (mut x0, mut y0, mut x1, mut y1) = (u32::MAX, u32::MAX, 0u32, 0u32);
    for y in r.y..r.y + r.h {
        let row = (y as usize * img.w as usize + r.x as usize) * 4;
        let slice = &img.px[row..row + r.w as usize * 4];
        // Primeiro e último pixel não vazio da linha.
        let first = slice.chunks_exact(4).position(|p| !blank(p));
        let Some(first) = first else { continue };
        let last = slice.chunks_exact(4).rposition(|p| !blank(p)).unwrap_or(first);
        x0 = x0.min(r.x + first as u32);
        x1 = x1.max(r.x + last as u32 + 1);
        y0 = y0.min(y);
        y1 = y1.max(y + 1);
    }
    if x1 <= x0 || y1 <= y0 {
        return None;
    }
    Some(Region { x: x0, y: y0, w: x1 - x0, h: y1 - y0 })
}

/// Amplia `c` em `pad` px por lado, sem sair de `bounds`.
pub fn pad(c: Region, pad: u32, bounds: Region) -> Region {
    let x0 = c.x.saturating_sub(pad).max(bounds.x);
    let y0 = c.y.saturating_sub(pad).max(bounds.y);
    let x1 = (c.x + c.w).saturating_add(pad).min(bounds.x + bounds.w);
    let y1 = (c.y + c.h).saturating_add(pad).min(bounds.y + bounds.h);
    Region { x: x0, y: y0, w: x1 - x0, h: y1 - y0 }
}

/// Aplica o trim em `r`: conteúdo mais `pad_px`; se tudo é vazio, devolve `r` sem mudança.
pub fn trim(img: &Raster, r: Region, pad_px: u32) -> Region {
    match content(img, r) {
        Some(c) => pad(c, pad_px, r),
        None => r,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn white(w: u32, h: u32) -> Raster {
        Raster::filled(w, h, [255, 255, 255, 255])
    }

    fn paint(img: &mut Raster, x0: u32, y0: u32, x1: u32, y1: u32, c: [u8; 4]) {
        for y in y0..y1 {
            for x in x0..x1 {
                img.set(x, y, c);
            }
        }
    }

    #[test]
    fn finds_content() {
        let mut im = white(100, 80);
        paint(&mut im, 20, 10, 60, 30, [0, 0, 0, 255]);
        assert_eq!(content(&im, im.full()), Some(Region { x: 20, y: 10, w: 40, h: 20 }));
    }

    #[test]
    fn transparent_counts_as_empty() {
        let mut im = Raster::filled(50, 50, [0, 0, 0, 0]);
        paint(&mut im, 5, 6, 7, 9, [10, 20, 30, 255]);
        assert_eq!(content(&im, im.full()), Some(Region { x: 5, y: 6, w: 2, h: 3 }));
    }

    #[test]
    fn near_white_counts_as_empty() {
        let mut im = white(40, 40);
        paint(&mut im, 0, 0, 40, 40, [252, 252, 252, 255]);
        assert_eq!(content(&im, im.full()), None);
        paint(&mut im, 10, 10, 11, 11, [200, 252, 252, 255]);
        assert_eq!(content(&im, im.full()), Some(Region { x: 10, y: 10, w: 1, h: 1 }));
    }

    #[test]
    fn fully_blank_page_stays_the_same() {
        let im = white(30, 30);
        assert_eq!(content(&im, im.full()), None);
        assert_eq!(trim(&im, im.full(), 8), im.full());
    }

    #[test]
    fn trim_with_margin_stays_within_bounds() {
        let mut im = white(100, 100);
        paint(&mut im, 3, 3, 10, 10, [0, 0, 0, 255]);
        let r = trim(&im, im.full(), 8);
        assert_eq!(r, Region { x: 0, y: 0, w: 18, h: 18 });
        paint(&mut im, 50, 50, 60, 60, [0, 0, 0, 255]);
        let r = trim(&im, im.full(), 5);
        assert_eq!(r, Region { x: 0, y: 0, w: 65, h: 65 });
    }

    #[test]
    fn trim_respects_given_region() {
        let mut im = white(100, 100);
        paint(&mut im, 0, 0, 5, 5, [0, 0, 0, 255]); // fora da região
        paint(&mut im, 40, 40, 50, 50, [0, 0, 0, 255]);
        let sub = Region { x: 20, y: 20, w: 60, h: 60 };
        assert_eq!(trim(&im, sub, 0), Region { x: 40, y: 40, w: 10, h: 10 });
        // A margem não invade fora da região.
        let r = trim(&im, Region { x: 38, y: 38, w: 20, h: 20 }, 10);
        assert_eq!(r, Region { x: 38, y: 38, w: 20, h: 20 });
    }

    #[test]
    fn pad_zero() {
        let c = Region { x: 4, y: 4, w: 2, h: 2 };
        assert_eq!(pad(c, 0, Region { x: 0, y: 0, w: 10, h: 10 }), c);
    }
}
