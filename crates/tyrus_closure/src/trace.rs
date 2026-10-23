//! Rastro de carga de módulos do oráculo (jsonl): uma linha por arquivo carregado ou lido.
use serde::Deserialize;
use tyrus_facts::RelativePath;

use crate::error::ClosureError;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TraceKind {
    Load,
    Read,
    /// Arquivo fora da raiz do fecho; o validador só o contabiliza.
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TraceEntry {
    pub kind: TraceKind,
    pub path: String,
}

/// # Errors
/// `E7203` na primeira linha inválida (JSON, campo desconhecido ou caminho que escapa).
pub fn parse_trace(text: &str) -> Result<Vec<TraceEntry>, ClosureError> {
    let mut entries = Vec::new();
    for (index, line) in text.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        entries.push(parse_line(line, index + 1)?);
    }
    Ok(entries)
}

fn parse_line(line: &str, number: usize) -> Result<TraceEntry, ClosureError> {
    let invalid = |reason: String| ClosureError::InvalidTrace {
        line: number,
        reason,
    };
    let entry: TraceEntry =
        serde_json::from_str(line).map_err(|error| invalid(error.to_string()))?;
    if entry.kind != TraceKind::External {
        RelativePath::parse(&entry.path).map_err(|error| invalid(error.to_string()))?;
    }
    Ok(entry)
}
