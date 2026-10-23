//! Erros do `record` com códigos estáveis `TYRUS-E505x` (R14); E5001 a E5009 ficam em `OracleError`.
use std::path::PathBuf;

use thiserror::Error;

use crate::error::OracleError;

#[derive(Debug, Error)]
pub enum RecordError {
    #[error(transparent)]
    Oracle(#[from] OracleError),

    #[error("o emit do TypeScript do corpus falhou: {reason}")]
    EmitFailed { reason: String },

    #[error("corpus inválido em {}: {reason}", .path.display())]
    CorpusInvalid { path: PathBuf, reason: String },

    #[error("falha de E/S em {}: {reason}", .path.display())]
    Io { path: PathBuf, reason: String },

    #[error("golden ou diff gravado inválido em {}: {reason}", .path.display())]
    StoredInvalid { path: PathBuf, reason: String },
}

impl RecordError {
    /// Código estável (R14): nunca muda de sentido e nunca é reaproveitado.
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::Oracle(inner) => inner.stable_code(),
            Self::EmitFailed { .. } => "TYRUS-E5050",
            Self::CorpusInvalid { .. } => "TYRUS-E5051",
            Self::Io { .. } => "TYRUS-E5052",
            Self::StoredInvalid { .. } => "TYRUS-E5053",
        }
    }
}

pub(crate) fn io_error(path: &std::path::Path, error: &std::io::Error) -> RecordError {
    RecordError::Io {
        path: path.to_path_buf(),
        reason: error.to_string(),
    }
}
