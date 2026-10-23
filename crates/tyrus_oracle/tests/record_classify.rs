#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::path::Path;

use tyrus_oracle::record::classify::{
    capture_reference, compare_baseline, Capture, Change, Comparison,
};
use tyrus_oracle::record::exec::RunOutcome;

fn ok(stdout: &str) -> RunOutcome {
    RunOutcome {
        exit_code: Some(0),
        stdout: stdout.as_bytes().to_vec(),
        stderr: Vec::new(),
        timed_out: false,
    }
}

fn failed(code: i32, stderr: &str) -> RunOutcome {
    RunOutcome {
        exit_code: Some(code),
        stdout: Vec::new(),
        stderr: stderr.as_bytes().to_vec(),
        timed_out: false,
    }
}

fn reason_of(capture: &Capture) -> String {
    match capture {
        Capture::Bad(reason) => reason.as_str().to_owned(),
        Capture::Golden { .. } => panic!("esperava BAD_PROBE"),
    }
}

#[test]
fn two_identical_clean_runs_become_a_golden() {
    let capture = capture_reference(&ok("1\n2\n"), &ok("1\n2\n"), Path::new("/e")).unwrap();
    assert_eq!(
        capture,
        Capture::Golden {
            stdout: "1\n2\n".to_owned(),
            stderr: String::new()
        }
    );
}

#[test]
fn diverging_runs_are_nondeterministic_and_excluded() {
    let capture = capture_reference(&ok("a\n"), &ok("b\n"), Path::new("/e")).unwrap();
    assert!(reason_of(&capture).starts_with("NONDETERMINISTIC_REFERENCE"));
}

#[test]
fn nonzero_exit_is_a_bad_probe_with_the_uncaught_error_line() {
    let stderr = "/x/p.js:5\nconst v = await f();\n          ^^^^^\n\nSyntaxError: await is only valid in async functions\n    at wrapSafe (node:internal)\n";
    let run = failed(1, stderr);
    let reason = reason_of(&capture_reference(&run, &run, Path::new("/e")).unwrap());
    assert_eq!(
        reason,
        "reference_exit_nonzero(1): SyntaxError: await is only valid in async functions"
    );
}

#[test]
fn timeout_is_a_bad_probe() {
    let mut run = ok("");
    run.timed_out = true;
    run.exit_code = None;
    assert_eq!(
        reason_of(&capture_reference(&run, &run, Path::new("/e")).unwrap()),
        "reference_timeout"
    );
}

#[test]
fn non_utf8_stdout_is_a_bad_probe() {
    let mut run = ok("");
    run.stdout = vec![0xff, 0xfe];
    assert_eq!(
        reason_of(&capture_reference(&run, &run, Path::new("/e")).unwrap()),
        "stdout_not_utf8"
    );
}

#[test]
fn emit_directory_is_masked_in_stderr() {
    let mut run = ok("x\n");
    run.stderr = b"aviso em /work/emit/probes/a.js\n".to_vec();
    let Capture::Golden { stderr, .. } =
        capture_reference(&run, &run, Path::new("/work/emit")).unwrap()
    else {
        panic!("esperava golden");
    };
    assert_eq!(stderr, "aviso em <emit>/probes/a.js\n");
}

#[test]
fn baseline_equal_bytes_is_same() {
    assert_eq!(compare_baseline("7\n", &ok("7\n")), Comparison::Same);
}

#[test]
fn baseline_with_different_stdout_is_changed() {
    assert_eq!(
        compare_baseline("7\n", &ok("8\n")),
        Comparison::Changed(Change::StdoutDiffers)
    );
}

#[test]
fn baseline_differing_only_in_trailing_whitespace_is_labelled() {
    assert_eq!(
        compare_baseline("7\n", &ok("7")),
        Comparison::Changed(Change::StdoutDiffersOnlyInTrailingWhitespace)
    );
}

#[test]
fn failed_baseline_is_changed() {
    assert_eq!(
        compare_baseline("7\n", &failed(1, "boom")),
        Comparison::Changed(Change::BaselineFailed)
    );
}
