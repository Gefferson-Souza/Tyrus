//! Emit por arquivo do `tsc` fixado do corpus, pelo sidecar `front/src/cli.ts`.
use std::ffi::OsString;
use std::path::Path;
use std::time::Duration;

use super::error::{io_error, RecordError};
use super::exec::{run_node, RunSpec};

const EMIT_TIMEOUT: Duration = Duration::from_secs(300);
const STDERR_TAIL_LINES: usize = 5;

fn failure(reason: impl Into<String>) -> RecordError {
    RecordError::EmitFailed {
        reason: reason.into(),
    }
}

/// Roda o sidecar com o Node de referência. Código 1 (erros de tipo; o emit acontece) é aceito;
/// outro código recusa, porque o corpus não poderia ser gravado.
pub fn emit_corpus(
    reference: &Path,
    front: &Path,
    corpus: &Path,
    out: &Path,
) -> Result<(), RecordError> {
    let emitted = out.join("emit");
    if emitted.exists() {
        std::fs::remove_dir_all(&emitted).map_err(|e| io_error(&emitted, &e))?;
    }
    let args: Vec<OsString> = vec![
        front.into(),
        "--project".into(),
        corpus.into(),
        "--out".into(),
        out.into(),
    ];
    let spec = RunSpec {
        binary: reference,
        args: &args,
        cwd: Path::new("."),
        timeout: EMIT_TIMEOUT,
    };
    let outcome = run_node(&spec).map_err(|e| failure(e.to_string()))?;
    if outcome.timed_out {
        return Err(failure("tempo esgotado"));
    }
    match outcome.exit_code {
        Some(0 | 1) => Ok(()),
        other => {
            let text = String::from_utf8_lossy(&outcome.stderr).into_owned();
            let lines: Vec<&str> = text.lines().rev().take(STDERR_TAIL_LINES).collect();
            let tail: Vec<&str> = lines.into_iter().rev().collect();
            Err(failure(format!(
                "sidecar saiu com {other:?}: {}",
                tail.join(" | ")
            )))
        }
    }
}
