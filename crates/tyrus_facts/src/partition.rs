//! `partition.lock`: onde cada unidade roda e o motivo (`Hosted(FLOOR)` no M1).
//! O esquema é `schemas/partition.lock.schema.json`.
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::document::SchemaVersion;
use crate::error::FactsError;
use crate::file_fact::{ensure_strictly_sorted, FileFact, RelativePath};
use crate::sha256::Sha256Hex;
use crate::verify::verify_entries;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Placement {
    Hosted,
    Native,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "RawPartitionUnit")]
pub struct PartitionUnit {
    path: RelativePath,
    sha256: Sha256Hex,
    placement: Placement,
    reason: String,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPartitionUnit {
    path: RelativePath,
    sha256: Sha256Hex,
    placement: Placement,
    reason: String,
}

impl TryFrom<RawPartitionUnit> for PartitionUnit {
    type Error = FactsError;

    fn try_from(raw: RawPartitionUnit) -> Result<Self, Self::Error> {
        validate_reason(&raw.reason)?;
        Ok(Self {
            path: raw.path,
            sha256: raw.sha256,
            placement: raw.placement,
            reason: raw.reason,
        })
    }
}

impl PartitionUnit {
    /// # Errors
    /// `InvalidDocument` para caminho não portável ou motivo fora de `^[A-Z][A-Z0-9_]*$`.
    pub fn new(
        path: &str,
        sha256: Sha256Hex,
        placement: Placement,
        reason: &str,
    ) -> Result<Self, FactsError> {
        validate_reason(reason)?;
        Ok(Self {
            path: RelativePath::parse(path)?,
            sha256,
            placement,
            reason: reason.to_owned(),
        })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    pub const fn placement(&self) -> Placement {
        self.placement
    }

    #[must_use]
    pub fn reason(&self) -> &str {
        &self.reason
    }
}

fn validate_reason(reason: &str) -> Result<(), FactsError> {
    let mut bytes = reason.bytes();
    let head_ok = bytes.next().is_some_and(|b| b.is_ascii_uppercase());
    if head_ok && bytes.all(|b| b.is_ascii_uppercase() || b.is_ascii_digit() || b == b'_') {
        Ok(())
    } else {
        Err(FactsError::invalid(format!(
            "motivo fora de `^[A-Z][A-Z0-9_]*$`: `{reason}`"
        )))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "RawPartitionLock")]
pub struct PartitionLock {
    schema_version: SchemaVersion,
    units: Vec<PartitionUnit>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawPartitionLock {
    #[serde(rename = "schema_version")]
    _version: SchemaVersion,
    units: Vec<PartitionUnit>,
}

impl TryFrom<RawPartitionLock> for PartitionLock {
    type Error = FactsError;

    fn try_from(raw: RawPartitionLock) -> Result<Self, Self::Error> {
        Self::new(raw.units)
    }
}

impl PartitionLock {
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// # Errors
    /// `InvalidDocument` se os caminhos não estiverem em ordem estrita por bytes.
    pub fn new(units: Vec<PartitionUnit>) -> Result<Self, FactsError> {
        ensure_strictly_sorted(units.iter().map(PartitionUnit::path), "partition.lock")?;
        Ok(Self {
            schema_version: SchemaVersion,
            units,
        })
    }

    #[must_use]
    pub fn units(&self) -> &[PartitionUnit] {
        &self.units
    }

    /// # Errors
    /// `E7101` na primeira divergência (inclui arquivo ausente); `E7104` se a leitura falhar.
    pub fn verify(&self, root: &Path) -> Result<(), FactsError> {
        let entries: Result<Vec<FileFact>, FactsError> = self
            .units
            .iter()
            .map(|unit| FileFact::new(unit.path(), unit.sha256.clone()))
            .collect();
        verify_entries(root, &entries?)
    }
}
