//! Embute no binário as versões dos motores (SHA, data, versão do crate) e da toolchain,
//! lidas de third_party/ (criado por scripts/fetch-upstream.sh). Sem git, usa variáveis de
//! ambiente THUMBCRAFT_<MOTOR>_SHA e THUMBCRAFT_<MOTOR>_DATE; na falta delas grava "desconhecido".

use std::path::Path;
use std::process::Command;

fn git(dir: &Path, args: &[&str]) -> Option<String> {
    let out = Command::new("git").arg("-C").arg(dir).args(args).output().ok()?;
    if !out.status.success() {
        return None;
    }
    let s = String::from_utf8(out.stdout).ok()?.trim().to_string();
    if s.is_empty() { None } else { Some(s) }
}

/// Primeira linha `version = "x"` depois de `[workspace.package]`.
fn crate_version(manifest: &Path) -> String {
    let Ok(txt) = std::fs::read_to_string(manifest) else { return "desconhecida".into() };
    let mut in_pkg = false;
    for line in txt.lines() {
        let l = line.trim();
        if l.starts_with('[') {
            in_pkg = l == "[workspace.package]";
            continue;
        }
        if in_pkg && l.starts_with("version") {
            if let Some(v) = l.split('"').nth(1) {
                return v.to_string();
            }
        }
    }
    "desconhecida".into()
}

fn main() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    println!("cargo:rerun-if-changed=build.rs");
    for (name, key) in [("wordcraft", "WORD"), ("gridcraft", "GRID"), ("deckcraft", "DECK")] {
        let dir = root.join("third_party").join(name);
        println!("cargo:rerun-if-changed={}", dir.join(".git/HEAD").display());
        println!("cargo:rerun-if-env-changed=THUMBCRAFT_{key}_SHA");
        println!("cargo:rerun-if-env-changed=THUMBCRAFT_{key}_DATE");
        let sha = git(&dir, &["rev-parse", "HEAD"]).or_else(|| std::env::var(format!("THUMBCRAFT_{key}_SHA")).ok()).unwrap_or_else(|| "desconhecido".into());
        let date = git(&dir, &["log", "-1", "--format=%cI"]).or_else(|| std::env::var(format!("THUMBCRAFT_{key}_DATE")).ok()).unwrap_or_else(|| "desconhecida".into());
        let ver = crate_version(&dir.join("Cargo.toml"));
        println!("cargo:rustc-env=TC_{key}_SHA={sha}");
        println!("cargo:rustc-env=TC_{key}_DATE={date}");
        println!("cargo:rustc-env=TC_{key}_VER={ver}");
    }
    // Marca de patch do GridCraft (hash dos patches aplicados por fetch-upstream.sh).
    let patch = std::fs::read_to_string(root.join("third_party/gridcraft/.git/thumbcraft-patched")).map(|s| s.trim().to_string()).unwrap_or_else(|_| "nenhum".into());
    println!("cargo:rustc-env=TC_GRID_PATCH={patch}");
    // Revisão do craft-libs: a do [patch.crates-io] do Cargo.toml raiz.
    println!("cargo:rerun-if-changed=../../Cargo.toml");
    let top = std::fs::read_to_string(root.join("Cargo.toml")).unwrap_or_default();
    let rev = top.split("rev = \"").nth(1).and_then(|s| s.split('"').next()).unwrap_or("desconhecida");
    println!("cargo:rustc-env=TC_CRAFT_LIBS_REV={rev}");
    // Revisão do craft-pdf usado para ler PDF: a da dependência craft-pdf-raster do próprio crate.
    println!("cargo:rerun-if-changed=Cargo.toml");
    let own = std::fs::read_to_string("Cargo.toml").unwrap_or_default();
    let pdf_rev = own.lines().find(|l| l.starts_with("craft-pdf-raster")).and_then(|l| l.split("rev = \"").nth(1)).and_then(|s| s.split('"').next()).unwrap_or("desconhecida");
    println!("cargo:rustc-env=TC_PDF_REV={pdf_rev}");
    // Versão do rustc.
    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".into());
    let rv = Command::new(rustc).arg("-V").output().ok().and_then(|o| String::from_utf8(o.stdout).ok()).map(|s| s.trim().to_string()).unwrap_or_else(|| "desconhecido".into());
    println!("cargo:rustc-env=TC_RUSTC={rv}");
}
