//! Erros do fecho com códigos estáveis `TYRUS-E72xx` (R14). Os erros de `tyrus_facts`
//! (`E71xx`) passam intactos, com o código original.
use std::path::{Path, PathBuf};

use thiserror::Error;
use tyrus_facts::FactsError;

#[derive(Debug, Error)]
pub enum ClosureError {
    #[error(transparent)]
    Facts(#[from] FactsError),

    #[error("dependência de desenvolvimento no fecho: {}", .paths.join(", "))]
    DevDependency { paths: Vec<String> },

    #[error("{} arquivo(s) carregado(s) ausente(s) do fecho: {}", .paths.len(), .paths.join(", "))]
    TraceMissing { paths: Vec<String> },

    #[error("rastro inválido na linha {line}: {reason}")]
    InvalidTrace { line: usize, reason: String },

    #[error("npm ci falhou ({status}): {stderr}")]
    Npm { status: String, stderr: String },

    #[error("E/S em {}: {reason}", .path.display())]
    Io { path: PathBuf, reason: String },
}

impl ClosureError {
    /// Código estável (R14): nunca muda de sentido e nunca é reaproveitado.
    #[must_use]
    pub fn stable_code(&self) -> &'static str {
        match self {
            Self::Facts(inner) => inner.stable_code(),
            Self::DevDependency { .. } => "TYRUS-E7201",
            Self::TraceMissing { .. } => "TYRUS-E7202",
            Self::InvalidTrace { .. } => "TYRUS-E7203",
            Self::Npm { .. } => "TYRUS-E7204",
            Self::Io { .. } => "TYRUS-E7205",
        }
    }

    pub(crate) fn io(path: &Path, error: &std::io::Error) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            reason: error.to_string(),
        }
    }

    pub(crate) fn io_reason(path: &Path, reason: impl Into<String>) -> Self {
        Self::Io {
            path: path.to_path_buf(),
            reason: reason.into(),
        }
    }
}
