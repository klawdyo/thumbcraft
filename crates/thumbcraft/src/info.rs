//! Texto de `--version` e `--formats`.

use thumbcraft_core::route::{DECK_EXTS, GRID_EXTS, WORD_EXTS};

/// Versão do thumbcraft e dos motores embutidos.
pub fn version() -> String {
    let patch = env!("TC_GRID_PATCH");
    let patch = if patch == "nenhum" { "sem patch".to_string() } else { format!("com patches/gridcraft (hash {})", &patch[..patch.len().min(12)]) };
    format!(
        "thumbcraft {}\n\
         wordcraft {} commit {} ({})\n\
         gridcraft {} commit {} ({}), {}\n\
         deckcraft {} commit {} ({})\n\
         craft-libs revisão {} (krilla e craft-pdf)\n\
         compilado com {}\n",
        env!("CARGO_PKG_VERSION"),
        env!("TC_WORD_VER"),
        env!("TC_WORD_SHA"),
        env!("TC_WORD_DATE"),
        env!("TC_GRID_VER"),
        env!("TC_GRID_SHA"),
        env!("TC_GRID_DATE"),
        patch,
        env!("TC_DECK_VER"),
        env!("TC_DECK_SHA"),
        env!("TC_DECK_DATE"),
        env!("TC_CRAFT_LIBS_REV"),
        env!("TC_RUSTC"),
    )
}

/// Extensões suportadas por motor, e o que não é.
pub fn formats() -> String {
    format!(
        "word (WordCraft): {}\n\
         grid (GridCraft): {} (xls e ods parciais)\n\
         deck (DeckCraft): {}\n\
         não suportados: ppt e odp (erro 3), pdf (reservado, erro 3)\n",
        WORD_EXTS.join(" "),
        GRID_EXTS.join(" "),
        DECK_EXTS.join(" "),
    )
}
