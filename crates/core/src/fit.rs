//! Matemática do ajuste (fit) de uma região renderizada a uma caixa de saída.

use crate::error::{Error, Res};
use crate::opts::{Fit, Gravity, MAX_OUT_PIXELS, MAX_OUT_SIDE};

/// Retângulo fracionário (coordenadas relativas à região de origem).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RectF {
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
}

/// Plano de saída: tamanho final, onde a imagem reduzida entra e de que parte da origem sai.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Plan {
    /// Tamanho final da imagem.
    pub out_w: u32,
    pub out_h: u32,
    /// Tamanho da imagem reduzida (igual ao final, exceto no contain).
    pub inner_w: u32,
    pub inner_h: u32,
    /// Posição da imagem reduzida dentro da final (só difere de 0 no contain).
    pub off_x: u32,
    pub off_y: u32,
    /// Parte da região de origem que vira a imagem reduzida (só difere da região toda no cover).
    pub src: RectF,
}

fn limit(w: u32, h: u32) -> Res<()> {
    if w == 0 || h == 0 || w > MAX_OUT_SIDE || h > MAX_OUT_SIDE || w as u64 * h as u64 > MAX_OUT_PIXELS {
        return Err(Error::Usage(format!(
            "a proporção da página dá uma imagem de {w}x{h} px, fora do limite; informe largura e altura juntas"
        )));
    }
    Ok(())
}

