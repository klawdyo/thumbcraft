//! Testes de integração do pipeline completo (`post::process`) pela API pública, com a `Fake`.
//!
//! Complementam os testes unitários de cada módulo: aqui o que importa são invariantes que
//! valem para qualquer combinação de opções (tamanho final exato, opacidade, determinismo,
//! escolha da página) e a composição de opções que os testes unitários cobrem isoladas.

use thumbcraft_core::fake::{Fake, Pattern};
use thumbcraft_core::opts::{Background, Crop, Filter, Fit, Gravity, RenderScale};
use thumbcraft_core::{post, Opts};

fn opts(w: Option<u32>, h: Option<u32>, fit: Fit) -> Opts {
    Opts { width: w, height: h, fit, ..Opts::default() }
}

/// Com largura E altura, a imagem final tem exatamente esse tamanho, seja qual for a página,
/// o fit, a gravidade ou o filtro.
#[test]
fn exact_final_size_for_every_combination() {
    let pages = [(612.0, 792.0), (792.0, 612.0), (100.0, 100.0), (960.0, 540.0), (50.0, 1000.0)];
    let boxes = [(1, 1), (7, 3), (120, 120), (160, 90), (90, 160), (333, 111)];
    let gravities = ["top", "bottom", "left", "right", "center", "top-left", "bottom-right"];
    for (pw, ph) in pages {
        for (w, h) in boxes {
            for fit in [Fit::Cover, Fit::Contain] {
                for g in gravities {
                    for filter in [Filter::Lanczos3, Filter::Area] {
                        let mut f = Fake::new(1, pw, ph, Pattern::Quadrants);
                        let mut o = opts(Some(w), Some(h), fit);
                        o.gravity = Some(Gravity::parse(g).unwrap());
                        o.filter = filter;
                        let d = post::process(&mut f, &o).unwrap_or_else(|e| panic!("{pw}x{ph} -> {w}x{h} {fit:?} {g}: {e}"));
                        assert_eq!((d.img.w, d.img.h), (w, h), "{pw}x{ph} -> {w}x{h} {fit:?} {g}");
                        d.img.check().unwrap();
                    }
                }
            }
        }
    }
}

/// Com fundo branco ou colorido a imagem final é sempre opaca, mesmo com origem transparente.
#[test]
fn opaque_background_keeps_output_opaque() {
    for transparent in [false, true] {
        for bg in ["white", "#336699"] {
            for fit in [Fit::Cover, Fit::Contain] {
                let mut f = Fake::new(1, 200.0, 100.0, Pattern::Box(10.0, 10.0, 60.0, 40.0));
                f.transparent = transparent;
                let mut o = opts(Some(80), Some(80), fit);
                o.background = Background::parse(bg).unwrap();
                let d = post::process(&mut f, &o).unwrap();
                assert!(d.img.opaque(), "transparent={transparent} bg={bg} {fit:?}");
            }
        }
    }
}

/// Mesma entrada, mesma saída, byte a byte (nada depende de estado global nem de ordem).
#[test]
fn processing_is_deterministic() {
    let mut o = opts(Some(97), Some(61), Fit::Contain);
    o.trim = true;
    o.crop = Some(Crop::parse("0.1,0.1,0.8,0.8").unwrap());
    let run = || {
        let mut f = Fake::new(1, 300.0, 200.0, Pattern::Box(40.0, 30.0, 220.0, 120.0));
        post::process(&mut f, &o).unwrap()
    };
    let (a, b) = (run(), run());
    assert_eq!(a.img, b.img);
    assert_eq!((a.scale, a.native, a.region, a.renders), (b.scale, b.native, b.region, b.renders));
}

/// `--page N` renderiza a página N (índice N-1) e só ela.
#[test]
fn picks_the_requested_page() {
    for page in 1..=5u32 {
        let mut f = Fake::new(5, 100.0, 100.0, Pattern::Blank);
        let mut o = opts(Some(50), None, Fit::Cover);
        o.page = page;
        post::process(&mut f, &o).unwrap();
        assert_eq!(f.calls.len(), 1);
        assert_eq!(f.calls[0].0, (page - 1) as usize);
    }
    let mut f = Fake::new(5, 100.0, 100.0, Pattern::Blank);
    let mut o = Opts::default();
    o.page = 6;
    assert_eq!(post::process(&mut f, &o).unwrap_err().code(), 4);
    assert!(f.calls.is_empty(), "não deve renderizar uma página que não existe");
}

