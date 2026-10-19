//! Roteamento do arquivo de entrada para um motor, por conteúdo e por extensão.

use crate::error::{Error, Res};

/// Motor que vai renderizar o arquivo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Engine {
    /// WordCraft: documentos.
    Word,
    /// GridCraft: planilhas.
    Grid,
    /// DeckCraft: apresentações.
    Deck,
    /// Ponto de extensão para PDF (ainda sem implementação).
    Pdf,
}

impl Engine {
    /// Nome curto, usado nas listagens e nos relatórios.
    pub fn name(self) -> &'static str {
        match self {
            Engine::Word => "word",
            Engine::Grid => "grid",
            Engine::Deck => "deck",
            Engine::Pdf => "pdf",
        }
    }
}

/// Extensões lidas por cada motor (minúsculas, sem ponto).
pub const WORD_EXTS: &[&str] = &["docx", "doc", "odt", "rtf", "html", "htm", "md", "tex", "txt"];
pub const GRID_EXTS: &[&str] = &["xlsx", "xls", "ods", "csv", "tsv"];
pub const DECK_EXTS: &[&str] = &["pptx", "potx", "ppsx"];

/// Contêiner ZIP, depois de olhar o conteúdo.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ZipKind {
    Word,
    Grid,
    Deck,
    OdfText,
    OdfSheet,
    OdfPres,
    /// ZIP legível, mas de outro tipo.
    Other,
    /// Não deu para ler o índice do ZIP (truncado, corrompido).
    Broken,
}

/// O que o início do arquivo revela.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Sniff {
    Zip(ZipKind),
    /// Contêiner CFB (formatos antigos .doc, .xls, .ppt e pacotes OOXML criptografados).
    Cfb,
    Pdf,
    Rtf,
    /// Sem assinatura conhecida, mas o início parece texto (sem bytes nulos).
    Text,
    Other,
}

/// Classifica um ZIP pelos nomes das entradas, por `[Content_Types].xml` e pelo `mimetype` (ODF).
pub fn classify_zip(names: &[String], content_types: Option<&str>, mimetype: Option<&str>) -> ZipKind {
    if let Some(m) = mimetype {
        let m = m.trim();
        if m.starts_with("application/vnd.oasis.opendocument.text") {
            return ZipKind::OdfText;
        }
        if m.starts_with("application/vnd.oasis.opendocument.spreadsheet") {
            return ZipKind::OdfSheet;
        }
        if m.starts_with("application/vnd.oasis.opendocument.presentation") {
            return ZipKind::OdfPres;
        }
    }
    let has = |p: &str| names.iter().any(|n| n.starts_with(p));
    if has("word/") {
        return ZipKind::Word;
    }
    if has("xl/") {
        return ZipKind::Grid;
    }
    if has("ppt/") {
        return ZipKind::Deck;
    }
    if let Some(ct) = content_types {
        if ct.contains("wordprocessingml") {
            return ZipKind::Word;
        }
        if ct.contains("spreadsheetml") {
            return ZipKind::Grid;
        }
        if ct.contains("presentationml") {
            return ZipKind::Deck;
        }
    }
    ZipKind::Other
}

/// Quantos bytes do início podem preceder o cabeçalho `%PDF-` de um PDF.
pub const PDF_HEADER_WINDOW: usize = 1024;

/// Lê os primeiros bytes de um arquivo que não é ZIP (a leitura do ZIP é feita à parte).
pub fn sniff_head(head: &[u8]) -> Sniff {
    if head.starts_with(b"%PDF") {
        return Sniff::Pdf;
    }
    if head.starts_with(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]) {
        return Sniff::Cfb;
    }
    if head.starts_with(b"{\\rtf") {
        return Sniff::Rtf;
    }
    // A especificação do PDF aceita lixo antes do cabeçalho %PDF-, dentro do primeiro 1 KB.
    if head[..head.len().min(PDF_HEADER_WINDOW)].windows(5).any(|w| w == b"%PDF-") {
        return Sniff::Pdf;
    }
    if !head.is_empty() && !head.contains(&0) {
        return Sniff::Text;
    }
    Sniff::Other
}

/// Mensagem padrão para .ppt e .odp.
pub fn deck_legacy(what: &str) -> Error {
    Error::Unsupported(format!(
        "o formato {what} não é suportado pelo DeckCraft (só pptx, potx e ppsx); converta para .pptx (por exemplo com o LibreOffice: soffice --headless --convert-to pptx) e tente de novo"
    ))
}

/// Extensão minúscula de um nome de arquivo, sem ponto (vazia se não houver).
pub fn ext_of(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    match base.rsplit_once('.') {
        Some((stem, ext)) if !stem.is_empty() => ext.to_ascii_lowercase(),
        _ => String::new(),
    }
}

