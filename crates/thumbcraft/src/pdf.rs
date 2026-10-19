//! Implementação de `Render` com o Hayro, pelo `craft-pdf` (PDF).
//!
//! O `craft-pdf` impõe os limites de entrada (512 MiB), de decodificação de streams e de saída (8192 px de lado
//! e 64 megapixels). Este módulo só traduz o documento e os erros para a porta `Render`.

use std::path::Path;
use std::sync::Arc;

use craft_pdf::raster::{self, Options, RenderCache};
use craft_pdf::syntax::Pdf as Document;
use craft_pdf::{OpenError, OpenOptions, interpret::InterpreterSettings};
use thumbcraft_core::{Error, Raster, Render, Res};

/// Destino dos avisos do leitor (conteúdo omitido, imagem que não decodificou). Os testes usam um coletor.
pub type WarningSink = Arc<dyn Fn(&str) + Send + Sync>;

/// Aviso padrão: uma linha em português na stderr (a stdout fica só para a imagem).
fn stderr_sink() -> WarningSink {
    Arc::new(|message: &str| eprintln!("thumbcraft: aviso: {message}"))
}

/// Documento PDF aberto, pronto para renderizar páginas em qualquer escala.
pub struct Pdf {
    doc: Document,
    sink: WarningSink,
}

impl Pdf {
    /// Abre um PDF que já está em memória, com os avisos indo para a stderr.
    pub fn from_bytes(bytes: &[u8]) -> Res<Pdf> {
        Pdf::from_bytes_with_sink(bytes, stderr_sink())
    }

    /// Abre um PDF que já está em memória, mandando os avisos para `sink`.
    pub fn from_bytes_with_sink(bytes: &[u8], sink: WarningSink) -> Res<Pdf> {
        let doc = craft_pdf::open_slice_with_options(bytes, None, &OpenOptions::default()).map_err(open_error)?;
        Pdf::from_document(doc, sink)
    }

    /// Abre um arquivo PDF, sem copiar o conteúdo, com os avisos indo para a stderr.
    pub fn open(path: &Path) -> Res<Pdf> {
        let options = OpenOptions::default();
        let size = std::fs::metadata(path).map_err(|e| Error::Invalid(format!("não foi possível abrir '{}': {e}", path.display())))?.len();
        craft_pdf::check_input_size_with(usize::try_from(size).unwrap_or(usize::MAX), &options).map_err(open_error)?;
        let bytes = std::fs::read(path).map_err(|e| Error::Invalid(format!("falha ao ler '{}': {e}", path.display())))?;
        let doc = craft_pdf::open_with_options(Arc::new(bytes), None, &options).map_err(open_error)?;
        Pdf::from_document(doc, stderr_sink())
    }

    /// Recusa documentos sem páginas e guarda o destino dos avisos.
    fn from_document(doc: Document, sink: WarningSink) -> Res<Pdf> {
        if doc.pages().is_empty() {
            return Err(Error::Invalid("o PDF não tem páginas".into()));
        }
        Ok(Pdf { doc, sink })
    }
}

/// Traduz os erros de abertura do `craft-pdf` em erros do thumbcraft (todos código 4).
fn open_error(error: OpenError) -> Error {
    match error {
        OpenError::InputLimit { limit } => Error::Invalid(format!("o PDF passa do limite de {limit} bytes")),
        OpenError::Allocation => Error::Invalid("sem memória para carregar o PDF".into()),
        OpenError::Load(inner) if format!("{inner:?}").contains("PasswordProtected") => {
            Error::Invalid("o PDF é protegido por senha de abertura; não é possível gerar a miniatura".into())
        }
        OpenError::Load(inner) => Error::Invalid(format!("não foi possível abrir como PDF: {inner:?}")),
        OpenError::BackendPanic => Error::Invalid("o leitor de PDF falhou ao abrir o arquivo".into()),
    }
}

/// Traduz os erros de renderização do `craft-pdf` em erros do thumbcraft.
fn raster_error(error: raster::Error) -> Error {
    match error {
        raster::Error::OutputLimit => Error::Invalid(format!(
            "a página passa do limite de tamanho da imagem ({} px de lado ou {} megapixels)",
            raster::MAX_SIDE,
            raster::MAX_PIXELS / 1_000_000
        )),
        raster::Error::InvalidPage => Error::Invalid("a página do PDF tem tamanho inválido".into()),
        raster::Error::Allocation => Error::Invalid("sem memória para renderizar a página do PDF".into()),
        raster::Error::BackendPanic => Error::Invalid("o renderizador de PDF falhou nesta página".into()),
        raster::Error::InvalidOptions | raster::Error::BufferLength | raster::Error::Cancelled => Error::Internal(format!("erro interno ao renderizar o PDF: {error}")),
    }
}

