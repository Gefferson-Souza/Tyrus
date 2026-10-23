//! Hash SHA-256 em hexadecimal minúsculo, validado na construção.
use std::fmt;
use std::io::Read;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::FactsError;

const HEX_LEN: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct Sha256Hex(String);

impl Sha256Hex {
    /// # Errors
    /// `InvalidDocument` quando o texto não tem 64 dígitos hexadecimais minúsculos.
    pub fn parse(text: &str) -> Result<Self, FactsError> {
        let valid =
            text.len() == HEX_LEN && text.bytes().all(|b| matches!(b, b'0'..=b'9' | b'a'..=b'f'));
        if valid {
            Ok(Self(text.to_owned()))
        } else {
            Err(FactsError::invalid(format!("sha256 inválido: `{text}`")))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Sha256Hex {
    type Error = FactsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<Sha256Hex> for String {
    fn from(value: Sha256Hex) -> Self {
        value.0
    }
}

impl fmt::Display for Sha256Hex {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[must_use]
pub fn sha256_hex(bytes: &[u8]) -> Sha256Hex {
    Sha256Hex(hex(&Sha256::digest(bytes)))
}

/// Hash em fluxo: fecho de produção pode ter arquivos grandes.
pub(crate) fn sha256_reader(reader: &mut impl Read) -> std::io::Result<Sha256Hex> {
    let mut hasher = Sha256::new();
    std::io::copy(reader, &mut hasher)?;
    Ok(Sha256Hex(hex(&hasher.finalize())))
}

fn hex(bytes: &[u8]) -> String {
    use std::fmt::Write;
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
}
