// dc-min IN OUT.png [SCALE] [SLIDE0]: first-slide PNG of a .pptx with deckcraft's own engine.
use deckcraft_fonts::FontDb;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 {
        eprintln!("usage: dc-min IN OUT.png [SCALE] [SLIDE_INDEX]");
        std::process::exit(2);
    }
    let bytes = match std::fs::read(&a[1]) {
        Ok(b) => b,
        Err(e) => { eprintln!("{}: {e}", a[1]); std::process::exit(1) }
    };
    let scale: f64 = a.get(3).and_then(|s| s.parse().ok()).unwrap_or(1.0);
    let idx: usize = a.get(4).and_then(|s| s.parse().ok()).unwrap_or(0);
    let mut p = if deckcraft_pptx::sniff(&bytes) {
        match deckcraft_pptx::import_checked(&bytes) {
            Ok((mut p, _)) => { deckcraft_format::repair(&mut p); p }
            Err(e) => { eprintln!("import: {e}"); std::process::exit(1) }
        }
    } else if deckcraft_format::sniff(&bytes) {
        match deckcraft_format::load_checked(&bytes) {
            Ok((p, _)) => p,
            Err(e) => { eprintln!("load: {e}"); std::process::exit(1) }
        }
    } else {
        eprintln!("{}: not a presentation deckcraft can read", a[1]);
        std::process::exit(1)
    };
    let db = FontDb::global();
    for font in &p.embedded_fonts {
        for id in font.faces().into_iter().flatten() {
            if let Some(item) = p.media(id) {
                if let Some(s) = deckcraft_fonts::embedded_sfnt(&item.data) { db.add_font(s.to_vec()); }
            }
        }
    }
    let _ = &mut p;
    if idx >= p.slides.len() { eprintln!("no slide {idx} (deck has {})", p.slides.len()); std::process::exit(1) }
    let longest = p.slide_size.width.max(p.slide_size.height).max(1.0);
    let scale = if scale.is_finite() { scale.clamp(0.01, 16.0) } else { 1.0 }.min(f64::from(deckcraft_render::MAX_SIDE) / longest);
    let opts = deckcraft_render::RenderOpts { scale, ..Default::default() };
    let img = deckcraft_render::render_slide(&p, idx, &opts);
    if let Err(e) = std::fs::write(&a[2], img.to_png_at(deckcraft_render::dpi_for_scale(scale))) { eprintln!("{}: {e}", a[2]); std::process::exit(1) }
    println!("{} ({}x{})", a[2], img.width, img.height);
}
