//! Formato gravado: goldens por sonda, `golden/meta.json` e `rerecord-diff.json`.
use serde::{Deserialize, Serialize};

use crate::codes::BadProbeReason;
use crate::record::classify::NONDETERMINISTIC_PREFIX;
use crate::tuple::NodeTuple;

pub const SCHEMA_VERSION: u32 = 1;
pub const COMPARISON_PROFILE: &str = "exact";
pub const EMIT_DESCRIPTION: &str = "tsc per-file emit (front sidecar), module commonjs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Golden {
    pub id: String,
    pub set: String,
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GoldenMeta {
    pub schema_version: u32,
    pub reference: NodeTuple,
    pub typescript: String,
    pub comparison_profile: String,
    pub emit: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Status {
    #[serde(rename = "SAME")]
    Same,
    #[serde(rename = "CHANGED")]
    Changed,
    #[serde(rename = "BAD_PROBE")]
    BadProbe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffEntry {
    pub id: String,
    pub set: String,
    pub status: Status,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reason: Option<BadProbeReason>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub change: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed_verdict: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Counts {
    pub total: usize,
    pub same: usize,
    pub changed: usize,
    pub bad_probe: usize,
    /// Subconjunto de `bad_probe` excluído por referência instável.
    pub nondeterministic: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DiffReport {
    pub schema_version: u32,
    pub reference: NodeTuple,
    pub seed: NodeTuple,
    pub typescript: String,
    pub counts: Counts,
    pub probes: Vec<DiffEntry>,
}

impl Counts {
    #[must_use]
    pub fn tally(entries: &[DiffEntry]) -> Self {
        let count = |status: Status| entries.iter().filter(|e| e.status == status).count();
        let nondeterministic = entries
            .iter()
            .filter(|e| {
                e.reason
                    .as_ref()
                    .is_some_and(|r| r.as_str().starts_with(NONDETERMINISTIC_PREFIX))
            })
            .count();
        Self {
            total: entries.len(),
            same: count(Status::Same),
            changed: count(Status::Changed),
            bad_probe: count(Status::BadProbe),
            nondeterministic,
        }
    }
}
