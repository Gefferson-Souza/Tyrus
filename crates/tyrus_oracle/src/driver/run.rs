//! Orquestra o corpus: referência (3 execuções sob `exact`), hospedado, promovido,
//! veredito e gravação de um `equivalence.json` por sonda.
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::driver::classify::{is_stable, run_perturbed};
use crate::driver::corpus::{list_probes, node_flag};
use crate::driver::decide::decide;
use crate::driver::outcome::{sha256_hex, Outcome};
use crate::driver::profile::Profile;
use crate::driver::record::{
    EquivalenceRecord, PromotedRecord, SideRecord, EQUIVALENCE_SCHEMA_VERSION,
};
use crate::driver::runner::{run_command, CommandSpec, RunEnv};
use crate::error::OracleError;
use crate::shares::Shares;
use crate::tuple::{NodeRole, NodeTuple};
use crate::verdict::{Verdict, VerdictRecord};

#[derive(Debug, Clone)]
pub struct RunConfig {
    pub corpus: String,
    pub probes_dir: PathBuf,
    pub out_dir: PathBuf,
    pub profile: Profile,
    pub tuple_role: NodeRole,
    pub tuple: NodeTuple,
    pub reference_node: PathBuf,
    pub hosted: CommandSpec,
    pub promoted: Option<CommandSpec>,
    pub shim: PathBuf,
    pub filter: Option<String>,
    pub reference_timeout: Duration,
    pub hosted_timeout: Duration,
}

#[derive(Debug)]
pub struct RunSummary {
    pub records: Vec<EquivalenceRecord>,
}

impl RunSummary {
    #[must_use]
    pub fn green_count(&self) -> usize {
        self.records
            .iter()
            .filter(|record| record.result.verdict.is_green())
            .count()
    }

    #[must_use]
    pub fn all_green(&self) -> bool {
        self.green_count() == self.records.len()
    }
}

/// Baseline hospedado: o driver não mede CPU, então o split host/plataforma é do ledger.
fn hosted_baseline_shares() -> Result<Shares, OracleError> {
    Shares::new(0.0, 1.0, 0.0, 0.0)
}

fn base_env(cfg: &RunConfig, shim_hash: Option<&str>) -> RunEnv {
    let mut env = RunEnv::default();
    if shim_hash.is_some() {
        env.extra.insert(
            "NODE_OPTIONS".to_owned(),
            format!("--require \"{}\"", cfg.shim.display()),
        );
    }
    env
}

fn read_shim_hash(cfg: &RunConfig) -> Result<Option<String>, OracleError> {
    if !cfg.profile.uses_shim() {
        return Ok(None);
    }
    let bytes = std::fs::read(&cfg.shim).map_err(|e| OracleError::CorpusUnavailable {
        path: cfg.shim.clone(),
        reason: format!("shim de determinismo ilegível: {e}"),
    })?;
    if cfg.shim.to_string_lossy().chars().any(char::is_whitespace) {
        return Err(OracleError::CommandInvalid {
            reason: "o caminho do shim não pode conter espaço".to_owned(),
        });
    }
    Ok(Some(sha256_hex(&bytes)))
}

struct ReferenceRuns {
    outcome: Outcome,
    stable: bool,
}

fn run_reference(cfg: &RunConfig, id: &str, env: &RunEnv) -> Result<ReferenceRuns, OracleError> {
    let spec = CommandSpec {
        program: cfg.reference_node.clone(),
        args: vec![node_flag(id).to_owned()],
    };
    let args = [probe_arg(cfg, id)];
    let timeout = cfg.reference_timeout;
    let baseline = run_command(&spec, &args, &cfg.probes_dir, env, timeout)?;
    if !cfg.profile.uses_shim() || baseline.timed_out {
        return Ok(ReferenceRuns {
            outcome: baseline,
            stable: true,
        });
    }
    let repeat = run_command(&spec, &args, &cfg.probes_dir, env, timeout)?;
    let perturbed = run_perturbed(&spec, &args, &cfg.probes_dir, env, timeout)?;
    let stable = is_stable(&baseline, &repeat, &perturbed);
    Ok(ReferenceRuns {
        outcome: baseline,
        stable,
    })
}

