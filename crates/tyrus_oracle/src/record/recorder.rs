//! Modo gravação: goldens, `meta.json` e `rerecord-diff.json`.
use std::path::{Path, PathBuf};

use super::capture::{baseline_run, capture_source};
use super::classify::{compare_baseline, Capture, Comparison};
use super::error::RecordError;
use super::model::{
    Counts, DiffEntry, DiffReport, Golden, GoldenMeta, Status, COMPARISON_PROFILE,
    EMIT_DESCRIPTION, SCHEMA_VERSION,
};
use super::sources::Source;
use super::store::{
    diff_path, golden_path, installed_typescript, meta_path, reset_goldens, write_json,
};
use crate::lock::OracleLock;

#[derive(Debug)]
pub struct RecordInputs<'a> {
    pub lock: &'a OracleLock,
    pub reference: &'a Path,
    pub seed: &'a Path,
    pub corpus: &'a Path,
    pub emit_dir: &'a Path,
    pub seed_results: Option<&'a Path>,
}

fn seed_verdict(dir: Option<&Path>, id: &str) -> Option<String> {
    let path: PathBuf = dir?.join(format!("{id}.json"));
    let value: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(path).ok()?).ok()?;
    value.get("verdict")?.as_str().map(str::to_owned)
}

fn entry(source: &Source, status: Status) -> DiffEntry {
    DiffEntry {
        id: source.id.clone(),
        set: source.set.to_owned(),
        status,
        reason: None,
        change: None,
        seed_verdict: None,
        baseline_exit_code: None,
        baseline_stdout: None,
    }
}

fn record_one(inputs: &RecordInputs<'_>, source: &Source) -> Result<DiffEntry, RecordError> {
    let mut diff = entry(source, Status::Same);
    diff.seed_verdict = seed_verdict(inputs.seed_results, &source.id);
    match capture_source(inputs.reference, inputs.emit_dir, source)? {
        Capture::Bad(reason) => {
            diff.status = Status::BadProbe;
            diff.reason = Some(reason);
            let baseline = baseline_run(inputs.seed, source)?;
            diff.baseline_exit_code = baseline.exit_code;
            diff.baseline_stdout = Some(String::from_utf8_lossy(&baseline.stdout).into_owned());
        }
        Capture::Golden { stdout, stderr } => {
            let golden = Golden {
                id: source.id.clone(),
                set: source.set.to_owned(),
                exit_code: 0,
                stdout,
                stderr,
            };
            write_json(&golden_path(inputs.corpus, source.set, &source.id), &golden)?;
            let baseline = baseline_run(inputs.seed, source)?;
            if let Comparison::Changed(change) = compare_baseline(&golden.stdout, &baseline) {
                diff.status = Status::Changed;
                diff.change = Some(change.as_str().to_owned());
            }
        }
    }
    Ok(diff)
}

/// Regrava todos os goldens e o diff contra a semente; devolve o relatório gravado.
pub fn record_corpus(
    inputs: &RecordInputs<'_>,
    sources: &[Source],
) -> Result<DiffReport, RecordError> {
    reset_goldens(inputs.corpus)?;
    let typescript = installed_typescript(inputs.corpus)?;
    let mut probes = Vec::with_capacity(sources.len());
    for source in sources {
        probes.push(record_one(inputs, source)?);
    }
    let meta = GoldenMeta {
        schema_version: SCHEMA_VERSION,
        reference: inputs.lock.reference.clone(),
        typescript: typescript.clone(),
        comparison_profile: COMPARISON_PROFILE.to_owned(),
        emit: EMIT_DESCRIPTION.to_owned(),
    };
    write_json(&meta_path(inputs.corpus), &meta)?;
    let report = DiffReport {
        schema_version: SCHEMA_VERSION,
        reference: inputs.lock.reference.clone(),
        seed: inputs.lock.seed.clone(),
        typescript,
        counts: Counts::tally(&probes),
        probes,
    };
    write_json(&diff_path(inputs.corpus), &report)?;
    Ok(report)
}
