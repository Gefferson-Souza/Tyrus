//! Erros do arquivo do fecho com códigos estáveis `TYRUS-E81xx` (R14).
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ArchiveError {
    #[error("hash divergente em `{subject}`: {reason}")]
    HashMismatch { subject: String, reason: String },

    #[error("versão de formato {found} desconhecida (suportada: {supported})")]
    UnknownFormatVersion { found: u32, supported: u32 },

    #[error("arquivo do fecho malformado: {reason}")]
    Malformed { reason: String },

    #[error("entrada `{path}` inválida: {reason}")]
    InvalidEntry { path: String, reason: String },

    #[error("falha de E/S: {0}")]
    Io(#[from] std::io::Error),
}

impl ArchiveError {
    /// Código estável (R14): nunca muda de sentido e nunca é reaproveitado.
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::HashMismatch { .. } => "TYRUS-E8101",
            Self::UnknownFormatVersion { .. } => "TYRUS-E8102",
            Self::Malformed { .. } => "TYRUS-E8103",
            Self::InvalidEntry { .. } => "TYRUS-E8104",
            Self::Io(_) => "TYRUS-E8105",
        }
    }
}

pub(crate) fn malformed(reason: &str) -> ArchiveError {
    ArchiveError::Malformed {
        reason: reason.to_owned(),
    }
}
