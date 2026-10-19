//! Erros do thumbcraft e seus códigos de saída.

use std::fmt;

/// Códigos de saída do processo.
pub const EXIT_OK: u8 = 0;
pub const EXIT_USAGE: u8 = 2;
pub const EXIT_UNSUPPORTED: u8 = 3;
pub const EXIT_INVALID: u8 = 4;
pub const EXIT_INTERNAL: u8 = 5;

/// Erro com categoria; a mensagem é sempre em português e vai para a stderr.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Error {
    /// Uso incorreto da linha de comando (código 2).
    Usage(String),
    /// Formato não suportado (código 3).
    Unsupported(String),
    /// Arquivo inválido ou falha ao abrir/renderizar (código 4).
    Invalid(String),
    /// Erro interno (código 5).
    Internal(String),
}

impl Error {
    /// Código de saída do processo para este erro.
    pub fn code(&self) -> u8 {
        match self {
            Error::Usage(_) => EXIT_USAGE,
            Error::Unsupported(_) => EXIT_UNSUPPORTED,
            Error::Invalid(_) => EXIT_INVALID,
            Error::Internal(_) => EXIT_INTERNAL,
        }
    }

    /// Texto da mensagem, sem prefixo.
    pub fn msg(&self) -> &str {
        match self {
            Error::Usage(m) | Error::Unsupported(m) | Error::Invalid(m) | Error::Internal(m) => m,
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.msg())
    }
}

impl std::error::Error for Error {}

/// Atalho de resultado.
pub type Res<T> = Result<T, Error>;