fn by_ext(ext: &str) -> Res<Engine> {
    if WORD_EXTS.contains(&ext) {
        return Ok(Engine::Word);
    }
    if GRID_EXTS.contains(&ext) {
        return Ok(Engine::Grid);
    }
    if DECK_EXTS.contains(&ext) {
        return Ok(Engine::Deck);
    }
    // Sem assinatura reconhecível (por exemplo, lixo antes do %PDF), vale a extensão: o motor de PDF decide.
    if ext == "pdf" {
        return Ok(Engine::Pdf);
    }
    match ext {
        "ppt" => Err(deck_legacy(".ppt")),
        "odp" => Err(deck_legacy(".odp")),
        "" => Err(Error::Unsupported("não foi possível identificar o formato: o arquivo não tem extensão e o conteúdo não é reconhecido".into())),
        e => Err(Error::Unsupported(format!("formato não suportado: .{e} (veja thumbcraft --formats)"))),
    }
}

/// Escolhe o motor. O conteúdo manda quando é inequívoco (ZIP com `word/`, `xl/` ou `ppt/`,
/// PDF, RTF); caso contrário vale a extensão. Isso tolera extensão errada, mas não adivinha
/// texto puro: um arquivo de texto chamado `.pptx` segue para o DeckCraft e falha ao abrir.
pub fn route(ext: &str, sniff: Sniff) -> Res<Engine> {
    let ext = ext.to_ascii_lowercase();
    match sniff {
        Sniff::Zip(ZipKind::Word | ZipKind::OdfText) => Ok(Engine::Word),
        Sniff::Zip(ZipKind::Grid | ZipKind::OdfSheet) => Ok(Engine::Grid),
        Sniff::Zip(ZipKind::Deck) => Ok(Engine::Deck),
        Sniff::Zip(ZipKind::OdfPres) => Err(deck_legacy(".odp")),
        Sniff::Pdf => Ok(Engine::Pdf),
        Sniff::Rtf => Ok(Engine::Word),
        // Texto com extensão .pdf (extensão errada): vai para o WordCraft, que lê texto puro.
        Sniff::Text if ext == "pdf" => Ok(Engine::Word),
        Sniff::Zip(ZipKind::Other | ZipKind::Broken) | Sniff::Cfb | Sniff::Text | Sniff::Other => by_ext(&ext),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn names(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn lowercase_and_dotless_extension() {
        assert_eq!(ext_of("a/b/Relatorio.DOCX"), "docx");
        assert_eq!(ext_of("sem_ext"), "");
        assert_eq!(ext_of(".gitignore"), "");
        assert_eq!(ext_of("a.b.c.xlsx"), "xlsx");
        assert_eq!(ext_of("pasta.d/arquivo"), "");
    }

    #[test]
    fn by_extension_when_content_is_silent() {
        for e in WORD_EXTS {
            assert_eq!(route(e, Sniff::Other).unwrap(), Engine::Word, "{e}");
        }
        for e in GRID_EXTS {
            assert_eq!(route(e, Sniff::Other).unwrap(), Engine::Grid, "{e}");
        }
        for e in DECK_EXTS {
            assert_eq!(route(e, Sniff::Other).unwrap(), Engine::Deck, "{e}");
        }
        assert_eq!(route("DOCX", Sniff::Other).unwrap(), Engine::Word);
    }

    #[test]
    fn zip_content_beats_wrong_extension() {
        let word = Sniff::Zip(ZipKind::Word);
        assert_eq!(route("pptx", word).unwrap(), Engine::Word);
        assert_eq!(route("ppt", Sniff::Zip(ZipKind::Deck)).unwrap(), Engine::Deck);
        assert_eq!(route("docx", Sniff::Zip(ZipKind::Grid)).unwrap(), Engine::Grid);
        assert_eq!(route("xlsx", Sniff::Zip(ZipKind::Deck)).unwrap(), Engine::Deck);
        assert_eq!(route("", Sniff::Zip(ZipKind::Word)).unwrap(), Engine::Word);
    }

    #[test]
    fn odf_by_mimetype() {
        assert_eq!(route("odt", Sniff::Zip(ZipKind::OdfText)).unwrap(), Engine::Word);
        assert_eq!(route("ods", Sniff::Zip(ZipKind::OdfSheet)).unwrap(), Engine::Grid);
        let e = route("odp", Sniff::Zip(ZipKind::OdfPres)).unwrap_err();
        assert_eq!(e.code(), 3);
        assert!(e.msg().contains("pptx"));
    }

    #[test]
    fn ppt_and_odp_are_error_3_with_suggestion() {
        for e in ["ppt", "odp"] {
            let err = route(e, Sniff::Cfb).unwrap_err();
            assert_eq!(err.code(), 3);
            assert!(err.msg().contains("pptx"), "{err}");
        }
        // Mesmo um .odp que é ZIP não-ODF cai na extensão.
        assert_eq!(route("odp", Sniff::Zip(ZipKind::Other)).unwrap_err().code(), 3);
    }

    #[test]
    fn pdf_routes_to_the_pdf_engine() {
        // Pelo conteúdo, com qualquer extensão (ou nenhuma).
        assert_eq!(route("pdf", Sniff::Pdf).unwrap(), Engine::Pdf);
        assert_eq!(route("txt", Sniff::Pdf).unwrap(), Engine::Pdf);
        assert_eq!(route("", Sniff::Pdf).unwrap(), Engine::Pdf);
        // Pela extensão, quando o conteúdo não diz (por exemplo, lixo antes do cabeçalho %PDF).
        assert_eq!(route("pdf", Sniff::Other).unwrap(), Engine::Pdf);
        assert_eq!(route("PDF", Sniff::Other).unwrap(), Engine::Pdf);
        assert_eq!(route("pdf", Sniff::Cfb).unwrap(), Engine::Pdf);
        // Markdown salvo como .pdf: segue como texto, para o WordCraft.
        assert_eq!(route("pdf", Sniff::Text).unwrap(), Engine::Word);
        assert_eq!(route("txt", Sniff::Text).unwrap(), Engine::Word);
    }

    #[test]
    fn legacy_cfb_follows_extension() {
        assert_eq!(route("doc", Sniff::Cfb).unwrap(), Engine::Word);
        assert_eq!(route("xls", Sniff::Cfb).unwrap(), Engine::Grid);
        // Pacote OOXML criptografado (CFB) com extensão moderna.
        assert_eq!(route("docx", Sniff::Cfb).unwrap(), Engine::Word);
        assert_eq!(route("pptx", Sniff::Cfb).unwrap(), Engine::Deck);
    }

    #[test]
    fn rtf_by_content() {
        assert_eq!(route("txt", Sniff::Rtf).unwrap(), Engine::Word);
        assert_eq!(route("", Sniff::Rtf).unwrap(), Engine::Word);
    }

    #[test]
    fn text_with_pptx_extension_goes_to_deck_and_fails_there() {
        assert_eq!(route("pptx", Sniff::Other).unwrap(), Engine::Deck);
        assert_eq!(route("pptx", Sniff::Zip(ZipKind::Broken)).unwrap(), Engine::Deck);
    }

    #[test]
    fn unknown_is_error_3() {
        assert_eq!(route("xyz", Sniff::Other).unwrap_err().code(), 3);
        assert_eq!(route("", Sniff::Other).unwrap_err().code(), 3);
        assert_eq!(route("exe", Sniff::Zip(ZipKind::Other)).unwrap_err().code(), 3);
    }

    #[test]
    fn classifies_zip() {
        assert_eq!(classify_zip(&names(&["[Content_Types].xml", "word/document.xml"]), None, None), ZipKind::Word);
        assert_eq!(classify_zip(&names(&["[Content_Types].xml", "xl/workbook.xml"]), None, None), ZipKind::Grid);
        assert_eq!(classify_zip(&names(&["[Content_Types].xml", "ppt/presentation.xml"]), None, None), ZipKind::Deck);
        assert_eq!(classify_zip(&names(&["a.txt"]), None, None), ZipKind::Other);
        // Nomes de pastas fora do padrão: vale o [Content_Types].xml.
        let ct = "<Override ContentType=\"application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml\"/>";
        assert_eq!(classify_zip(&names(&["[Content_Types].xml", "outro/x.xml"]), Some(ct), None), ZipKind::Grid);
        assert_eq!(classify_zip(&names(&["mimetype"]), None, Some("application/vnd.oasis.opendocument.text")), ZipKind::OdfText);
        assert_eq!(classify_zip(&names(&["mimetype"]), None, Some("application/vnd.oasis.opendocument.spreadsheet\n")), ZipKind::OdfSheet);
        assert_eq!(classify_zip(&names(&["mimetype"]), None, Some("application/vnd.oasis.opendocument.presentation")), ZipKind::OdfPres);
    }

    #[test]
    fn header_sniff() {
        assert_eq!(sniff_head(b"%PDF-1.7"), Sniff::Pdf);
        assert_eq!(sniff_head(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1, 0, 0]), Sniff::Cfb);
        assert_eq!(sniff_head(b"{\\rtf1\\ansi"), Sniff::Rtf);
        assert_eq!(sniff_head(b"Ola"), Sniff::Text);
        assert_eq!(sniff_head(&[1, 0, 2]), Sniff::Other);
        assert_eq!(sniff_head(b""), Sniff::Other);
    }

    #[test]
    fn pdf_header_may_follow_junk_within_the_first_1024_bytes() {
        // A especificação do PDF aceita lixo antes do cabeçalho %PDF-, dentro do primeiro 1 KB.
        let mut junk = vec![b'x'; 1000];
        junk.extend_from_slice(b"
%PDF-1.7
");
        assert_eq!(sniff_head(&junk), Sniff::Pdf);
        assert_eq!(sniff_head(b"lixo
%PDF-1.4
1 0 obj"), Sniff::Pdf);
        // Depois do primeiro 1 KB não conta: continua sendo texto.
        let mut late = vec![b'x'; 1100];
        late.extend_from_slice(b"
%PDF-1.7
");
        assert_eq!(sniff_head(&late), Sniff::Text);
        // Um arquivo de texto que só cita "PDF" não é PDF.
        assert_eq!(sniff_head(b"o formato PDF e antigo"), Sniff::Text);
    }
}