impl Render for Pdf {
    fn unit(&self) -> &'static str {
        "página"
    }

    fn pages(&self) -> usize {
        self.doc.pages().len()
    }

    fn size(&mut self, page: usize) -> Res<(f64, f64)> {
        let pages = self.doc.pages();
        let page = pages.get(page).ok_or_else(|| Error::Invalid(format!("a página {} não existe", page + 1)))?;
        let (width, height) = page.render_dimensions();
        Ok((f64::from(width), f64::from(height)))
    }

    fn render(&mut self, page: usize, scale: f64) -> Res<Raster> {
        let pages = self.doc.pages();
        let selected = pages.get(page).ok_or_else(|| Error::Invalid(format!("a página {} não existe", page + 1)))?;
        // Uma página de PDF não é transparente: fundo branco opaco.
        let options = Options { scale: [scale as f32; 2], background: [255, 255, 255, 255], ..Options::default() };
        // O buffer é nosso (sem a cópia de `Raster::bytes`): numa página grande são até 256 MB, e a cópia dobrava o pico.
        let [width, height] = raster::dimensions(selected, &options).map_err(raster_error)?;
        let len = usize::try_from(u64::from(width) * u64::from(height) * 4).map_err(|_| raster_error(raster::Error::OutputLimit))?;
        let mut px = Vec::new();
        px.try_reserve_exact(len).map_err(|_| raster_error(raster::Error::Allocation))?;
        px.resize(len, 0u8);
        let diagnostics = raster::render_into(selected, &RenderCache::new(), &InterpreterSettings::default(), &options, &mut px).map_err(raster_error)?;
        if diagnostics.image_decode_failed {
            (self.sink)("uma imagem do PDF não pôde ser decodificada; a miniatura pode estar incompleta");
        }
        if diagnostics.content_truncated {
            (self.sink)("parte do conteúdo da página foi omitida pelos limites do leitor de PDF; a miniatura pode estar incompleta");
        }
        Ok(Raster { w: width, h: height, px })
    }
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use thumbcraft_core::Render;

    use super::*;

    /// Monta um PDF mínimo e válido: uma página por tamanho em pontos, todas com o mesmo conteúdo;
    /// `page_entries` vai dentro de cada página e `extra_objects` são objetos a mais, numerados depois das páginas.
    fn minimal_pdf(pages: &[(u32, u32)], content: &str, page_entries: &str, extra_objects: &[String]) -> Vec<u8> {
        let mut objects: Vec<String> = vec!["<< /Type /Catalog /Pages 2 0 R >>".into()];
        let kids: Vec<String> = (0..pages.len()).map(|i| format!("{} 0 R", 3 + 2 * i)).collect();
        objects.push(format!("<< /Type /Pages /Kids [{}] /Count {} >>", kids.join(" "), pages.len()));
        for (i, (w, h)) in pages.iter().enumerate() {
            objects.push(format!("<< /Type /Page /Parent 2 0 R /MediaBox [0 0 {w} {h}] /Contents {} 0 R {page_entries} >>", 4 + 2 * i));
            objects.push(format!("<< /Length {} >>\nstream\n{content}\nendstream", content.len()));
        }
        objects.extend(extra_objects.iter().cloned());
        let mut out = b"%PDF-1.7\n".to_vec();
        let mut offsets = Vec::new();
        for (i, body) in objects.iter().enumerate() {
            offsets.push(out.len());
            out.extend_from_slice(format!("{} 0 obj\n{body}\nendobj\n", i + 1).as_bytes());
        }
        let xref = out.len();
        out.extend_from_slice(format!("xref\n0 {}\n0000000000 65535 f \n", objects.len() + 1).as_bytes());
        for offset in offsets {
            out.extend_from_slice(format!("{offset:010} 00000 n \n").as_bytes());
        }
        out.extend_from_slice(format!("trailer\n<< /Size {} /Root 1 0 R >>\nstartxref\n{xref}\n%%EOF\n", objects.len() + 1).as_bytes());
        out
    }

    fn open(bytes: &[u8]) -> Pdf {
        Pdf::from_bytes(bytes).expect("o PDF de teste deveria abrir")
    }

    #[test]
    fn reports_page_count_and_sizes() {
        let mut pdf = open(&minimal_pdf(&[(200, 100), (300, 400)], "", "", &[]));
        assert_eq!(pdf.pages(), 2);
        assert_eq!(pdf.size(0).unwrap(), (200.0, 100.0));
        assert_eq!(pdf.size(1).unwrap(), (300.0, 400.0));
        assert_eq!(pdf.unit(), "página");
    }

    #[test]
    fn rotated_page_swaps_the_size() {
        let mut pdf = open(&minimal_pdf(&[(200, 100)], "", "/Rotate 90", &[]));
        assert_eq!(pdf.size(0).unwrap(), (100.0, 200.0));
    }

    #[test]
    fn renders_content_over_an_opaque_white_background() {
        // Quadrado preto no canto inferior esquerdo (a origem do PDF é embaixo), 50 x 25 pt.
        let mut pdf = open(&minimal_pdf(&[(100, 50)], "0 g 0 0 50 25 re f", "", &[]));
        let image = pdf.render(0, 2.0).unwrap();
        assert_eq!((image.w, image.h), (200, 100));
        assert!(image.opaque(), "uma página de PDF não é transparente");
        assert_eq!(image.at(10, 90), [0, 0, 0, 255], "dentro do quadrado, embaixo à esquerda");
        assert_eq!(image.at(150, 10), [255, 255, 255, 255], "fora do quadrado, em cima à direita");
    }

    #[test]
    fn scale_sets_the_raster_size() {
        let mut pdf = open(&minimal_pdf(&[(200, 100)], "", "", &[]));
        let image = pdf.render(0, 1.5).unwrap();
        assert_eq!((image.w, image.h), (300, 150));
        image.check().unwrap();
    }

    #[test]
    fn missing_page_is_error_4() {
        let mut pdf = open(&minimal_pdf(&[(100, 100)], "", "", &[]));
        assert_eq!(pdf.size(3).unwrap_err().code(), 4);
        assert_eq!(pdf.render(3, 1.0).unwrap_err().code(), 4);
    }

    #[test]
    fn document_without_pages_is_error_4() {
        let error = Pdf::from_bytes(&minimal_pdf(&[], "", "", &[])).err().expect("deveria recusar");
        assert_eq!(error.code(), 4);
    }

    #[test]
    fn invalid_data_is_error_4() {
        for bytes in [&b""[..], b"isto nao e um pdf", b"%PDF-1.7\n", b"%PDF-1.7\n1 0 obj\n<<"] {
            let error = Pdf::from_bytes(bytes).err().unwrap_or_else(|| panic!("deveria recusar {bytes:?}"));
            assert_eq!(error.code(), 4, "{bytes:?}");
        }
    }

    #[test]
    fn scale_over_the_output_limit_is_error_4() {
        let mut pdf = open(&minimal_pdf(&[(1000, 1000)], "", "", &[]));
        let error = pdf.render(0, 10.0).unwrap_err();
        assert_eq!(error.code(), 4);
        assert!(error.msg().contains("limite"), "{}", error.msg());
    }

    #[test]
    fn undecodable_image_is_reported_through_the_warning_sink() {
        // A imagem é um JPEG inválido: o Hayro desenha o resto da página e avisa.
        let image = "<< /Type /XObject /Subtype /Image /Width 8 /Height 8 /ColorSpace /DeviceRGB /BitsPerComponent 8 /Filter /DCTDecode /Length 10 >>\nstream\n0123456789\nendstream".to_string();
        let bytes = minimal_pdf(&[(100, 100)], "q 50 0 0 50 0 0 cm /Im0 Do Q", "/Resources << /XObject << /Im0 5 0 R >> >>", &[image]);
        let seen = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = {
            let seen = Arc::clone(&seen);
            Arc::new(move |message: &str| seen.lock().unwrap().push(message.to_string())) as WarningSink
        };
        let mut pdf = Pdf::from_bytes_with_sink(&bytes, sink).unwrap();
        pdf.render(0, 1.0).unwrap();
        let messages = seen.lock().unwrap();
        assert_eq!(messages.len(), 1, "{messages:?}");
        assert!(messages[0].contains("imagem"), "{}", messages[0]);
    }

    #[test]
    fn a_clean_page_reports_no_warnings() {
        let seen = Arc::new(Mutex::new(Vec::<String>::new()));
        let sink = {
            let seen = Arc::clone(&seen);
            Arc::new(move |message: &str| seen.lock().unwrap().push(message.to_string())) as WarningSink
        };
        let mut pdf = Pdf::from_bytes_with_sink(&minimal_pdf(&[(100, 100)], "0 g 0 0 10 10 re f", "", &[]), sink).unwrap();
        pdf.render(0, 1.0).unwrap();
        assert!(seen.lock().unwrap().is_empty());
    }

    #[test]
    fn has_no_native_mode() {
        let mut pdf = open(&minimal_pdf(&[(100, 100)], "", "", &[]));
        assert_eq!(pdf.native_png(0, 1.0).unwrap_err().code(), 5);
    }
}
