//! `oracle.lock`: pin por tupla exata da referência e da semente (D14).
use std::collections::BTreeMap;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::error::OracleError;
use crate::tuple::{NodeRole, NodeTuple};

pub const LOCK_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OracleLock {
    pub schema_version: u32,
    pub reference: NodeTuple,
    pub seed: NodeTuple,
    /// Versões dos crates do hospedeiro; preenchido pelo ADR da plataforma.
    pub host_crates: BTreeMap<String, String>,
}

fn lock_invalid(reason: impl Into<String>) -> OracleError {
    OracleError::LockInvalid {
        reason: reason.into(),
    }
}

fn check_tuple(role: NodeRole, tuple: &NodeTuple) -> Result<(), OracleError> {
    let fields = [
        ("node", &tuple.node),
        ("v8", &tuple.v8),
        ("icu", &tuple.icu),
        ("unicode", &tuple.unicode),
        ("tz", &tuple.tz),
        ("cldr", &tuple.cldr),
    ];
    match fields
        .into_iter()
        .find(|(_, value)| value.trim().is_empty())
    {
        Some((field, _)) => Err(lock_invalid(format!("{role}.{field} vazio"))),
        None => Ok(()),
    }
}

impl OracleLock {
    pub fn parse(text: &str) -> Result<Self, OracleError> {
        let lock: Self = serde_json::from_str(text).map_err(|e| lock_invalid(e.to_string()))?;
        if lock.schema_version != LOCK_SCHEMA_VERSION {
            return Err(lock_invalid(format!(
                "schema_version {} não suportada (esperada {LOCK_SCHEMA_VERSION})",
                lock.schema_version
            )));
        }
        check_tuple(NodeRole::Reference, &lock.reference)?;
        check_tuple(NodeRole::Seed, &lock.seed)?;
        Ok(lock)
    }

    pub fn load(path: &Path) -> Result<Self, OracleError> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| lock_invalid(format!("{}: {e}", path.display())))?;
        Self::parse(&text)
    }

    #[must_use]
    pub const fn tuple(&self, role: NodeRole) -> &NodeTuple {
        match role {
            NodeRole::Reference => &self.reference,
            NodeRole::Seed => &self.seed,
        }
    }
}
