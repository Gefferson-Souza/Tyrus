//! Registros fechados do oráculo: normalizadores (verdes, com catraca) e premissas
//! conhecidas (nunca verdes). Id fora do registro é erro; ampliar o registro é um
//! evento com gate (D14), não uma edição de teste.
use serde::{Deserialize, Serialize};

use crate::error::OracleError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Normalizer {
    /// Quadros de `Error.stack` (nomes de arquivo e posições do emit).
    Stack,
}

impl Normalizer {
    pub const ALL: [Self; 1] = [Self::Stack];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Stack => "N-STACK",
        }
    }

    pub fn from_id(id: &str) -> Result<Self, OracleError> {
        Self::ALL
            .into_iter()
            .find(|normalizer| normalizer.id() == id)
            .ok_or_else(|| OracleError::UnknownNormalizer { id: id.to_owned() })
    }
}

impl TryFrom<String> for Normalizer {
    type Error = OracleError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::from_id(&id)
    }
}

impl From<Normalizer> for String {
    fn from(normalizer: Normalizer) -> Self {
        normalizer.id().to_owned()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub enum Assumption {
    /// `Function.prototype.toString` nativo capturado antes do stub (I3).
    NativeToString,
}

impl Assumption {
    pub const ALL: [Self; 1] = [Self::NativeToString];

    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::NativeToString => "A-NATIVE-TOSTRING",
        }
    }

    pub fn from_id(id: &str) -> Result<Self, OracleError> {
        Self::ALL
            .into_iter()
            .find(|assumption| assumption.id() == id)
            .ok_or_else(|| OracleError::UnknownAssumption { id: id.to_owned() })
    }
}

impl TryFrom<String> for Assumption {
    type Error = OracleError;

    fn try_from(id: String) -> Result<Self, Self::Error> {
        Self::from_id(&id)
    }
}

impl From<Assumption> for String {
    fn from(assumption: Assumption) -> Self {
        assumption.id().to_owned()
    }
}

/// Lista não vazia, ordenada e sem repetição de premissas.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(try_from = "Vec<Assumption>", into = "Vec<Assumption>")]
pub struct Assumptions(Vec<Assumption>);

impl Assumptions {
    #[must_use]
    pub fn as_slice(&self) -> &[Assumption] {
        &self.0
    }
}

impl TryFrom<Vec<Assumption>> for Assumptions {
    type Error = OracleError;

    fn try_from(mut items: Vec<Assumption>) -> Result<Self, Self::Error> {
        if items.is_empty() {
            return Err(OracleError::InvalidVerdictField {
                field: "assumptions",
                reason: "a lista de premissas não pode ser vazia".to_owned(),
            });
        }
        items.sort();
        items.dedup();
        Ok(Self(items))
    }
}

impl From<Assumptions> for Vec<Assumption> {
    fn from(assumptions: Assumptions) -> Self {
        assumptions.0
    }
}
