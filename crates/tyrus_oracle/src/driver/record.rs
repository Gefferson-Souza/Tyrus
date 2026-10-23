//! Registro `equivalence.json` de uma sonda (esquema em `schemas/equivalence.schema.json`).
use serde::{Deserialize, Serialize};

use crate::driver::outcome::{sha256_hex, Outcome};
use crate::tuple::NodeTuple;
use crate::verdict::{Verdict, VerdictRecord};

pub const EQUIVALENCE_SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SideRecord {
    pub exit_code: Option<i32>,
    pub stdout_sha256: String,
    pub stderr_sha256: String,
    pub error_first_line: Option<String>,
    pub timed_out: bool,
}

impl SideRecord {
    #[must_use]
    pub fn from_outcome(outcome: &Outcome) -> Self {
        Self {
            exit_code: outcome.exit_code,
            stdout_sha256: sha256_hex(&outcome.stdout),
            stderr_sha256: sha256_hex(&outcome.stderr),
            error_first_line: outcome.error_first_line(),
            timed_out: outcome.timed_out,
        }
    }
}

/// Terceira coluna: o promovido é julgado contra a mesma referência, sem frações
/// (a fração nativa vem da medição do ledger, não do driver).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromotedRecord {
    pub verdict: Verdict,
    pub detail: Option<String>,
    pub side: SideRecord,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EquivalenceRecord {
    pub schema_version: u32,
    pub probe: String,
    pub corpus: String,
    pub profile: String,
    pub tuple_role: String,
    pub tuple: NodeTuple,
    pub shim_sha256: Option<String>,
    pub reference_stable: bool,
    pub result: VerdictRecord,
    pub detail: Option<String>,
    pub reference: SideRecord,
    pub hosted: Option<SideRecord>,
    pub promoted: Option<PromotedRecord>,
}
