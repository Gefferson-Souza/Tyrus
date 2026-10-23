//! Decisão de veredito: função pura sobre as duas saídas (alvo de mutation test).
use crate::codes::BadProbeReason;
use crate::driver::outcome::Outcome;
use crate::driver::profile::Profile;
use crate::verdict::Verdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Decision {
    pub verdict: Verdict,
    /// Motivo curto de um veredito não verde (`TIMEOUT`, `RUNTIME_PANIC`, `stdout`...).
    pub detail: Option<String>,
}

fn decision(verdict: Verdict, detail: Option<&str>) -> Decision {
    Decision {
        verdict,
        detail: detail.map(str::to_owned),
    }
}

fn bad_probe(reason: &str) -> Decision {
    match BadProbeReason::try_from(reason.to_owned()) {
        Ok(reason) => decision(Verdict::BadProbe { reason }, None),
        // Motivo é texto fixo curto; se a validação falhar, ainda assim não é verde.
        Err(_) => decision(Verdict::Different, Some("BAD_PROBE_REASON_INVALID")),
    }
}

fn trimmed(bytes: &[u8]) -> String {
    String::from_utf8_lossy(bytes).trim().to_owned()
}

fn decide_legacy(reference: &Outcome, hosted: &Outcome) -> Decision {
    if reference.exit_code != Some(0) {
        return bad_probe(&format!(
            "referência saiu com {:?}",
            reference.exit_code.unwrap_or(-1)
        ));
    }
    if hosted.timed_out {
        return decision(Verdict::Different, Some("TIMEOUT"));
    }
    if hosted.exit_code != Some(0) {
        return decision(Verdict::Different, Some("RUNTIME_PANIC"));
    }
    if trimmed(&reference.stdout) == trimmed(&hosted.stdout) {
        decision(Verdict::Identical, None)
    } else {
        decision(Verdict::Different, Some("stdout"))
    }
}

fn decide_exact(reference: &Outcome, hosted: &Outcome) -> Decision {
    if hosted.timed_out {
        return decision(Verdict::Different, Some("TIMEOUT"));
    }
    let mismatches: Vec<&str> = [
        ("stdout", reference.stdout != hosted.stdout),
        ("stderr", reference.stderr != hosted.stderr),
        ("exit_code", reference.exit_code != hosted.exit_code),
    ]
    .into_iter()
    .filter_map(|(field, differs)| differs.then_some(field))
    .collect();
    if mismatches.is_empty() {
        decision(Verdict::Identical, None)
    } else {
        decision(Verdict::Different, Some(&mismatches.join(",")))
    }
}

/// Saída 9 do Node é opção inválida; os outros marcadores são falha do harness, não
/// comportamento do programa, e nunca podem servir de oráculo.
const HARNESS_EXIT_CODE: i32 = 9;
const HARNESS_STDERR_MARKERS: [&str; 2] = ["bad option", "ERR_UNSUPPORTED_TYPESCRIPT_SYNTAX"];

fn reference_harness_failure(reference: &Outcome) -> Option<&'static str> {
    let stderr = String::from_utf8_lossy(&reference.stderr);
    if reference.exit_code == Some(HARNESS_EXIT_CODE) || stderr.contains(HARNESS_STDERR_MARKERS[0])
    {
        return Some("referência rejeitou a opção do Node");
    }
    stderr
        .contains(HARNESS_STDERR_MARKERS[1])
        .then_some("referência não executa a sintaxe TypeScript da sonda")
}

#[must_use]
pub fn decide(profile: Profile, reference: &Outcome, hosted: &Outcome) -> Decision {
    if reference.timed_out {
        return bad_probe("referência estourou o tempo");
    }
    match profile {
        Profile::Exact => match reference_harness_failure(reference) {
            Some(reason) => bad_probe(reason),
            None => decide_exact(reference, hosted),
        },
        Profile::LegacyTrim => decide_legacy(reference, hosted),
    }
}
