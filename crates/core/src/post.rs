//! Pós-processamento: renderização em alta resolução, recorte, trim, ajuste e redução.

use crate::error::{Error, Res};
use crate::fit::{self, Plan};
use crate::opts::{Background, Opts};
use crate::raster::{Raster, Region};
use crate::render::Render;
use crate::resize::resample;
use crate::scale::{self, LIMITS};
use crate::trim;

/// Resultado do pipeline.
#[derive(Debug)]
pub struct Done {
    /// Imagem final (alfa pré-multiplicado; opaca se o fundo não é transparente).
    pub img: Raster,
    /// Escala de renderização usada (px/pt).
    pub scale: f64,
    /// Tamanho da renderização nativa, em px.
    pub native: (u32, u32),
    /// Região da renderização que virou a imagem (depois de crop e trim).
    pub region: Region,
    /// Quantas vezes o motor renderizou (2 quando o trim pediu mais resolução).
    pub renders: u32,
}

/// Copia a região `r` de `img`.
pub fn extract(img: &Raster, r: Region) -> Raster {
    let mut px = Vec::with_capacity(r.w as usize * r.h as usize * 4);
    for y in r.y..r.y + r.h {
        let a = (y as usize * img.w as usize + r.x as usize) * 4;
        px.extend_from_slice(&img.px[a..a + r.w as usize * 4]);
    }
    Raster { w: r.w, h: r.h, px }
}

/// Compõe sobre uma cor opaca (deixa a imagem inteira opaca).
pub fn flatten(img: &mut Raster, bg: [u8; 3]) {
    for p in img.px.chunks_exact_mut(4) {
        let a = p[3] as u32;
        if a == 255 {
            continue;
        }
        let back = 255 - a;
        for c in 0..3 {
            p[c] = (p[c] as u32 + (bg[c] as u32 * back + 127) / 255).min(255) as u8;
        }
        p[3] = 255;
    }
}

fn blit(dst: &mut Raster, src: &Raster, ox: u32, oy: u32) {
    for y in 0..src.h {
        let s = (y as usize * src.w as usize) * 4;
        let d = ((y + oy) as usize * dst.w as usize + ox as usize) * 4;
        dst.px[d..d + src.w as usize * 4].copy_from_slice(&src.px[s..s + src.w as usize * 4]);
    }
}

/// Monta a imagem final a partir do plano.
fn assemble(img: &Raster, region: Region, plan: &Plan, o: &Opts) -> Raster {
    let whole = plan.src.x == 0.0
        && plan.src.y == 0.0
        && plan.src.w == region.w as f64
        && plan.src.h == region.h as f64
        && plan.inner_w == region.w
        && plan.inner_h == region.h;
    let mut inner = if whole {
        extract(img, region)
    } else {
        let win = fit::RectF { x: region.x as f64 + plan.src.x, y: region.y as f64 + plan.src.y, w: plan.src.w, h: plan.src.h };
        resample(img, region, win, plan.inner_w, plan.inner_h, o.filter)
    };
    if let Background::Solid(bg) = o.background {
        flatten(&mut inner, bg);
    }
    if (plan.inner_w, plan.inner_h) == (plan.out_w, plan.out_h) {
        return inner;
    }
    let fill = match o.background {
        Background::Solid(c) => [c[0], c[1], c[2], 255],
        Background::Transparent => [0, 0, 0, 0],
    };
    let mut canvas = Raster::filled(plan.out_w, plan.out_h, fill);
    blit(&mut canvas, &inner, plan.off_x, plan.off_y);
    canvas
}

