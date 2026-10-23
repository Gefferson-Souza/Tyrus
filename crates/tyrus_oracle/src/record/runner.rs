//! Orquestração do comando `record`: tupla, emit, gravação ou checagem.
use std::ffi::OsString;
use std::io::{self, Write};
use std::path::{Path, PathBuf};

use super::checker::check_corpus;
use super::emit::emit_corpus;
use super::error::{io_error, RecordError};
use super::model::DiffReport;
use super::options::{RecordOptions, USAGE};
use super::recorder::{record_corpus, RecordInputs};
use super::sources::list_sources;
use crate::lock::OracleLock;
use crate::probe::{resolve_binary, verify_tuple};
use crate::tuple::NodeRole;

pub const EXIT_OK: u8 = 0;
pub const EXIT_CHECK_FAILED: u8 = 1;
pub const EXIT_ORACLE_ERROR: u8 = 2;
pub const EXIT_USAGE: u8 = 64;
pub const EXIT_IO: u8 = 74;

fn summary(report: &DiffReport) -> String {
    let c = &report.counts;
    format!(
        "record: total={} same={} changed={} bad_probe={} nondeterministic={}",
        c.total, c.same, c.changed, c.bad_probe, c.nondeterministic
    )
}

// Os processos filhos rodam em outro cwd: caminhos relativos têm de virar absolutos antes.
fn absolute(path: &Path) -> Result<PathBuf, RecordError> {
    std::path::absolute(path).map_err(|e| io_error(path, &e))
}

fn execute<F>(
    options: &RecordOptions,
    get_env: &F,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> Result<u8, RecordError>
where
    F: Fn(&str) -> Option<OsString>,
{
    let lock = OracleLock::load(&options.lock)?;
    let reference = resolve_binary(NodeRole::Reference, get_env)?;
    verify_tuple(&lock, NodeRole::Reference, &reference)?;
    let seed = if options.check {
        None
    } else {
        let binary = resolve_binary(NodeRole::Seed, get_env)?;
        verify_tuple(&lock, NodeRole::Seed, &binary)?;
        Some(binary)
    };
    let corpus = absolute(&options.corpus)?;
    let out_dir = absolute(&options.out_dir())?;
    if !options.no_emit {
        emit_corpus(&reference, &options.front, &corpus, &out_dir)?;
    }
    let emit_dir = out_dir.join("emit");
    let sources = list_sources(&corpus)?;
    if let Some(seed) = seed {
        let inputs = RecordInputs {
            lock: &lock,
            reference: &reference,
            seed: &seed,
            corpus: &corpus,
            emit_dir: &emit_dir,
            seed_results: options.seed_results.as_deref(),
        };
        let report = record_corpus(&inputs, &sources)?;
        let _ = writeln!(out, "{}", summary(&report));
        return Ok(EXIT_OK);
    }
    let problems = check_corpus(&lock, &reference, &corpus, &emit_dir, &sources)?;
    for problem in &problems {
        let _ = writeln!(err, "{problem}");
    }
    let _ = writeln!(
        out,
        "check: sondas={} divergências={}",
        sources.len(),
        problems.len()
    );
    Ok(if problems.is_empty() {
        EXIT_OK
    } else {
        EXIT_CHECK_FAILED
    })
}

/// Executa `record` e devolve o código de saída; falhas de E/S nos fluxos sobem como `Err`.
pub fn run_record<F>(
    args: &[String],
    get_env: &F,
    out: &mut dyn Write,
    err: &mut dyn Write,
) -> io::Result<u8>
where
    F: Fn(&str) -> Option<OsString>,
{
    let options = match RecordOptions::parse(args) {
        Ok(options) => options,
        Err(message) => {
            writeln!(err, "{message}\n{USAGE}")?;
            return Ok(EXIT_USAGE);
        }
    };
    match execute(&options, get_env, out, err) {
        Ok(code) => Ok(code),
        Err(error) => {
            writeln!(err, "{}: {error}", error.stable_code())?;
            Ok(match error {
                RecordError::Io { .. } => EXIT_IO,
                _ => EXIT_ORACLE_ERROR,
            })
        }
    }
}
