//! Classificador de perturbação: uma referência cuja saída muda com a espera real
//! (ordem de timers sensível a >= 10 ms) não serve de oráculo.
use std::time::Duration;

use crate::driver::outcome::Outcome;
use crate::driver::runner::{run_command, CommandSpec, RunEnv};
use crate::error::OracleError;

pub const PERTURB_ENV: &str = "TYRUS_SHIM_PERTURB_MS";
pub const PERTURB_MS: u32 = 15;

/// A referência é estável se a repetição e a execução perturbada reproduzem a base.
#[must_use]
pub fn is_stable(baseline: &Outcome, repeat: &Outcome, perturbed: &Outcome) -> bool {
    baseline.same_observable(repeat) && baseline.same_observable(perturbed)
}

pub fn run_perturbed(
    spec: &CommandSpec,
    args: &[String],
    cwd: &std::path::Path,
    env: &RunEnv,
    timeout: Duration,
) -> Result<Outcome, OracleError> {
    let mut perturbed = env.clone();
    perturbed
        .extra
        .insert(PERTURB_ENV.to_owned(), PERTURB_MS.to_string());
    run_command(spec, args, cwd, &perturbed, timeout)
}