/// Executa o pipeline completo sobre um documento aberto.
pub fn process(r: &mut dyn Render, o: &Opts) -> Res<Done> {
    o.check()?;
    let n = r.pages();
    let page = o.page as usize;
    if page > n {
        return Err(Error::Invalid(format!("o arquivo não tem {} {}: só há {} no total", r.unit(), page, n)));
    }
    let idx = page - 1;
    let (pw, ph) = r.size(idx)?;
    if !(pw.is_finite() && ph.is_finite() && pw > 0.0 && ph > 0.0) {
        return Err(Error::Invalid(format!("a {} {} tem tamanho inválido", r.unit(), page)));
    }
    let (fw, fh) = o.crop.map_or((1.0, 1.0), |c| (c.w, c.h));
    let mut scale = scale::choose(pw, ph, pw * fw, ph * fh, o, &LIMITS);
    let mut renders = 0u32;
    loop {
        let img = r.render(idx, scale)?;
        renders += 1;
        img.check()?;
        let mut region = o.crop.map_or(img.full(), |c| c.to_region(img.w, img.h));
        if o.trim {
            region = trim::trim(&img, region, o.trim_pad);
            // O conteúdo ocupa só parte da região: se a resolução ficou curta para o alvo,
            // renderiza de novo, uma única vez, na escala que o conteúdo pede.
            if renders == 1 && o.scale == crate::opts::RenderScale::Auto {
                let (kx, ky) = (img.w as f64 / pw, img.h as f64 / ph);
                let want = scale::choose(pw, ph, region.w as f64 / kx, region.h as f64 / ky, o, &LIMITS);
                if want > scale * 1.25 {
                    scale = want;
                    continue;
                }
            }
        }
        let plan = fit::plan(region.w, region.h, o.width, o.height, o.fit, o.gravity())?;
        let out = assemble(&img, region, &plan, o);
        return Ok(Done { img: out, scale, native: (img.w, img.h), region, renders });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fake::{Fake, Pattern};
    use crate::opts::{Background, Crop, Fit, Gravity, RenderScale};

    fn opts(w: Option<u32>, h: Option<u32>, fit: Fit) -> Opts {
        Opts { width: w, height: h, fit, ..Opts::default() }
    }

    #[test]
    fn width_only_sizes_and_keeps_aspect_ratio() {
        let mut f = Fake::new(1, 600.0, 800.0, Pattern::Blank);
        let d = process(&mut f, &opts(Some(120), None, Fit::Cover)).unwrap();
        assert_eq!((d.img.w, d.img.h), (120, 160));
        assert!(d.img.opaque());
    }

    #[test]
    fn renders_at_least_double_the_target() {
        let mut f = Fake::new(1, 600.0, 800.0, Pattern::Blank);
        let d = process(&mut f, &opts(Some(1200), Some(1200), Fit::Cover)).unwrap();
        // Alvo 1200x1200 cover de 600x800: escala necessária 2 * 1200/600 = 4.
        assert_eq!(f.calls.len(), 1);
        assert!((f.calls[0].1 - 4.0).abs() < 1e-9);
        assert!(d.native.0 >= 2400);
    }

    #[test]
    fn small_target_uses_minimum_scale_1() {
        let mut f = Fake::new(1, 600.0, 800.0, Pattern::Blank);
        process(&mut f, &opts(Some(120), Some(120), Fit::Cover)).unwrap();
        assert_eq!(f.calls[0].1, 1.0);
    }

    #[test]
    fn no_target_renders_at_2_and_does_not_shrink() {
        let mut f = Fake::new(1, 300.0, 400.0, Pattern::Blank);
        let d = process(&mut f, &Opts::default()).unwrap();
        assert_eq!(f.calls[0].1, 2.0);
        assert_eq!((d.img.w, d.img.h), (600, 800));
    }

    #[test]
    fn cover_gravity_top_and_bottom() {
        // Página 100x200 com quadrantes; caixa 40x40 cover: sobra altura.
        let mut f = Fake::new(1, 100.0, 200.0, Pattern::Quadrants);
        let mut o = opts(Some(40), Some(40), Fit::Cover);
        o.scale = RenderScale::Fixed(1.0);
        o.filter = crate::opts::Filter::Area;
        // Topo: janela y de 0 a 100 => só vermelho e verde.
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.img.at(2, 2), [255, 0, 0, 255]);
        assert_eq!(d.img.at(37, 37), [0, 255, 0, 255]);
        // Base: janela y de 100 a 200 => azul e amarelo.
        o.gravity = Some(Gravity::parse("bottom").unwrap());
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.img.at(2, 2), [0, 0, 255, 255]);
        assert_eq!(d.img.at(37, 37), [255, 255, 0, 255]);
    }

    #[test]
    fn contain_fills_with_background_and_positions() {
        let mut f = Fake::new(1, 100.0, 200.0, Pattern::Quadrants);
        let mut o = opts(Some(100), Some(100), Fit::Contain);
        o.scale = RenderScale::Fixed(1.0);
        o.filter = crate::opts::Filter::Area;
        o.background = Background::parse("#102030").unwrap();
        let d = process(&mut f, &o).unwrap();
        assert_eq!((d.img.w, d.img.h), (100, 100));
        // Imagem reduzida 50x100 centralizada: faixa de fundo de 25 px de cada lado.
        assert_eq!(d.img.at(0, 50), [0x10, 0x20, 0x30, 255]);
        assert_eq!(d.img.at(99, 50), [0x10, 0x20, 0x30, 255]);
        assert_eq!(d.img.at(26, 2), [255, 0, 0, 255]);
        // À esquerda.
        o.gravity = Some(Gravity::parse("left").unwrap());
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.img.at(2, 2), [255, 0, 0, 255]);
        assert_eq!(d.img.at(99, 2), [0x10, 0x20, 0x30, 255]);
    }

    #[test]
    fn contain_transparent_background_keeps_alpha() {
        let mut f = Fake::new(1, 100.0, 200.0, Pattern::Quadrants);
        let mut o = opts(Some(100), Some(100), Fit::Contain);
        o.background = Background::Transparent;
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.img.at(0, 50)[3], 0);
        assert!(!d.img.opaque());
    }

    #[test]
    fn transparent_source_background_becomes_white_by_default() {
        let mut f = Fake::new(1, 50.0, 50.0, Pattern::Box(10.0, 10.0, 20.0, 20.0));
        f.transparent = true;
        let d = process(&mut f, &opts(Some(25), None, Fit::Cover)).unwrap();
        assert!(d.img.opaque());
        assert_eq!(d.img.at(0, 0), [255, 255, 255, 255]);
        let mut o = opts(Some(25), None, Fit::Cover);
        o.background = Background::Transparent;
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.img.at(0, 0)[3], 0);
    }

    #[test]
    fn crop_in_fractions_before_resize() {
        // Metade direita da página: verde em cima, amarelo embaixo.
        let mut f = Fake::new(1, 100.0, 100.0, Pattern::Quadrants);
        let mut o = opts(Some(10), Some(10), Fit::Cover);
        o.crop = Some(Crop::parse("0.5,0,0.5,1").unwrap());
        o.scale = RenderScale::Fixed(2.0);
        o.filter = crate::opts::Filter::Area;
        let d = process(&mut f, &o).unwrap();
        assert_eq!(d.region, Region { x: 100, y: 0, w: 100, h: 200 });
        // Cover quadrado do recorte 100x200: topo => só verde.
        assert_eq!(d.img.at(5, 5), [0, 255, 0, 255]);
    }

    #[test]
    fn crop_requests_more_resolution() {
        // Recorte de 10% (50 pt) para 100 px de largura: 2 px/pt de saída, 4 px/pt renderizados.
        let mut f = Fake::new(1, 500.0, 500.0, Pattern::Blank);
        let mut o = opts(Some(100), None, Fit::Cover);
        o.crop = Some(Crop::parse("0,0,0.1,0.1").unwrap());
        process(&mut f, &o).unwrap();
        assert!((f.calls[0].1 - 4.0).abs() < 1e-9);
    }

    #[test]
    fn trim_cuts_margins_and_rerenders_at_higher_resolution() {
        // Conteúdo de 50x50 pt no meio de uma A4; alvo de 400 px de largura.
        let mut f = Fake::new(1, 595.0, 842.0, Pattern::Box(100.0, 100.0, 150.0, 150.0));
        let mut o = opts(Some(400), None, Fit::Cover);
        o.trim = true;
        o.trim_pad = 0;
        let d = process(&mut f, &o).unwrap();
        assert_eq!(f.calls.len(), 2, "uma segunda renderização com a escala do conteúdo");
        assert_eq!(d.renders, 2);
        assert!(f.calls[1].1 > f.calls[0].1 * 2.0);
        // A região final é o quadrado preto, e a imagem sai toda preta.
        assert_eq!(d.region.w, d.region.h);
        assert_eq!((d.img.w, d.img.h), (400, 400));
        assert_eq!(d.img.at(200, 200), [0, 0, 0, 255]);
        assert_eq!(d.img.at(0, 0), [0, 0, 0, 255]);
    }

    #[test]
    fn trim_without_need_for_more_resolution_renders_once() {
        let mut f = Fake::new(1, 200.0, 200.0, Pattern::Box(10.0, 10.0, 190.0, 190.0));
        let mut o = opts(Some(50), None, Fit::Cover);
        o.trim = true;
        process(&mut f, &o).unwrap();
        assert_eq!(f.calls.len(), 1);
    }

    #[test]
    fn trim_on_blank_page_does_not_break() {
        let mut f = Fake::new(1, 200.0, 300.0, Pattern::Blank);
        let mut o = opts(Some(40), Some(40), Fit::Contain);
        o.trim = true;
        let d = process(&mut f, &o).unwrap();
        assert_eq!((d.img.w, d.img.h), (40, 40));
        assert_eq!(d.region.w * 3, d.region.h * 2);
    }

    #[test]
    fn trim_respects_requested_margin() {
        let mut f = Fake::new(1, 100.0, 100.0, Pattern::Box(40.0, 40.0, 60.0, 60.0));
        let mut o = Opts::default();
        o.scale = RenderScale::Fixed(1.0);
        o.trim = true;
        o.trim_pad = 5;
        let d = process(&mut f, &o).unwrap();
        assert_eq!((d.img.w, d.img.h), (30, 30));
    }

    #[test]
    fn fixed_scale_does_not_rerender() {
        let mut f = Fake::new(1, 595.0, 842.0, Pattern::Box(100.0, 100.0, 150.0, 150.0));
        let mut o = opts(Some(400), None, Fit::Cover);
        o.trim = true;
        o.scale = RenderScale::Fixed(1.0);
        process(&mut f, &o).unwrap();
        assert_eq!(f.calls.len(), 1);
    }

    #[test]
    fn missing_page_is_error_4() {
        let mut f = Fake::new(3, 100.0, 100.0, Pattern::Blank);
        let mut o = Opts::default();
        o.page = 4;
        let e = process(&mut f, &o).unwrap_err();
        assert_eq!(e.code(), 4);
        assert!(e.msg().contains('4') && e.msg().contains('3'), "{e}");
        o.page = 3;
        assert!(process(&mut f, &o).is_ok());
        assert_eq!(f.calls.last().unwrap().0, 2, "a CLI conta de 1, o motor de 0");
    }

    #[test]
    fn engine_failure_propagates() {
        let mut f = Fake::new(1, 100.0, 100.0, Pattern::Blank);
        f.fail = true;
        assert_eq!(process(&mut f, &Opts::default()).unwrap_err().code(), 4);
    }

    #[test]
    fn invalid_page_size() {
        let mut f = Fake::new(1, 0.0, 100.0, Pattern::Blank);
        assert_eq!(process(&mut f, &Opts::default()).unwrap_err().code(), 4);
        let mut f = Fake::new(1, f64::NAN, 100.0, Pattern::Blank);
        assert_eq!(process(&mut f, &Opts::default()).unwrap_err().code(), 4);
    }

    #[test]
    fn invalid_options_are_error_2() {
        let mut f = Fake::new(1, 100.0, 100.0, Pattern::Blank);
        assert_eq!(process(&mut f, &opts(Some(0), None, Fit::Cover)).unwrap_err().code(), 2);
        assert!(f.calls.is_empty(), "nem renderiza");
    }

    #[test]
    fn render_limits() {
        // Alvo enorme numa página A4: a renderização para em 8192 px de lado maior.
        let mut f = Fake::new(1, 59.5, 84.2, Pattern::Blank);
        let d = process(&mut f, &opts(Some(16_000), Some(100), Fit::Cover)).unwrap();
        assert!(d.native.0 <= 8192 && d.native.1 <= 8192);
        assert_eq!((d.img.w, d.img.h), (16_000, 100));
    }

    #[test]
    fn degenerate_box_1x1() {
        let mut f = Fake::new(1, 100.0, 100.0, Pattern::Quadrants);
        let d = process(&mut f, &opts(Some(1), Some(1), Fit::Cover)).unwrap();
        assert_eq!((d.img.w, d.img.h), (1, 1));
        assert!(d.img.opaque());
    }

    #[test]
    fn extreme_ratio_with_single_dimension_is_error_2() {
        let mut f = Fake::new(1, 1.0, 5000.0, Pattern::Blank);
        let e = process(&mut f, &opts(Some(2000), None, Fit::Cover)).unwrap_err();
        assert_eq!(e.code(), 2);
    }

    #[test]
    fn flatten_composites_over_color() {
        let mut r = Raster::filled(1, 1, [0, 0, 0, 0]);
        flatten(&mut r, [10, 20, 30]);
        assert_eq!(r.at(0, 0), [10, 20, 30, 255]);
        // Meio transparente: preto a 50% sobre branco dá cinza.
        let mut r = Raster::filled(1, 1, [0, 0, 0, 128]);
        flatten(&mut r, [255, 255, 255]);
        let p = r.at(0, 0);
        assert!((p[0] as i32 - 127).abs() <= 1 && p[3] == 255);
    }
}
