//! Classificação pura de uma sonda: golden estável, `BAD_PROBE` ou diferença contra a semente.
use std::path::Path;

use super::exec::RunOutcome;
use crate::codes::BadProbeReason;
use crate::error::OracleError;

pub const NONDETERMINISTIC_PREFIX: &str = "NONDETERMINISTIC_REFERENCE";
pub const KILLED_BY_SIGNAL: &str = "reference_killed_by_signal";
const EMIT_MASK: &str = "<emit>";
const REASON_LIMIT: usize = 180;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Capture {
    Golden { stdout: String, stderr: String },
    Bad(BadProbeReason),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    BaselineFailed,
    StdoutDiffers,
    StdoutDiffersOnlyInTrailingWhitespace,
}

impl Change {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::BaselineFailed => "baseline_exit_nonzero",
            Self::StdoutDiffers => "stdout_differs",
            Self::StdoutDiffersOnlyInTrailingWhitespace => {
                "stdout_differs_only_in_trailing_whitespace"
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Comparison {
    Same,
    Changed(Change),
}

/// Sonda ruim com motivo de uma linha, truncado e sem caracteres de controle.
pub fn bad_probe(text: &str) -> Result<Capture, OracleError> {
    let clipped: String = text
        .chars()
        .take(REASON_LIMIT)
        .map(|c| if c.is_control() { ' ' } else { c })
        .collect();
    BadProbeReason::try_from(clipped).map(Capture::Bad)
}

/// Primeira linha `Nome: mensagem` de um erro não capturado, ignorando o rastro de pilha.
#[must_use]
pub fn uncaught_error_line(stderr: &str) -> Option<String> {
    stderr
        .lines()
        .map(str::trim)
        .find(|line| is_error_line(line))
        .map(str::to_owned)
}

fn is_error_line(line: &str) -> bool {
    let Some((name, _)) = line.split_once(": ") else {
        return false;
    };
    !name.is_empty() && name.ends_with("Error") && name.chars().all(|c| c.is_ascii_alphanumeric())
}

fn describe_failure(run: &RunOutcome) -> Option<String> {
    if run.timed_out {
        return Some("reference_timeout".to_owned());
    }
    let Some(code) = run.exit_code else {
        return Some(KILLED_BY_SIGNAL.to_owned());
    };
    if code == 0 {
        return None;
    }
    let stderr = String::from_utf8_lossy(&run.stderr);
    Some(match uncaught_error_line(&stderr) {
        Some(line) => format!("reference_exit_nonzero({code}): {line}"),
        None => format!("reference_exit_nonzero({code})"),
    })
}

fn mask_emit(stderr: &[u8], emit_dir: &Path) -> String {
    let text = String::from_utf8_lossy(stderr).into_owned();
    text.replace(&emit_dir.display().to_string(), EMIT_MASK)
}

/// Duas execuções da referência viram um golden só se concordarem e saírem com 0.
pub fn capture_reference(
    first: &RunOutcome,
    second: &RunOutcome,
    emit_dir: &Path,
) -> Result<Capture, OracleError> {
    if let Some(reason) = describe_failure(first).or_else(|| describe_failure(second)) {
        return bad_probe(&reason);
    }
    let same = first.stdout == second.stdout
        && mask_emit(&first.stderr, emit_dir) == mask_emit(&second.stderr, emit_dir);
    if !same {
        return bad_probe(&format!(
            "{NONDETERMINISTIC_PREFIX}: duas execuções da referência divergem"
        ));
    }
    match String::from_utf8(first.stdout.clone()) {
        Ok(stdout) => Ok(Capture::Golden {
            stdout,
            stderr: mask_emit(&first.stderr, emit_dir),
        }),
        Err(_) => bad_probe("stdout_not_utf8"),
    }
}

/// Compara o golden novo com a saída da semente (Node 22 com remoção de tipos), byte a byte.
#[must_use]
pub fn compare_baseline(golden_stdout: &str, baseline: &RunOutcome) -> Comparison {
    if baseline.timed_out || baseline.exit_code != Some(0) {
        return Comparison::Changed(Change::BaselineFailed);
    }
    if baseline.stdout == golden_stdout.as_bytes() {
        return Comparison::Same;
    }
    let seed_text = String::from_utf8_lossy(&baseline.stdout);
    if seed_text.trim_end() == golden_stdout.trim_end() {
        Comparison::Changed(Change::StdoutDiffersOnlyInTrailingWhitespace)
    } else {
        Comparison::Changed(Change::StdoutDiffers)
    }
}
