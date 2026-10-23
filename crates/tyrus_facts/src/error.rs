//! Erros dos fatos com códigos estáveis `TYRUS-E71xx` (R14). Este crate não depende
//! de `tyrus_diagnostics` (legado); o bloco E7 é compartilhado por tabela única.
use std::path::PathBuf;

use thiserror::Error;

use crate::sha256::Sha256Hex;

#[derive(Debug, Error)]
pub enum FactsError {
    #[error("hash divergente em {path}: esperado {expected}, obtido {}", actual_text(.actual.as_ref()))]
    HashMismatch {
        path: String,
        expected: Sha256Hex,
        actual: Option<Sha256Hex>,
    },

    #[error("versão de esquema desconhecida ({found}); suportada: {supported}")]
    UnknownSchemaVersion { found: String, supported: u64 },

    #[error("documento inválido: {reason}")]
    InvalidDocument { reason: String },

    #[error("não foi possível ler {}: {reason}", .path.display())]
    Io { path: PathBuf, reason: String },
}

impl FactsError {
    /// Código estável (R14): nunca muda de sentido e nunca é reaproveitado.
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::HashMismatch { .. } => "TYRUS-E7101",
            Self::UnknownSchemaVersion { .. } => "TYRUS-E7102",
            Self::InvalidDocument { .. } => "TYRUS-E7103",
            Self::Io { .. } => "TYRUS-E7104",
        }
    }

    pub(crate) fn invalid(reason: impl Into<String>) -> Self {
        Self::InvalidDocument {
            reason: reason.into(),
        }
    }
}

fn actual_text(actual: Option<&Sha256Hex>) -> String {
    actual.map_or_else(|| "ausente".to_owned(), ToString::to_string)
}
