//! Valores textuais validados que acompanham vereditos não verdes.
use serde::{Deserialize, Serialize};

use crate::error::OracleError;

const CODE_MAX_LEN: usize = 64;
const CODE_MIN_LEN: usize = 3;
const DETAIL_MAX_LEN: usize = 128;
const REASON_MAX_LEN: usize = 200;

fn invalid(field: &'static str, reason: &str) -> OracleError {
    OracleError::InvalidVerdictField {
        field,
        reason: reason.to_owned(),
    }
}

/// Código de recusa do build, como `ADDON_NAPI_UNSUPPORTED` ou
/// `UNBOUND_PLATFORM(binding.membro)` (D12).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RefusalCode {
    code: String,
    detail: Option<String>,
}

impl RefusalCode {
    pub fn new(code: &str, detail: Option<&str>) -> Result<Self, OracleError> {
        let first_is_upper = code.chars().next().is_some_and(|c| c.is_ascii_uppercase());
        let body_is_valid = code
            .chars()
            .all(|c| c.is_ascii_uppercase() || c.is_ascii_digit() || c == '_');
        if !(CODE_MIN_LEN..=CODE_MAX_LEN).contains(&code.len()) || !first_is_upper || !body_is_valid
        {
            return Err(invalid(
                "code",
                "use A-Z, 0-9 e `_`, iniciando por letra (3 a 64)",
            ));
        }
        if let Some(text) = detail {
            let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | ':' | '-');
            if text.is_empty() || text.len() > DETAIL_MAX_LEN || !text.chars().all(allowed) {
                return Err(invalid(
                    "code",
                    "detalhe vazio, longo ou com caractere inválido",
                ));
            }
        }
        Ok(Self {
            code: code.to_owned(),
            detail: detail.map(str::to_owned),
        })
    }

    #[must_use]
    pub fn code(&self) -> &str {
        &self.code
    }

    #[must_use]
    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }
}

impl TryFrom<String> for RefusalCode {
    type Error = OracleError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        match text.split_once('(') {
            None => Self::new(&text, None),
            Some((code, rest)) => {
                let detail = rest
                    .strip_suffix(')')
                    .ok_or_else(|| invalid("code", "parêntese de detalhe não fechado"))?;
                Self::new(code, Some(detail))
            }
        }
    }
}

impl From<RefusalCode> for String {
    fn from(refusal: RefusalCode) -> Self {
        match refusal.detail {
            Some(detail) => format!("{}({detail})", refusal.code),
            None => refusal.code,
        }
    }
}

/// Motivo curto de uma sonda ruim (`BAD_PROBE`), em uma linha.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct BadProbeReason(String);

impl BadProbeReason {
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for BadProbeReason {
    type Error = OracleError;

    fn try_from(text: String) -> Result<Self, Self::Error> {
        let printable = text.chars().all(|c| !c.is_control());
        if text.is_empty() || text.chars().count() > REASON_MAX_LEN || !printable {
            return Err(invalid("reason", "use de 1 a 200 caracteres em uma linha"));
        }
        Ok(Self(text))
    }
}

impl From<BadProbeReason> for String {
    fn from(reason: BadProbeReason) -> Self {
        reason.0
    }
}
