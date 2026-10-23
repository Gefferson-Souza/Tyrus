//! `closure.lock`: lista ordenada dos arquivos do fecho de produção, com modo
//! normalizado e sha256. O esquema é `schemas/closure.lock.schema.json`.
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::document::SchemaVersion;
use crate::error::FactsError;
use crate::file_fact::{ensure_strictly_sorted, FileFact, RelativePath};
use crate::sha256::Sha256Hex;
use crate::verify::verify_entries;

/// Só dois modos existem no fecho; o resto é normalizado para um deles.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FileMode {
    #[serde(rename = "0644")]
    Regular,
    #[serde(rename = "0755")]
    Executable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ClosureFile {
    path: RelativePath,
    mode: FileMode,
    sha256: Sha256Hex,
}

impl ClosureFile {
    /// # Errors
    /// `InvalidDocument` quando o caminho não é relativo portável.
    pub fn new(path: &str, mode: FileMode, sha256: Sha256Hex) -> Result<Self, FactsError> {
        Ok(Self {
            path: RelativePath::parse(path)?,
            mode,
            sha256,
        })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    pub const fn mode(&self) -> FileMode {
        self.mode
    }

    #[must_use]
    pub fn sha256(&self) -> &Sha256Hex {
        &self.sha256
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields, try_from = "RawClosureLock")]
pub struct ClosureLock {
    schema_version: SchemaVersion,
    files: Vec<ClosureFile>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct RawClosureLock {
    #[serde(rename = "schema_version")]
    _version: SchemaVersion,
    files: Vec<ClosureFile>,
}

impl TryFrom<RawClosureLock> for ClosureLock {
    type Error = FactsError;

    fn try_from(raw: RawClosureLock) -> Result<Self, Self::Error> {
        Self::new(raw.files)
    }
}

impl ClosureLock {
    #[must_use]
    pub const fn schema_version(&self) -> SchemaVersion {
        self.schema_version
    }

    /// # Errors
    /// `InvalidDocument` se os caminhos não estiverem em ordem estrita por bytes.
    pub fn new(files: Vec<ClosureFile>) -> Result<Self, FactsError> {
        ensure_strictly_sorted(files.iter().map(ClosureFile::path), "closure.lock")?;
        Ok(Self {
            schema_version: SchemaVersion,
            files,
        })
    }

    #[must_use]
    pub fn files(&self) -> &[ClosureFile] {
        &self.files
    }

    /// # Errors
    /// `E7101` na primeira divergência (inclui arquivo ausente); `E7104` se a leitura falhar.
    pub fn verify(&self, root: &Path) -> Result<(), FactsError> {
        let entries: Result<Vec<FileFact>, FactsError> = self
            .files
            .iter()
            .map(|file| FileFact::new(file.path(), file.sha256().clone()))
            .collect();
        verify_entries(root, &entries?)
    }
}
