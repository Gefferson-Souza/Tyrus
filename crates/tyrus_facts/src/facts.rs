//! `facts.json`: inventário de arquivos com sha256, versão do TypeScript, hash do
//! tsconfig e opções de emit. O esquema é `schemas/facts.schema.json`.
use std::path::Path;

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use crate::document::SchemaVersion;
use crate::error::FactsError;
use crate::file_fact::{ensure_unique, FileFact};
use crate::verify::verify_entries;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TypescriptInfo {
    pub version: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Facts {
    pub schema_version: SchemaVersion,
    pub typescript: TypescriptInfo,
    pub tsconfig: FileFact,
    pub emit_options: Map<String, Value>,
    pub sources: Vec<FileFact>,
    pub outputs: Vec<FileFact>,
}

impl Facts {
    pub(crate) fn validate(&self) -> Result<(), FactsError> {
        ensure_unique(self.sources.iter().map(FileFact::path), "sources")?;
        ensure_unique(self.outputs.iter().map(FileFact::path), "outputs")?;
        if is_dotted_triple(&self.typescript.version) {
            Ok(())
        } else {
            Err(FactsError::invalid(format!(
                "versão do TypeScript fora de `x.y.z`: `{}`",
                self.typescript.version
            )))
        }
    }

    /// Recalcula os hashes: tsconfig e fontes relativos a `project_root`, saídas a `out_dir`.
    ///
    /// # Errors
    /// `E7101` na primeira divergência (inclui arquivo ausente); `E7104` se a leitura falhar.
    pub fn verify(&self, project_root: &Path, out_dir: &Path) -> Result<(), FactsError> {
        verify_entries(project_root, std::slice::from_ref(&self.tsconfig))?;
        verify_entries(project_root, &self.sources)?;
        verify_entries(out_dir, &self.outputs)
    }
}

fn is_dotted_triple(text: &str) -> bool {
    let parts: Vec<&str> = text.split('.').collect();
    parts.len() == 3
        && parts
            .iter()
            .all(|part| !part.is_empty() && part.bytes().all(|b| b.is_ascii_digit()))
}
