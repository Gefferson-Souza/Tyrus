#![allow(clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::driver::decide::decide;
use tyrus_oracle::driver::outcome::Outcome;
use tyrus_oracle::driver::profile::Profile;
use tyrus_oracle::verdict::Verdict;

fn outcome(stdout: &str, exit_code: i32) -> Outcome {
    Outcome {
        exit_code: Some(exit_code),
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
        timed_out: false,
    }
}

#[test]
fn trailing_space_differs_under_exact_and_matches_under_legacy_trim() {
    let reference = outcome("1\n", 0);
    let hosted = outcome("1 \n", 0);
    assert_eq!(
        decide(Profile::Exact, &reference, &hosted).verdict,
        Verdict::Different
    );
    assert_eq!(
        decide(Profile::LegacyTrim, &reference, &hosted).verdict,
        Verdict::Identical
    );
}

#[test]
fn exact_compares_stderr_and_exit_code() {
    let reference = outcome("a\n", 0);
    let mut hosted = outcome("a\n", 0);
    hosted.stderr = b"aviso\n".to_vec();
    let decision = decide(Profile::Exact, &reference, &hosted);
    assert_eq!(decision.verdict, Verdict::Different);
    assert_eq!(decision.detail.as_deref(), Some("stderr"));
    let decision = decide(Profile::Exact, &reference, &outcome("a\n", 3));
    assert_eq!(decision.detail.as_deref(), Some("exit_code"));
}

#[test]
fn legacy_trim_turns_a_failing_reference_into_a_bad_probe() {
    let decision = decide(Profile::LegacyTrim, &outcome("", 1), &outcome("", 0));
    assert!(matches!(decision.verdict, Verdict::BadProbe { .. }));
}

#[test]
fn legacy_trim_reports_a_failing_hosted_run_as_runtime_panic() {
    let decision = decide(Profile::LegacyTrim, &outcome("x", 0), &outcome("x", 101));
    assert_eq!(decision.verdict, Verdict::Different);
    assert_eq!(decision.detail.as_deref(), Some("RUNTIME_PANIC"));
}

#[test]
fn a_hosted_timeout_is_never_identical() {
    let mut hosted = outcome("x", 0);
    hosted.timed_out = true;
    for profile in [Profile::Exact, Profile::LegacyTrim] {
        let decision = decide(profile, &outcome("x", 0), &hosted);
        assert_eq!(decision.verdict, Verdict::Different, "{profile:?}");
        assert_eq!(decision.detail.as_deref(), Some("TIMEOUT"));
    }
}

#[test]
fn a_reference_timeout_is_a_bad_probe() {
    let mut reference = outcome("x", 0);
    reference.timed_out = true;
    let decision = decide(Profile::Exact, &reference, &outcome("x", 0));
    assert!(matches!(decision.verdict, Verdict::BadProbe { .. }));
}

#[test]
fn identical_outputs_are_identical_under_exact() {
    let decision = decide(Profile::Exact, &outcome("ok\n", 0), &outcome("ok\n", 0));
    assert_eq!(decision.verdict, Verdict::Identical);
    assert_eq!(decision.detail, None);
}

#[test]
fn profile_names_round_trip() {
    assert_eq!("exact".parse::<Profile>().unwrap(), Profile::Exact);
    assert_eq!(
        "legacy_trim".parse::<Profile>().unwrap(),
        Profile::LegacyTrim
    );
    assert_eq!(Profile::LegacyTrim.as_str(), "legacy_trim");
    assert!("outro".parse::<Profile>().is_err());
}

#[test]
fn legacy_trim_still_differs_when_the_trimmed_stdout_differs() {
    let decision = decide(Profile::LegacyTrim, &outcome("a\n", 0), &outcome("b\n", 0));
    assert_eq!(decision.verdict, Verdict::Different);
    assert_eq!(decision.detail.as_deref(), Some("stdout"));
    let decision = decide(Profile::LegacyTrim, &outcome("x\n", 0), &outcome("", 0));
    assert_eq!(decision.verdict, Verdict::Different);
}

fn failing_reference(exit_code: i32, stderr: &str) -> Outcome {
    Outcome {
        exit_code: Some(exit_code),
        stdout: Vec::new(),
        stderr: stderr.as_bytes().to_vec(),
        timed_out: false,
    }
}

#[test]
fn exact_never_trusts_a_reference_that_died_from_a_harness_error() {
    let cases = [
        failing_reference(9, ""),
        failing_reference(9, "node: bad option: --experimental-transform-types\n"),
        failing_reference(1, "node: bad option: --x\n"),
        failing_reference(1, "SyntaxError [ERR_UNSUPPORTED_TYPESCRIPT_SYNTAX]: enum\n"),
    ];
    for reference in cases {
        let decision = decide(Profile::Exact, &reference, &reference.clone());
        assert!(
            matches!(decision.verdict, Verdict::BadProbe { .. }),
            "{reference:?} => {decision:?}"
        );
        assert!(!decision.verdict.is_green());
    }
}

#[test]
fn exact_still_accepts_a_reference_that_fails_by_the_probe_itself() {
    let reference = failing_reference(1, "Error: boom\n    at x\n");
    let decision = decide(Profile::Exact, &reference, &reference.clone());
    assert_eq!(decision.verdict, Verdict::Identical);
}