/// Calcula o plano para uma região `rw` x `rh` (px, ambos maiores que 0).
///
/// - sem largura nem altura: a região sai como está;
/// - só uma dimensão: a outra preserva a proporção (mínimo 1 px);
/// - as duas: aplica `fit` com a âncora `grav`.
pub fn plan(rw: u32, rh: u32, width: Option<u32>, height: Option<u32>, fit: Fit, grav: Gravity) -> Res<Plan> {
    if rw == 0 || rh == 0 {
        return Err(Error::Invalid("a região a ser reduzida não tem área".into()));
    }
    let full = RectF { x: 0.0, y: 0.0, w: rw as f64, h: rh as f64 };
    let simple = |w: u32, h: u32| Plan { out_w: w, out_h: h, inner_w: w, inner_h: h, off_x: 0, off_y: 0, src: full };
    let (frw, frh) = (rw as f64, rh as f64);
    match (width, height) {
        (None, None) => Ok(simple(rw, rh)),
        (Some(w), None) => {
            let h = ((w as f64 * frh / frw).round() as u64).max(1);
            let h = u32::try_from(h).unwrap_or(u32::MAX);
            limit(w, h)?;
            Ok(simple(w, h))
        }
        (None, Some(h)) => {
            let w = ((h as f64 * frw / frh).round() as u64).max(1);
            let w = u32::try_from(w).unwrap_or(u32::MAX);
            limit(w, h)?;
            Ok(simple(w, h))
        }
        (Some(w), Some(h)) => {
            limit(w, h)?;
            let (fw, fh) = (w as f64, h as f64);
            match fit {
                Fit::Cover => {
                    let s = (fw / frw).max(fh / frh);
                    let sw = (fw / s).min(frw);
                    let sh = (fh / s).min(frh);
                    let src = RectF { x: (frw - sw) * grav.x, y: (frh - sh) * grav.y, w: sw, h: sh };
                    Ok(Plan { out_w: w, out_h: h, inner_w: w, inner_h: h, off_x: 0, off_y: 0, src })
                }
                Fit::Contain => {
                    let s = (fw / frw).min(fh / frh);
                    let iw = ((frw * s).round() as u32).clamp(1, w);
                    let ih = ((frh * s).round() as u32).clamp(1, h);
                    let off_x = ((w - iw) as f64 * grav.x).round() as u32;
                    let off_y = ((h - ih) as f64 * grav.y).round() as u32;
                    Ok(Plan { out_w: w, out_h: h, inner_w: iw, inner_h: ih, off_x, off_y, src: full })
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const T: Gravity = Gravity::TOP;
    const C: Gravity = Gravity::CENTER;

    #[test]
    fn no_dimensions_keeps_region() {
        let p = plan(300, 400, None, None, Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (300, 400));
        assert_eq!(p.src, RectF { x: 0.0, y: 0.0, w: 300.0, h: 400.0 });
    }

    #[test]
    fn width_only_keeps_aspect_ratio() {
        let p = plan(600, 800, Some(120), None, Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (120, 160));
        let p = plan(600, 800, Some(100), None, Fit::Contain, C).unwrap();
        assert_eq!((p.out_w, p.out_h), (100, 133));
    }

    #[test]
    fn height_only_keeps_aspect_ratio() {
        let p = plan(600, 800, None, Some(200), Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (150, 200));
    }

    #[test]
    fn single_dimension_never_rounds_to_zero() {
        let p = plan(1000, 1, Some(10), None, Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (10, 1));
        let p = plan(1, 1000, None, Some(10), Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (1, 10));
    }

    #[test]
    fn extreme_ratio_exceeds_limit() {
        assert!(plan(1, 10_000, Some(5000), None, Fit::Cover, T).is_err());
    }

    #[test]
    fn cover_portrait_in_square_box_crops_height_from_top() {
        // Página 600x800 em caixa 100x100: sobra altura. Fica o topo.
        let p = plan(600, 800, Some(100), Some(100), Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (100, 100));
        assert_eq!(p.src, RectF { x: 0.0, y: 0.0, w: 600.0, h: 600.0 });
    }

    #[test]
    fn cover_vertical_gravities() {
        let c = plan(600, 800, Some(100), Some(100), Fit::Cover, C).unwrap();
        assert_eq!(c.src.y, 100.0);
        let b = plan(600, 800, Some(100), Some(100), Fit::Cover, Gravity::parse("bottom").unwrap()).unwrap();
        assert_eq!(b.src.y, 200.0);
        assert_eq!(b.src.x, 0.0);
    }

    #[test]
    fn cover_landscape_in_square_box_uses_horizontal_gravity() {
        // Slide 1600x900 em 100x100: sobra largura.
        let left_plan = plan(1600, 900, Some(100), Some(100), Fit::Cover, Gravity::parse("left").unwrap()).unwrap();
        assert_eq!(left_plan.src, RectF { x: 0.0, y: 0.0, w: 900.0, h: 900.0 });
        let dir = plan(1600, 900, Some(100), Some(100), Fit::Cover, Gravity::parse("right").unwrap()).unwrap();
        assert_eq!(dir.src.x, 700.0);
        // Padrão top centraliza na horizontal.
        let top_plan = plan(1600, 900, Some(100), Some(100), Fit::Cover, T).unwrap();
        assert_eq!(top_plan.src.x, 350.0);
    }

    #[test]
    fn cover_same_ratio_does_not_crop() {
        let p = plan(1600, 900, Some(320), Some(180), Fit::Cover, T).unwrap();
        assert!((p.src.w - 1600.0).abs() < 1e-9 && (p.src.h - 900.0).abs() < 1e-9);
    }

    #[test]
    fn contain_portrait_in_square_box() {
        let p = plan(600, 800, Some(200), Some(200), Fit::Contain, C).unwrap();
        assert_eq!((p.inner_w, p.inner_h), (150, 200));
        assert_eq!((p.off_x, p.off_y), (25, 0));
        assert_eq!((p.out_w, p.out_h), (200, 200));
    }

    #[test]
    fn contain_gravities() {
        let e = plan(600, 800, Some(200), Some(200), Fit::Contain, Gravity::parse("left").unwrap()).unwrap();
        assert_eq!(e.off_x, 0);
        let d = plan(600, 800, Some(200), Some(200), Fit::Contain, Gravity::parse("right").unwrap()).unwrap();
        assert_eq!(d.off_x, 50);
        // Slide em caixa alta: sobra altura.
        let t = plan(1600, 900, Some(160), Some(160), Fit::Contain, Gravity::parse("top").unwrap()).unwrap();
        assert_eq!((t.inner_w, t.inner_h), (160, 90));
        assert_eq!((t.off_x, t.off_y), (0, 0));
        let b = plan(1600, 900, Some(160), Some(160), Fit::Contain, Gravity::parse("bottom-right").unwrap()).unwrap();
        assert_eq!((b.off_x, b.off_y), (0, 70));
    }

    #[test]
    fn contain_never_exceeds_box_nor_zero() {
        // Região fina em caixa pequena: o lado menor arredonda para 1.
        let p = plan(10_000, 3, Some(50), Some(50), Fit::Contain, C).unwrap();
        assert_eq!(p.inner_w, 50);
        assert_eq!(p.inner_h, 1);
        assert!(p.off_y <= 49);
    }

    #[test]
    fn box_larger_than_region_enlarges() {
        let p = plan(100, 100, Some(400), Some(400), Fit::Contain, C).unwrap();
        assert_eq!((p.inner_w, p.inner_h), (400, 400));
        let p = plan(100, 50, Some(400), Some(400), Fit::Cover, T).unwrap();
        assert!((p.src.w - 50.0).abs() < 1e-9 && (p.src.h - 50.0).abs() < 1e-9);
    }

    #[test]
    fn degenerate_box_one_pixel() {
        let p = plan(600, 800, Some(1), Some(1), Fit::Cover, T).unwrap();
        assert_eq!((p.out_w, p.out_h), (1, 1));
        let p = plan(600, 800, Some(1), Some(1), Fit::Contain, C).unwrap();
        assert_eq!((p.inner_w, p.inner_h), (1, 1));
        let p = plan(1, 1, Some(1), Some(1), Fit::Cover, T).unwrap();
        assert_eq!(p.src, RectF { x: 0.0, y: 0.0, w: 1.0, h: 1.0 });
    }

    #[test]
    fn output_limits() {
        assert!(plan(10, 10, Some(MAX_OUT_SIDE + 1), Some(10), Fit::Cover, T).is_err());
        assert!(plan(10, 10, Some(16_000), Some(16_000), Fit::Cover, T).is_err());
        assert!(plan(10, 10, Some(MAX_OUT_SIDE), Some(1), Fit::Cover, T).is_ok());
    }

    #[test]
    fn empty_region_is_error() {
        assert!(plan(0, 10, Some(10), Some(10), Fit::Cover, T).is_err());
        assert!(plan(10, 0, None, None, Fit::Cover, T).is_err());
    }
}
