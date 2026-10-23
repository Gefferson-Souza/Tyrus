//! Valida o fecho contra o rastro de carga: tudo que o oráculo carregou precisa estar no
//! `closure.lock` e no disco, e os bytes do fecho precisam bater com os hashes.
use std::collections::BTreeSet;
use std::path::Path;

use tyrus_facts::{ClosureFile, ClosureLock};

use crate::error::ClosureError;
use crate::trace::{TraceEntry, TraceKind};

#[derive(Debug, PartialEq, Eq)]
pub struct ValidationReport {
    /// Arquivos distintos do rastro conferidos contra o fecho.
    pub checked: usize,
    pub external: usize,
}

/// # Errors
/// `E7202` com todos os arquivos carregados ausentes; `E7101` se algum byte do fecho mudou.
pub fn validate_trace(
    root: &Path,
    lock: &ClosureLock,
    trace: &[TraceEntry],
) -> Result<ValidationReport, ClosureError> {
    let locked: BTreeSet<&str> = lock.files().iter().map(ClosureFile::path).collect();
    let traced: BTreeSet<&str> = trace
        .iter()
        .filter(|entry| entry.kind != TraceKind::External)
        .map(|entry| entry.path.as_str())
        .collect();
    let missing: Vec<String> = traced
        .iter()
        .filter(|path| !locked.contains(**path) || !root.join(path).is_file())
        .map(|path| (*path).to_owned())
        .collect();
    if !missing.is_empty() {
        return Err(ClosureError::TraceMissing { paths: missing });
    }
    lock.verify(root)?;
    Ok(ValidationReport {
        checked: traced.len(),
        external: external_count(trace),
    })
}

fn external_count(trace: &[TraceEntry]) -> usize {
    trace
        .iter()
        .filter(|entry| entry.kind == TraceKind::External)
        .count()
}
