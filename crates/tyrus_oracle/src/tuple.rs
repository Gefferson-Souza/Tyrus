//! Tupla exata de um binário do Node: o pin é por tupla, nunca só por versão (D14).
use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

pub const ENV_NODE_REFERENCE: &str = "TYRUS_NODE_REFERENCE";
pub const ENV_NODE_SEED: &str = "TYRUS_NODE_SEED";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeRole {
    Reference,
    Seed,
}

impl NodeRole {
    #[must_use]
    pub const fn env_var(self) -> &'static str {
        match self {
            Self::Reference => ENV_NODE_REFERENCE,
            Self::Seed => ENV_NODE_SEED,
        }
    }

    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Reference => "reference",
            Self::Seed => "seed",
        }
    }
}

impl fmt::Display for NodeRole {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for NodeRole {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "reference" => Ok(Self::Reference),
            "seed" => Ok(Self::Seed),
            other => Err(format!(
                "papel desconhecido `{other}` (use `reference` ou `seed`)"
            )),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct NodeTuple {
    pub node: String,
    pub v8: String,
    pub icu: String,
    pub unicode: String,
    pub tz: String,
    pub cldr: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldDiff {
    pub field: &'static str,
    pub expected: String,
    pub actual: String,
}

impl fmt::Display for FieldDiff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}: esperado `{}`, obtido `{}`",
            self.field, self.expected, self.actual
        )
    }
}

impl NodeTuple {
    /// Diferenças de `self` (esperada) para `actual` (medida), na ordem dos campos.
    #[must_use]
    pub fn diff(&self, actual: &Self) -> Vec<FieldDiff> {
        let pairs = [
            ("node", &self.node, &actual.node),
            ("v8", &self.v8, &actual.v8),
            ("icu", &self.icu, &actual.icu),
            ("unicode", &self.unicode, &actual.unicode),
            ("tz", &self.tz, &actual.tz),
            ("cldr", &self.cldr, &actual.cldr),
        ];
        pairs
            .into_iter()
            .filter(|(_, expected, got)| expected != got)
            .map(|(field, expected, got)| FieldDiff {
                field,
                expected: expected.clone(),
                actual: got.clone(),
            })
            .collect()
    }
}
