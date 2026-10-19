// all-min IN OUT [N]: thumbnail-ish export chosen by extension.
// .doc/.docx -> PNG of page N (WordCraft engine); .pptx -> PNG of slide N-1 (DeckCraft engine);
// .xlsx -> PDF (GridCraft engine; substitute, GridCraft has no PNG path in its CLI/engine).
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 { eprintln!("uso: all-min IN OUT [N]"); std::process::exit(2); }
    let ext = std::path::Path::new(&a[1]).extension().and_then(|e| e.to_str()).unwrap_or("").to_ascii_lowercase();
    let n: u64 = a.get(3).and_then(|p| p.parse().ok()).unwrap_or(1);
    match ext.as_str() {
        #[cfg(feature = "word")]
        "doc" | "docx" => word(&a[1], &a[2], n),
        #[cfg(feature = "ppt")]
        "pptx" => ppt(&a[1], &a[2], n),
        #[cfg(feature = "xlsx")]
        "xlsx" | "xlsm" => xlsx(&a[1], &a[2]),
        _ => { eprintln!("extensao nao suportada: {ext}"); std::process::exit(2) }
    }
}

#[cfg(feature = "word")]
fn word(inp: &str, out: &str, page: u64) {
    use serde_json::json;
    let region = wordcraft_engine::Region::from_locale_vars(|n| std::env::var(n).ok(), || None);
    let (doc, _) = wordcraft_engine::io::open_path_in(std::path::Path::new(inp), None, &region).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(1) });
    let mut s = wordcraft_engine::Session::new(doc);
    match s.run("file.exportPng", &json!({"path": out, "page": page, "scale": 2.0})) {
        Ok(r) => println!("{r}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1) }
    }
}

#[cfg(feature = "ppt")]
fn ppt(inp: &str, out: &str, n: u64) {
    let bytes = std::fs::read(inp).unwrap_or_else(|e| { eprintln!("{inp}: {e}"); std::process::exit(1) });
    let idx = (n.max(1) - 1) as usize;
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
    } else { eprintln!("{inp}: not a presentation"); std::process::exit(1) };
    let db = deckcraft_fonts::FontDb::global();
    for font in &p.embedded_fonts {
        for id in font.faces().into_iter().flatten() {
            if let Some(item) = p.media(id) {
                if let Some(s) = deckcraft_fonts::embedded_sfnt(&item.data) { db.add_font(s.to_vec()); }
            }
        }
    }
    let _ = &mut p;
    if idx >= p.slides.len() { eprintln!("no slide {idx}"); std::process::exit(1) }
    let longest = p.slide_size.width.max(p.slide_size.height).max(1.0);
    let scale = 1.0f64.min(f64::from(deckcraft_render::MAX_SIDE) / longest);
    let opts = deckcraft_render::RenderOpts { scale, ..Default::default() };
    let img = deckcraft_render::render_slide(&p, idx, &opts);
    std::fs::write(out, img.to_png_at(deckcraft_render::dpi_for_scale(scale))).unwrap_or_else(|e| { eprintln!("{out}: {e}"); std::process::exit(1) });
    println!("{out} ({}x{})", img.width, img.height);
}

#[cfg(feature = "xlsx")]
fn xlsx(inp: &str, out: &str) {
    use serde_json::json;
    let mut s = gridcraft_engine::Session::new();
    if let Err(e) = s.run("file.open", json!({"path": inp})) { eprintln!("{e}"); std::process::exit(1) }
    match s.run("file.exportPdf", json!({"path": out})) {
        Ok(r) => println!("{r}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1) }
    }
}