/// Documento sem páginas: erro 4, sem renderizar nada.
#[test]
fn document_without_pages_is_error_4() {
    let mut f = Fake::new(0, 100.0, 100.0, Pattern::Blank);
    let e = post::process(&mut f, &Opts::default()).unwrap_err();
    assert_eq!(e.code(), 4);
    assert!(f.calls.is_empty());
}

/// Crop e trim juntos: o trim age dentro do recorte e a região final cabe na renderização.
#[test]
fn crop_and_trim_together_stay_inside_page() {
    // Conteúdo no quadrante superior esquerdo; recorte pega a metade esquerda.
    let mut f = Fake::new(1, 400.0, 400.0, Pattern::Box(20.0, 20.0, 120.0, 90.0));
    let mut o = opts(Some(200), None, Fit::Cover);
    o.crop = Some(Crop::parse("0,0,0.5,1").unwrap());
    o.trim = true;
    o.trim_pad = 0;
    let d = post::process(&mut f, &o).unwrap();
    let r = d.region;
    assert!(r.x + r.w <= d.native.0 && r.y + r.h <= d.native.1);
    // Sem margem, a região é exatamente a caixa preta (em px nativos).
    let k = d.scale;
    assert_eq!((r.x, r.y), ((20.0 * k).round() as u32, (20.0 * k).round() as u32));
    assert_eq!((r.w, r.h), (((120.0 - 20.0) * k).round() as u32, ((90.0 - 20.0) * k).round() as u32));
    // A proporção da saída acompanha a da região (só largura dada).
    assert_eq!(d.img.w, 200);
    let expected = (200.0 * r.h as f64 / r.w as f64).round() as i64;
    assert!((d.img.h as i64 - expected).abs() <= 1, "{} vs {}", d.img.h, expected);
}

/// Escala fixa nunca re-renderiza, mesmo com trim; escala automática re-renderiza no máximo uma vez.
#[test]
fn at_most_two_renders() {
    for scale in [RenderScale::Auto, RenderScale::Fixed(1.5)] {
        let mut f = Fake::new(1, 600.0, 800.0, Pattern::Box(300.0, 400.0, 330.0, 430.0));
        let mut o = opts(Some(600), Some(600), Fit::Contain);
        o.trim = true;
        o.scale = scale;
        let d = post::process(&mut f, &o).unwrap();
        assert!(d.renders <= 2);
        assert_eq!(d.renders as usize, f.calls.len());
        if scale != RenderScale::Auto {
            assert_eq!(d.renders, 1);
        }
    }
}

/// Erro do motor no meio do caminho vira erro 4 e nenhuma imagem.
#[test]
fn engine_failure_is_error_4() {
    let mut f = Fake::new(1, 100.0, 100.0, Pattern::Blank);
    f.fail = true;
    assert_eq!(post::process(&mut f, &opts(Some(10), Some(10), Fit::Cover)).unwrap_err().code(), 4);
}

/// Opções inválidas falham com erro 2 antes de qualquer renderização.
#[test]
fn invalid_option_does_not_render() {
    let mut f = Fake::new(1, 100.0, 100.0, Pattern::Blank);
    let mut o = Opts::default();
    o.width = Some(0);
    assert_eq!(post::process(&mut f, &o).unwrap_err().code(), 2);
    o = Opts::default();
    o.page = 0;
    assert_eq!(post::process(&mut f, &o).unwrap_err().code(), 2);
    assert!(f.calls.is_empty());
}

/// Páginas pequenas e minúsculas não derrubam o pipeline nem produzem imagem vazia.
#[test]
fn tiny_pages() {
    for (pw, ph) in [(0.5, 0.5), (1.0, 1.0), (3.0, 700.0)] {
        let mut f = Fake::new(1, pw, ph, Pattern::Quadrants);
        let d = post::process(&mut f, &opts(Some(64), Some(64), Fit::Cover)).unwrap();
        assert_eq!((d.img.w, d.img.h), (64, 64));
        let d = post::process(&mut f, &Opts::default()).unwrap();
        assert!(d.img.w >= 1 && d.img.h >= 1);
    }
}
