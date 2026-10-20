//! Erros do oráculo com códigos estáveis `TYRUS-E5xxx` (R14). O bloco E4xxx já é
//! do `FormattingError` do protótipo; códigos nunca são reaproveitados.
use std::path::PathBuf;

use thiserror::Error;

use crate::tuple::{FieldDiff, NodeRole};

#[derive(Debug, Error)]
pub enum OracleError {
    #[error("a tupla do Node ({role}) difere do oracle.lock: {}", join_diffs(.diffs))]
    TupleMismatch {
        role: NodeRole,
        diffs: Vec<FieldDiff>,
    },

    #[error(
        "a variável de ambiente {env_var} não está definida; aponte para o binário exato do Node"
    )]
    NodeBinaryNotConfigured { env_var: &'static str },

    #[error("não foi possível sondar o Node em {}: {reason}", .path.display())]
    NodeProbeFailed { path: PathBuf, reason: String },

    #[error("a saída da sonda do Node é inválida: {reason}")]
    NodeProbeOutputInvalid { reason: String },

    #[error("oracle.lock inválido: {reason}")]
    LockInvalid { reason: String },

    #[error("normalizador `{id}` fora do registro fechado")]
    UnknownNormalizer { id: String },

    #[error("premissa `{id}` fora do registro fechado")]
    UnknownAssumption { id: String },

    #[error("fração inválida `{value}`: deve ser finita e estar em [0, 1]")]
    InvalidShare { value: String },

    #[error("campo `{field}` do veredito inválido: {reason}")]
    InvalidVerdictField { field: &'static str, reason: String },
}

impl OracleError {
    /// Código estável (R14): nunca muda de sentido e nunca é reaproveitado.
    #[must_use]
    pub const fn stable_code(&self) -> &'static str {
        match self {
            Self::TupleMismatch { .. } => "TYRUS-E5001",
            Self::NodeBinaryNotConfigured { .. } => "TYRUS-E5002",
            Self::NodeProbeFailed { .. } => "TYRUS-E5003",
            Self::NodeProbeOutputInvalid { .. } => "TYRUS-E5004",
            Self::LockInvalid { .. } => "TYRUS-E5005",
            Self::UnknownNormalizer { .. } => "TYRUS-E5006",
            Self::UnknownAssumption { .. } => "TYRUS-E5007",
            Self::InvalidShare { .. } => "TYRUS-E5008",
            Self::InvalidVerdictField { .. } => "TYRUS-E5009",
        }
    }
}

fn join_diffs(diffs: &[FieldDiff]) -> String {
    diffs
        .iter()
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join("; ")
}
