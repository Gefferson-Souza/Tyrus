//! Execução de uma sonda na referência (duas vezes) e na semente (uma vez).
use std::ffi::OsString;
use std::path::Path;

use super::classify::{bad_probe, capture_reference, Capture};
use super::error::{io_error, RecordError};
use super::exec::{run_node, RunOutcome, RunSpec, RUN_TIMEOUT};
use super::sources::Source;

const STRIP_TYPES: &str = "--experimental-strip-types";
const TRANSFORM_TYPES: &str = "--experimental-transform-types";
const TRANSFORM_PREFIX: &str = "C13_";

fn run_once(binary: &Path, args: &[OsString], cwd: &Path) -> Result<RunOutcome, RecordError> {
    let spec = RunSpec {
        binary,
        args,
        cwd,
        timeout: RUN_TIMEOUT,
    };
    run_node(&spec).map_err(|e| io_error(binary, &e))
}

/// Duas execuções do JS emitido no Node de referência, classificadas por `capture_reference`.
pub fn capture_source(
    reference: &Path,
    emit_dir: &Path,
    source: &Source,
) -> Result<Capture, RecordError> {
    let js = source.emitted_js(emit_dir);
    if !js.is_file() {
        return Ok(bad_probe("emit_missing")?);
    }
    let args = [OsString::from(&js)];
    let first = run_once(reference, &args, emit_dir)?;
    let second = run_once(reference, &args, emit_dir)?;
    Ok(capture_reference(&first, &second, emit_dir)?)
}

/// Linha de base: o `.ts` da semente executado no Node semente com remoção de tipos.
/// Roda numa cópia sem `package.json` ancestral: o `"type": "commonjs"` do corpus desligaria a
/// detecção de módulo do Node 22 e mudaria o veredito de `await` de topo (C12_04).
pub fn baseline_run(seed: &Path, source: &Source) -> Result<RunOutcome, RecordError> {
    let flag = if source.id.starts_with(TRANSFORM_PREFIX) {
        TRANSFORM_TYPES
    } else {
        STRIP_TYPES
    };
    let dir = std::env::temp_dir()
        .join(format!("tyrus-baseline-{}", std::process::id()))
        .join(format!("{}-{}", source.set, source.id));
    std::fs::create_dir_all(&dir).map_err(|e| io_error(&dir, &e))?;
    let copy = dir.join(format!("{}.ts", source.id));
    std::fs::copy(&source.ts_path, &copy).map_err(|e| io_error(&copy, &e))?;
    let args = [
        OsString::from(flag),
        OsString::from("--no-warnings"),
        OsString::from(&copy),
    ];
    let outcome = run_once(seed, &args, &dir);
    // Resíduo em /tmp não altera o resultado já obtido.
    let _ = std::fs::remove_dir_all(&dir);
    outcome
}