fn probe_arg(cfg: &RunConfig, id: &str) -> String {
    cfg.probes_dir
        .join(format!("{id}.ts"))
        .display()
        .to_string()
}

fn run_target(
    cfg: &RunConfig,
    spec: &CommandSpec,
    id: &str,
    env: &RunEnv,
) -> Result<Outcome, OracleError> {
    run_command(
        spec,
        &[probe_arg(cfg, id)],
        &cfg.probes_dir,
        env,
        cfg.hosted_timeout,
    )
}

fn run_probe(
    cfg: &RunConfig,
    id: &str,
    shim_hash: Option<&String>,
) -> Result<EquivalenceRecord, OracleError> {
    let env = base_env(cfg, shim_hash.map(String::as_str));
    let reference = run_reference(cfg, id, &env)?;
    let (verdict, detail, hosted, promoted) = if reference.stable {
        judge_targets(cfg, id, &env, &reference.outcome)?
    } else {
        (Verdict::NondeterministicReference, None, None, None)
    };
    Ok(EquivalenceRecord {
        schema_version: EQUIVALENCE_SCHEMA_VERSION,
        probe: id.to_owned(),
        corpus: cfg.corpus.clone(),
        profile: cfg.profile.as_str().to_owned(),
        tuple_role: cfg.tuple_role.as_str().to_owned(),
        tuple: cfg.tuple.clone(),
        shim_sha256: shim_hash.cloned(),
        reference_stable: reference.stable,
        result: VerdictRecord {
            verdict,
            shares: hosted_baseline_shares()?,
        },
        detail,
        reference: SideRecord::from_outcome(&reference.outcome),
        hosted,
        promoted,
    })
}

type Judged = (
    Verdict,
    Option<String>,
    Option<SideRecord>,
    Option<PromotedRecord>,
);

fn judge_targets(
    cfg: &RunConfig,
    id: &str,
    env: &RunEnv,
    reference: &Outcome,
) -> Result<Judged, OracleError> {
    let hosted = run_target(cfg, &cfg.hosted, id, env)?;
    let decision = decide(cfg.profile, reference, &hosted);
    let promoted = match &cfg.promoted {
        Some(spec) => {
            let outcome = run_target(cfg, spec, id, env)?;
            let judged = decide(cfg.profile, reference, &outcome);
            Some(PromotedRecord {
                verdict: judged.verdict,
                detail: judged.detail,
                side: SideRecord::from_outcome(&outcome),
            })
        }
        None => None,
    };
    Ok((
        decision.verdict,
        decision.detail,
        Some(SideRecord::from_outcome(&hosted)),
        promoted,
    ))
}

fn write_record(out_dir: &Path, record: &EquivalenceRecord) -> Result<(), OracleError> {
    let dir = out_dir.join(&record.probe);
    let path = dir.join("equivalence.json");
    let failed = |reason: String| OracleError::OutputWriteFailed {
        path: path.clone(),
        reason,
    };
    std::fs::create_dir_all(&dir).map_err(|e| failed(e.to_string()))?;
    let mut text = serde_json::to_string_pretty(record).map_err(|e| failed(e.to_string()))?;
    text.push('\n');
    std::fs::write(&path, text).map_err(|e| failed(e.to_string()))
}

pub fn run_corpus(cfg: &RunConfig) -> Result<RunSummary, OracleError> {
    let ids = list_probes(&cfg.probes_dir, cfg.filter.as_deref())?;
    let shim_hash = read_shim_hash(cfg)?;
    let mut records = Vec::with_capacity(ids.len());
    for id in &ids {
        let record = run_probe(cfg, id, shim_hash.as_ref())?;
        write_record(&cfg.out_dir, &record)?;
        records.push(record);
    }
    Ok(RunSummary { records })
}
