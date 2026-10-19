//! Escolha da escala de renderização (px/pt) e seus limites.

use crate::opts::{Fit, Opts, RenderScale};

/// Limites de segurança da renderização nativa.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    /// Escala mínima em `auto` (px/pt).
    pub min_scale: f64,
    /// Maior lado da renderização, em px.
    pub max_side: u32,
    /// Maior número de pixels da renderização.
    pub max_pixels: u64,
}

pub const LIMITS: Limits = Limits { min_scale: 1.0, max_side: 8192, max_pixels: 64_000_000 };

/// Escala dos minis (WordCraft, DeckCraft e GridCraft), usada quando não há alvo de tamanho.
pub const DEFAULT_SCALE: f64 = 2.0;

/// Fator de supersampling: a renderização tem ao menos 2x o tamanho do alvo.
pub const SUPERSAMPLE: f64 = 2.0;

/// Maior escala que respeita os limites para uma página de `pw` x `ph` pontos.
pub fn max_scale(pw: f64, ph: f64, lim: &Limits) -> f64 {
    let by_side = (lim.max_side as f64 / pw).min(lim.max_side as f64 / ph);
    let by_pixels = (lim.max_pixels as f64 / (pw * ph)).sqrt();
    by_side.min(by_pixels)
}

/// Aplica os limites a uma escala pedida (o limite vence o mínimo de 1 px/pt).
pub fn clamp_scale(pw: f64, ph: f64, s: f64, lim: &Limits) -> f64 {
    s.min(max_scale(pw, ph, lim))
}

/// Escala necessária para que uma região de `rw` x `rh` pontos, ao ir para o alvo de `opts`,
/// seja reduzida por um fator de pelo menos [`SUPERSAMPLE`]. Não aplica limites.
pub fn needed_scale(rw: f64, rh: f64, opts: &Opts) -> f64 {
    let sw = opts.width.map(|w| w as f64 / rw);
    let sh = opts.height.map(|h| h as f64 / rh);
    let out_per_pt = match (sw, sh) {
        (None, None) => return DEFAULT_SCALE,
        (Some(a), None) | (None, Some(a)) => a,
        (Some(a), Some(b)) => match opts.fit {
            Fit::Cover => a.max(b),
            Fit::Contain => a.min(b),
        },
    };
    SUPERSAMPLE * out_per_pt
}

/// Escala final para a página inteira de `pw` x `ph` pontos, com a região de interesse de
/// `rw` x `rh` pontos (a página toda quando não há recorte).
pub fn choose(pw: f64, ph: f64, rw: f64, rh: f64, opts: &Opts, lim: &Limits) -> f64 {
    let want = match opts.scale {
        RenderScale::Fixed(v) => v,
        RenderScale::Auto => needed_scale(rw, rh, opts).max(lim.min_scale),
    };
    clamp_scale(pw, ph, want, lim)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn o(w: Option<u32>, h: Option<u32>, fit: Fit) -> Opts {
        Opts { width: w, height: h, fit, ..Opts::default() }
    }

    #[test]
    fn no_target_uses_minis_scale() {
        assert_eq!(choose(595.0, 842.0, 595.0, 842.0, &o(None, None, Fit::Cover), &LIMITS), 2.0);
    }

    #[test]
    fn small_target_falls_to_minimum() {
        // 120x120 cover de uma A4: 2 * 120/595 = 0,4, sobe para 1 px/pt.
        let s = choose(595.0, 842.0, 595.0, 842.0, &o(Some(120), Some(120), Fit::Cover), &LIMITS);
        assert_eq!(s, 1.0);
    }

    #[test]
    fn large_target_doubles_target_side() {
        // Só largura 1200: o lado em px da renderização é 2400 (2 * 1200), escala 2400/595.
        let s = choose(595.0, 842.0, 595.0, 842.0, &o(Some(1200), None, Fit::Cover), &LIMITS);
        assert!((s - 2400.0 / 595.0).abs() < 1e-9);
    }

    #[test]
    fn cover_uses_most_demanding_axis() {
        // Slide 960x540 em 480x480 cover: o eixo da altura manda (480/540 por pt).
        let s = choose(960.0, 540.0, 960.0, 540.0, &o(Some(480), Some(480), Fit::Cover), &LIMITS);
        assert!((s - 2.0 * 480.0 / 540.0).abs() < 1e-9);
    }

    #[test]
    fn contain_uses_least_demanding_axis() {
        let s = needed_scale(960.0, 540.0, &o(Some(480), Some(480), Fit::Contain));
        assert!((s - 2.0 * 480.0 / 960.0).abs() < 1e-9);
    }

    #[test]
    fn side_and_pixel_limit() {
        // Alvo gigante: o lado maior da renderização não passa de 8192.
        let s = choose(595.0, 842.0, 595.0, 842.0, &o(Some(16_000), None, Fit::Cover), &LIMITS);
        assert!((842.0 * s - 8192.0).abs() < 1e-6);
        // Página enorme (sem relação com lado): o limite de 64 MP vale.
        let lim = Limits { min_scale: 1.0, max_side: 100_000, max_pixels: 64_000_000 };
        let s = choose(10_000.0, 10_000.0, 10_000.0, 10_000.0, &o(None, None, Fit::Cover), &lim);
        assert!((10_000.0 * 10_000.0 * s * s - 64_000_000.0).abs() < 1.0);
    }

    #[test]
    fn limit_beats_minimum_on_giant_page() {
        let s = choose(20_000.0, 20_000.0, 20_000.0, 20_000.0, &o(Some(100), Some(100), Fit::Cover), &LIMITS);
        assert!(s < 1.0);
        assert!(20_000.0 * s <= 8192.0 + 1e-6);
    }

    #[test]
    fn fixed_scale_respects_limits() {
        let mut op = o(Some(100), Some(100), Fit::Cover);
        op.scale = RenderScale::Fixed(3.0);
        assert_eq!(choose(595.0, 842.0, 595.0, 842.0, &op, &LIMITS), 3.0);
        op.scale = RenderScale::Fixed(50.0);
        let s = choose(595.0, 842.0, 595.0, 842.0, &op, &LIMITS);
        assert!(s < 50.0 && 842.0 * s <= 8192.0 + 1e-6);
    }

    #[test]
    fn small_crop_requests_larger_scale() {
        // Região de 100x100 pt para 200 px de largura: 4 px/pt.
        let s = choose(595.0, 842.0, 100.0, 100.0, &o(Some(200), None, Fit::Cover), &LIMITS);
        assert!((s - 4.0).abs() < 1e-9);
    }
}
