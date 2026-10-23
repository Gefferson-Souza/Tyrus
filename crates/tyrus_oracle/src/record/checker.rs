//! Modo `--check`: reexecuta a referência duas vezes e compara com o que está gravado.
use std::path::Path;

use super::capture::capture_source;
use super::classify::Capture;
use super::error::RecordError;
use super::model::{DiffReport, Golden, GoldenMeta, Status};
use super::sources::Source;
use super::store::{diff_path, golden_path, installed_typescript, meta_path, read_json};
use crate::lock::OracleLock;

fn reason_kind(reason: &str) -> &str {
    reason.split([':', '(']).next().unwrap_or(reason)
}

fn check_meta(corpus: &Path, lock: &OracleLock) -> Result<Vec<String>, RecordError> {
    let meta: GoldenMeta = read_json(&meta_path(corpus))?;
    let mut problems = Vec::new();
    if meta.reference != lock.reference {
        problems.push("golden/meta.json: tupla difere de oracle.lock".to_owned());
    }
    if meta.typescript != installed_typescript(corpus)? {
        problems.push("golden/meta.json: versão do typescript difere da instalada".to_owned());
    }
    Ok(problems)
}

fn check_one(
    corpus: &Path,
    committed: &DiffReport,
    source: &Source,
    capture: &Capture,
) -> Result<Option<String>, RecordError> {
    let label = format!("{}/{}", source.set, source.id);
    let entry = committed
        .probes
        .iter()
        .find(|p| p.id == source.id && p.set == source.set);
    let Some(entry) = entry else {
        return Ok(Some(format!("{label}: ausente de rerecord-diff.json")));
    };
    Ok(match capture {
        Capture::Bad(reason) => match (&entry.status, &entry.reason) {
            (Status::BadProbe, Some(old))
                if reason_kind(old.as_str()) == reason_kind(reason.as_str()) =>
            {
                None
            }
            _ => Some(format!("{label}: agora é BAD_PROBE ({})", reason.as_str())),
        },
        Capture::Golden { stdout, stderr } => {
            let path = golden_path(corpus, source.set, &source.id);
            let stored: Golden = if path.is_file() {
                read_json(&path)?
            } else {
                return Ok(Some(format!("{label}: golden ausente")));
            };
            (stored.stdout != *stdout || stored.stderr != *stderr || stored.exit_code != 0)
                .then(|| format!("{label}: golden diverge da referência"))
        }
    })
}

/// Lista de divergências; vazia quando goldens, meta e diff conferem com duas execuções frescas.
pub fn check_corpus(
    lock: &OracleLock,
    reference: &Path,
    corpus: &Path,
    emit_dir: &Path,
    sources: &[Source],
) -> Result<Vec<String>, RecordError> {
    let mut problems = check_meta(corpus, lock)?;
    let committed: DiffReport = read_json(&diff_path(corpus))?;
    if committed.counts.total != sources.len() {
        problems.push("rerecord-diff.json: total difere do número de sondas".to_owned());
    }
    for source in sources {
        let capture = capture_source(reference, emit_dir, source)?;
        problems.extend(check_one(corpus, &committed, source, &capture)?);
    }
    Ok(problems)
}
