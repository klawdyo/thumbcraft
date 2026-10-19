//! Inspeção do arquivo de entrada e abertura pelo motor certo.

use std::io::Read;
use std::path::Path;

use thumbcraft_core::route::{self, Engine, Sniff, ZipKind};
use thumbcraft_core::{Error, Render, Res};

use crate::{deck::Deck, grid::Grid, pdf::Pdf, word::Word};

/// Lê uma entrada do ZIP como texto, até `max` bytes.
fn zip_text(z: &mut zip::ZipArchive<std::fs::File>, name: &str, max: u64) -> Option<String> {
    let f = z.by_name(name).ok()?;
    let mut s = String::new();
    f.take(max).read_to_string(&mut s).ok()?;
    Some(s)
}

/// Olha o início do arquivo e, sendo ZIP, os nomes das entradas e o `[Content_Types].xml`.
pub fn sniff(path: &Path) -> Res<Sniff> {
    let mut f = std::fs::File::open(path).map_err(|e| Error::Invalid(format!("não foi possível abrir '{}': {e}", path.display())))?;
    if !f.metadata().map(|m| m.is_file()).unwrap_or(false) {
        return Err(Error::Invalid(format!("'{}' não é um arquivo", path.display())));
    }
    let mut head = [0u8; route::PDF_HEADER_WINDOW];
    let n = f.read(&mut head).map_err(|e| Error::Invalid(format!("falha ao ler '{}': {e}", path.display())))?;
    let head = &head[..n];
    if !head.starts_with(b"PK") {
        return Ok(route::sniff_head(head));
    }
    let Ok(mut z) = zip::ZipArchive::new(f) else { return Ok(Sniff::Zip(ZipKind::Broken)) };
    let names: Vec<String> = z.file_names().take(100_000).map(str::to_string).collect();
    let ct = zip_text(&mut z, "[Content_Types].xml", 1 << 20);
    let mime = zip_text(&mut z, "mimetype", 256);
    Ok(Sniff::Zip(route::classify_zip(&names, ct.as_deref(), mime.as_deref())))
}

/// Escolhe o motor do arquivo.
pub fn engine_for(path: &Path) -> Res<Engine> {
    let ext = route::ext_of(&path.to_string_lossy());
    let sniff = sniff(path)?;
    route::route(&ext, sniff)
}

/// Abre o arquivo com o motor.
pub fn open(engine: Engine, path: &Path) -> Res<Box<dyn Render>> {
    Ok(match engine {
        Engine::Word => Box::new(Word::open(path)?),
        Engine::Grid => Box::new(Grid::open(path)?),
        Engine::Deck => Box::new(Deck::open(path)?),
        Engine::Pdf => Box::new(Pdf::open(path)?),
    })
}
