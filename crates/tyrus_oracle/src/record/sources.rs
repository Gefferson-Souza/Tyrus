//! Descoberta das sondas do corpus e do caminho do JS emitido de cada uma.
use std::path::{Path, PathBuf};

use super::error::{io_error, RecordError};

pub const SETS: [&str; 2] = ["probes", "calib"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Source {
    pub set: &'static str,
    pub id: String,
    pub ts_path: PathBuf,
}

impl Source {
    #[must_use]
    pub fn emitted_js(&self, emit_dir: &Path) -> PathBuf {
        emit_dir.join(self.set).join(format!("{}.js", self.id))
    }
}

fn set_sources(corpus: &Path, set: &'static str) -> Result<Vec<Source>, RecordError> {
    let dir = corpus.join(set);
    let entries = std::fs::read_dir(&dir).map_err(|e| RecordError::CorpusInvalid {
        path: dir.clone(),
        reason: e.to_string(),
    })?;
    let mut sources = Vec::new();
    for entry in entries {
        let path = entry.map_err(|e| io_error(&dir, &e))?.path();
        let is_ts = path.extension().is_some_and(|ext| ext == "ts");
        let stem = path.file_stem().and_then(|s| s.to_str());
        if let (true, Some(id)) = (is_ts, stem) {
            sources.push(Source {
                set,
                id: id.to_owned(),
                ts_path: path.clone(),
            });
        }
    }
    Ok(sources)
}

/// Todas as sondas de `probes/` e `calib/`, em ordem estável por (conjunto, id).
pub fn list_sources(corpus: &Path) -> Result<Vec<Source>, RecordError> {
    let mut all = Vec::new();
    for set in SETS {
        all.extend(set_sources(corpus, set)?);
    }
    if all.is_empty() {
        return Err(RecordError::CorpusInvalid {
            path: corpus.to_path_buf(),
            reason: "nenhuma sonda .ts encontrada".to_owned(),
        });
    }
    all.sort_by(|a, b| (a.set, &a.id).cmp(&(b.set, &b.id)));
    Ok(all)
}
