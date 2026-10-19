use serde_json::json;
fn main() {
    let a: Vec<String> = std::env::args().collect();
    if a.len() < 3 { eprintln!("uso: wc-min in.(doc|docx) out.png [page]"); std::process::exit(2); }
    let page: u64 = a.get(3).and_then(|p| p.parse().ok()).unwrap_or(1);
    let region = wordcraft_engine::Region::from_locale_vars(|n| std::env::var(n).ok(), || None);
    let (doc, _) = wordcraft_engine::io::open_path_in(std::path::Path::new(&a[1]), None, &region).unwrap_or_else(|e| { eprintln!("{e}"); std::process::exit(1) });
    let mut s = wordcraft_engine::Session::new(doc);
    match s.run("file.exportPng", &json!({"path": a[2], "page": page, "scale": 2.0})) {
        Ok(r) => println!("{r}"),
        Err(e) => { eprintln!("{e}"); std::process::exit(1) }
    }
}
