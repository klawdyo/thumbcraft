//! Stand-in for the `wordcraft-pdf` crate of WordCraft.
//!
//! It keeps the public API that `wordcraft-engine` calls (`PdfOptions`, `export`, `export_layout`, `PdfError`) so the
//! engine compiles unchanged, but every export fails: thumbcraft never writes PDF. The real crate pulls in `krilla`,
//! which duplicates `skrifa` and its font crates in the binary. The swap is one line in WordCraft's root
//! `Cargo.toml` (`patches/wordcraft/`), registered in `docs/UPSTREAM-CHANGES.md`.

use wordcraft_doc::Document;
use wordcraft_layout::DocLayout;

/// Same fields and defaults as the real `PdfOptions`; none of them has any effect here.
#[derive(Clone, Debug)]
pub struct PdfOptions {
    pub pages: Option<Vec<usize>>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub tagged: bool,
    pub include_markup: bool,
    pub comments: bool,
    pub compress: bool,
    pub paper: Option<(f32, f32)>,
}

impl Default for PdfOptions {
    fn default() -> Self {
        PdfOptions { pages: None, title: None, author: None, tagged: true, include_markup: false, comments: false, compress: true, paper: None }
    }
}

/// Always `Unavailable` in this stand-in.
#[derive(Debug, PartialEq, Eq)]
pub enum PdfError {
    Unavailable,
}

impl std::fmt::Display for PdfError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("PDF export is not available in thumbcraft")
    }
}

impl std::error::Error for PdfError {}

/// Always fails with [`PdfError::Unavailable`].
pub fn export(_doc: &Document, _opts: &PdfOptions) -> Result<Vec<u8>, PdfError> {
    Err(PdfError::Unavailable)
}

/// Always fails with [`PdfError::Unavailable`].
pub fn export_layout(_doc: &Document, _lay: &DocLayout, _opts: &PdfOptions) -> Result<Vec<u8>, PdfError> {
    Err(PdfError::Unavailable)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn export_always_fails_with_a_clear_message() {
        let doc = Document::default();
        let err = export(&doc, &PdfOptions::default()).unwrap_err();
        assert_eq!(err, PdfError::Unavailable);
        assert_eq!(err.to_string(), "PDF export is not available in thumbcraft");
    }

    #[test]
    fn default_options_match_the_real_crate() {
        let o = PdfOptions::default();
        assert!(o.tagged && o.compress);
        assert!(!o.include_markup && !o.comments);
        assert!(o.pages.is_none() && o.paper.is_none());
    }
}
