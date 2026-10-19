//! Entrada e saída por `-` (stdin e stdout), com leitura e gravação injetáveis para testar sem I/O real.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use thumbcraft_core::{Error, Res};

/// Maior documento aceito pela stdin (256 MiB); acima disso o erro é de entrada inválida (código 4).
pub const MAX_STDIN: u64 = 256 << 20;

/// Lê toda a fonte em memória, recusando entrada vazia ou maior que `max` bytes.
pub fn read_limited(src: &mut impl Read, max: u64) -> Res<Vec<u8>> {
    let mut bytes = Vec::new();
    src.take(max + 1).read_to_end(&mut bytes).map_err(|e| Error::Internal(format!("falha ao ler a stdin: {e}")))?;
    if bytes.is_empty() {
        return Err(Error::Invalid("a stdin está vazia".into()));
    }
    if bytes.len() as u64 > max {
        return Err(Error::Invalid(format!("a entrada da stdin passa do limite de {} MiB", max >> 20)));
    }
    Ok(bytes)
}

/// Grava os bytes no destino e descarrega o buffer; qualquer falha é erro interno (código 5).
pub fn write_all(dst: &mut impl Write, bytes: &[u8]) -> Res<()> {
    dst.write_all(bytes).and_then(|()| dst.flush()).map_err(|e| Error::Internal(format!("não foi possível gravar na stdout: {e}")))
}

/// Cópia temporária dos bytes da stdin, porque os motores abrem por caminho. A extensão vem da dica de nome
/// e some junto com a pasta quando o valor sai de escopo.
pub struct TempInput {
    dir: PathBuf,
    file: PathBuf,
}

impl TempInput {
    pub fn create(bytes: &[u8], name_hint: Option<&str>) -> Res<TempInput> {
        let nanos = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or(0);
        let dir = std::env::temp_dir().join(format!("thumbcraft-{}-{nanos}", std::process::id()));
        let bad = |e: std::io::Error| Error::Internal(format!("não foi possível criar o arquivo temporário: {e}"));
        std::fs::create_dir_all(&dir).map_err(bad)?;
        let ext = name_hint.map(thumbcraft_core::route::ext_of).unwrap_or_default();
        let file = dir.join(if ext.is_empty() { "input".to_string() } else { format!("input.{ext}") });
        let tmp = TempInput { dir, file };
        std::fs::write(&tmp.file, bytes).map_err(bad)?;
        Ok(tmp)
    }

    pub fn path(&self) -> &Path {
        &self.file
    }
}

impl Drop for TempInput {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.dir);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("fonte quebrada"))
        }
    }
    impl Write for Broken {
        fn write(&mut self, _: &[u8]) -> std::io::Result<usize> {
            Err(std::io::Error::new(std::io::ErrorKind::BrokenPipe, "destino fechado"))
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    #[test]
    fn reads_everything_up_to_the_limit() {
        assert_eq!(read_limited(&mut Cursor::new(vec![7u8; 10]), 10).unwrap(), vec![7u8; 10]);
    }

    #[test]
    fn empty_input_is_code_4() {
        assert_eq!(read_limited(&mut Cursor::new(Vec::new()), 10).unwrap_err().code(), 4);
    }

    #[test]
    fn input_over_the_limit_is_code_4() {
        assert_eq!(read_limited(&mut Cursor::new(vec![0u8; 11]), 10).unwrap_err().code(), 4);
    }

    #[test]
    fn read_failure_is_code_5() {
        assert_eq!(read_limited(&mut Broken, 10).unwrap_err().code(), 5);
    }

    #[test]
    fn writes_the_bytes_and_flushes() {
        let mut out = Vec::new();
        write_all(&mut out, b"abc").unwrap();
        assert_eq!(out, b"abc");
    }

    #[test]
    fn write_failure_is_code_5() {
        assert_eq!(write_all(&mut Broken, b"abc").unwrap_err().code(), 5);
    }

    #[test]
    fn temp_input_keeps_the_hinted_extension_and_cleans_up() {
        let (path, dir) = {
            let t = TempInput::create(b"conteudo", Some("Pasta/Relatorio.PPTX")).unwrap();
            assert_eq!(t.path().extension().and_then(|e| e.to_str()), Some("pptx"));
            assert_eq!(std::fs::read(t.path()).unwrap(), b"conteudo");
            (t.path().to_path_buf(), t.dir.clone())
        };
        assert!(!path.exists() && !dir.exists());
    }

    #[test]
    fn temp_input_without_hint_has_no_extension() {
        let t = TempInput::create(b"x", None).unwrap();
        assert_eq!(t.path().extension(), None);
    }
}
