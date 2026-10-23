//! Leitura e escrita dos arquivos gravados, em JSON legível com quebra de linha final.
use std::path::{Path, PathBuf};

use serde::de::DeserializeOwned;
use serde::Serialize;

use super::error::{io_error, RecordError};

pub const GOLDEN_DIR: &str = "golden";
pub const META_FILE: &str = "meta.json";
pub const DIFF_FILE: &str = "rerecord-diff.json";

#[must_use]
pub fn golden_path(corpus: &Path, set: &str, id: &str) -> PathBuf {
    corpus.join(GOLDEN_DIR).join(set).join(format!("{id}.json"))
}

#[must_use]
pub fn meta_path(corpus: &Path) -> PathBuf {
    corpus.join(GOLDEN_DIR).join(META_FILE)
}

#[must_use]
pub fn diff_path(corpus: &Path) -> PathBuf {
    corpus.join(DIFF_FILE)
}

pub fn write_json<T: Serialize>(path: &Path, value: &T) -> Result<(), RecordError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| io_error(parent, &e))?;
    }
    let mut text = serde_json::to_string_pretty(value).map_err(|e| RecordError::StoredInvalid {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })?;
    text.push('\n');
    std::fs::write(path, text).map_err(|e| io_error(path, &e))
}

pub fn read_json<T: DeserializeOwned>(path: &Path) -> Result<T, RecordError> {
    let text = std::fs::read_to_string(path).map_err(|e| io_error(path, &e))?;
    serde_json::from_str(&text).map_err(|e| RecordError::StoredInvalid {
        path: path.to_path_buf(),
        reason: e.to_string(),
    })
}

/// Apaga só o diretório de goldens, que este comando gera; o restante do corpus nunca é tocado.
pub fn reset_goldens(corpus: &Path) -> Result<(), RecordError> {
    let dir = corpus.join(GOLDEN_DIR);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).map_err(|e| io_error(&dir, &e))?;
    }
    Ok(())
}

/// Versão do `typescript` instalado no corpus (lida do `package.json` do pacote).
pub fn installed_typescript(corpus: &Path) -> Result<String, RecordError> {
    let path = corpus.join("node_modules/typescript/package.json");
    let value: serde_json::Value = read_json(&path)?;
    value
        .get("version")
        .and_then(serde_json::Value::as_str)
        .map(str::to_owned)
        .ok_or_else(|| RecordError::StoredInvalid {
            path,
            reason: "campo `version` ausente".to_owned(),
        })
}
