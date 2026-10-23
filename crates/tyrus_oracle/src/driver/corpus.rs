//! Listagem das sondas de um corpus e flags do Node por sonda (lógica do `run_probes.py`).
use std::path::{Path, PathBuf};

use crate::error::OracleError;

const TRANSFORM_PREFIXES: [&str; 1] = ["C13_"];
const STRIP_FLAG: &str = "--experimental-strip-types";
const TRANSFORM_FLAG: &str = "--experimental-transform-types";

/// Enum precisa de `transform-types`; o modo só-strip rejeita a sintaxe.
#[must_use]
pub fn node_flag(probe_id: &str) -> &'static str {
    if TRANSFORM_PREFIXES
        .iter()
        .any(|prefix| probe_id.starts_with(prefix))
    {
        TRANSFORM_FLAG
    } else {
        STRIP_FLAG
    }
}

#[must_use]
pub fn default_probes_dir(corpus: &str) -> PathBuf {
    Path::new("oracle/corpus").join(corpus).join("probes")
}

/// Ids ordenados (ordem de bytes) das sondas `.ts` do diretório, filtrados por prefixo.
pub fn list_probes(dir: &Path, filter: Option<&str>) -> Result<Vec<String>, OracleError> {
    let unavailable = |reason: String| OracleError::CorpusUnavailable {
        path: dir.to_path_buf(),
        reason,
    };
    let entries = std::fs::read_dir(dir).map_err(|e| unavailable(e.to_string()))?;
    let mut ids = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| unavailable(e.to_string()))?.path();
        let stem = path
            .file_name()
            .and_then(|name| name.to_str())
            .and_then(|name| name.strip_suffix(".ts"));
        if let Some(id) = stem {
            if filter.is_none_or(|prefix| id.starts_with(prefix)) {
                ids.push(id.to_owned());
            }
        }
    }
    ids.sort();
    if ids.is_empty() {
        return Err(unavailable("nenhuma sonda `.ts` encontrada".to_owned()));
    }
    Ok(ids)
}
