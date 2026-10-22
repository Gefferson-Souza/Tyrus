#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::ffi::OsString;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use tyrus_oracle::cli::{run, EXIT_ORACLE_ERROR, EXIT_USAGE};

fn lock_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/oracle.lock")
}

fn lock_arg() -> String {
    lock_path().display().to_string()
}

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|s| (*s).to_owned()).collect()
}

struct Run {
    code: u8,
    stdout: String,
    stderr: String,
}

fn run_in_process(items: &[&str], env: impl Fn(&str) -> Option<OsString>) -> Run {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args(items), &env, &mut out, &mut err).unwrap();
    Run {
        code,
        stdout: String::from_utf8(out).unwrap(),
        stderr: String::from_utf8(err).unwrap(),
    }
}

fn process_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

#[test]
fn verify_reference_prints_the_measured_tuple() {
    let run = run_in_process(&["lock", "verify", "--lock", &lock_arg()], process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "OK reference node=26.11.1 v8=14.6.202.34-node.37 icu=78.3 unicode=17.0 tz=2026e cldr=48.0\n"
    );
}

#[test]
fn verify_all_checks_both_roles() {
    let run = run_in_process(
        &["lock", "verify", "--lock", &lock_arg(), "--role", "all"],
        process_env,
    );
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(run.stdout.lines().count(), 2);
    assert!(
        run.stdout.contains("OK seed node=22.21.1"),
        "{}",
        run.stdout
    );
}

#[test]
fn verify_fails_with_e5001_when_the_binary_has_another_tuple() {
    let seed_as_reference = |name: &str| {
        let wanted = if name == "TYRUS_NODE_REFERENCE" {
            "TYRUS_NODE_SEED"
        } else {
            name
        };
        std::env::var_os(wanted)
    };
    let run = run_in_process(
        &["lock", "verify", "--lock", &lock_arg()],
        seed_as_reference,
    );
    assert_eq!(run.code, EXIT_ORACLE_ERROR);
    assert!(run.stderr.starts_with("TYRUS-E5001: "), "{}", run.stderr);
    assert!(run.stdout.is_empty());
}

#[test]
fn verify_fails_with_e5002_without_the_env_var() {
    let run = run_in_process(&["lock", "verify", "--lock", &lock_arg()], |_| None);
    assert_eq!(run.code, EXIT_ORACLE_ERROR);
    assert!(run.stderr.starts_with("TYRUS-E5002: "), "{}", run.stderr);
}

#[test]
fn verify_fails_with_e5005_for_a_missing_lock() {
    let run = run_in_process(
        &["lock", "verify", "--lock", "/nao/existe.lock"],
        process_env,
    );
    assert_eq!(run.code, EXIT_ORACLE_ERROR);
    assert!(run.stderr.starts_with("TYRUS-E5005: "), "{}", run.stderr);
}

#[test]
fn usage_errors_exit_64_and_print_the_usage() {
    let bad_invocations: [&[&str]; 8] = [
        &[],
        &["lock"],
        &["verify"],
        &["lock", "show"],
        &["verify", "verify"],
        &["lock", "verify", "--role", "latest"],
        &["lock", "verify", "--lock"],
        &["lock", "verify", "--bogus"],
    ];
    for invocation in bad_invocations {
        let run = run_in_process(invocation, process_env);
        assert_eq!(run.code, EXIT_USAGE, "{invocation:?}");
        assert!(
            run.stderr.contains("uso: tyrus-oracle lock verify"),
            "{}",
            run.stderr
        );
    }
}

fn run_binary(extra: &[&str], remove_env: &[&str]) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_tyrus-oracle"));
    command
        .args(["lock", "verify", "--lock", &lock_arg()])
        .args(extra);
    for name in remove_env {
        command.env_remove(name);
    }
    command.output().unwrap()
}

#[test]
fn binary_maps_results_to_exit_codes() {
    let ok = run_binary(&[], &[]);
    assert_eq!(
        ok.status.code(),
        Some(0),
        "{}",
        String::from_utf8_lossy(&ok.stderr)
    );
    let unset = run_binary(&[], &["TYRUS_NODE_REFERENCE"]);
    assert_eq!(unset.status.code(), Some(i32::from(EXIT_ORACLE_ERROR)));
    let usage = Command::new(env!("CARGO_BIN_EXE_tyrus-oracle"))
        .output()
        .unwrap();
    assert_eq!(usage.status.code(), Some(i32::from(EXIT_USAGE)));
}

#[test]
fn exit_codes_are_part_of_the_contract() {
    assert_eq!((EXIT_ORACLE_ERROR, EXIT_USAGE), (2, 64));
}
