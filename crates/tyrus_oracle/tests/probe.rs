#![allow(clippy::expect_used, clippy::unwrap_used, clippy::panic)]

use std::ffi::OsString;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tyrus_oracle::error::OracleError;
use tyrus_oracle::lock::OracleLock;
use tyrus_oracle::probe::{
    parse_probe_output, probe_node, probe_node_with_timeout, resolve_binary, verify_tuple,
};
use tyrus_oracle::tuple::{NodeRole, ENV_NODE_REFERENCE, ENV_NODE_SEED};

fn committed_lock() -> OracleLock {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/oracle.lock");
    OracleLock::load(&path).unwrap()
}

fn pinned_binary(role: NodeRole) -> PathBuf {
    let value = std::env::var_os(role.env_var()).unwrap_or_else(|| {
        panic!(
            "defina {} com o caminho absoluto do Node {role} (ver CLAUDE.md)",
            role.env_var()
        )
    });
    PathBuf::from(value)
}

fn fake_node(name: &str, script_body: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("fake-node-{name}"));
    std::fs::write(&path, format!("#!/bin/sh\n{script_body}\n")).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).unwrap();
    path
}

#[test]
fn lock_probe_matches_reference() {
    let lock = committed_lock();
    let measured = verify_tuple(
        &lock,
        NodeRole::Reference,
        &pinned_binary(NodeRole::Reference),
    );
    assert_eq!(measured.unwrap(), lock.reference);
}

#[test]
fn lock_probe_matches_seed() {
    let lock = committed_lock();
    let measured = verify_tuple(&lock, NodeRole::Seed, &pinned_binary(NodeRole::Seed));
    assert_eq!(measured.unwrap(), lock.seed);
}

#[test]
fn lock_probe_rejects_wrong_tuple() {
    let lock = committed_lock();
    let seed_binary = pinned_binary(NodeRole::Seed);
    let error = verify_tuple(&lock, NodeRole::Reference, &seed_binary).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5001");
    let OracleError::TupleMismatch { role, diffs } = error else {
        panic!("esperava TupleMismatch");
    };
    assert_eq!(role, NodeRole::Reference);
    let fields: Vec<&str> = diffs.iter().map(|d| d.field).collect();
    assert_eq!(fields, ["node", "v8", "icu", "unicode", "tz", "cldr"]);
}

#[test]
fn probe_runs_with_a_cleared_environment() {
    assert!(
        std::env::var_os("HOME").is_some(),
        "o teste precisa de HOME no ambiente do processo"
    );
    let body = r#"if [ -n "${HOME:-}" ] || [ -n "${NODE_OPTIONS:-}" ]; then exit 7; fi
echo '{"node":"26.11.1","v8":"14.6.202.34-node.37","icu":"78.3","unicode":"17.0","tz":"2026e","cldr":"48.0"}'"#;
    let tuple = probe_node(&fake_node("clean-env", body)).unwrap();
    assert_eq!(tuple, committed_lock().reference);
}

#[test]
fn resolve_binary_reads_the_role_env_var() {
    let env = |name: &str| (name == ENV_NODE_SEED).then(|| OsString::from("/opt/node22/bin/node"));
    assert_eq!(
        resolve_binary(NodeRole::Seed, env).unwrap(),
        Path::new("/opt/node22/bin/node")
    );
    let error = resolve_binary(NodeRole::Reference, env).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5002");
    assert!(error.to_string().contains(ENV_NODE_REFERENCE));
}

#[test]
fn resolve_binary_treats_an_empty_value_as_unset() {
    let error = resolve_binary(NodeRole::Reference, |_| Some(OsString::new())).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5002");
}

#[test]
fn probe_rejects_relative_missing_and_failing_binaries() {
    let relative = probe_node(Path::new("node")).unwrap_err();
    assert_eq!(relative.stable_code(), "TYRUS-E5003");
    let missing = probe_node(Path::new("/nao/existe/node")).unwrap_err();
    assert_eq!(missing.stable_code(), "TYRUS-E5003");
    let failing = probe_node(&fake_node("exit1", "exit 1")).unwrap_err();
    assert_eq!(failing.stable_code(), "TYRUS-E5003");
}

#[test]
fn probe_kills_a_binary_that_exceeds_the_timeout() {
    let slow = fake_node("slow", "exec sleep 30");
    let started = std::time::Instant::now();
    let error = probe_node_with_timeout(&slow, Duration::from_millis(300)).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5003");
    assert!(started.elapsed() < Duration::from_secs(5));
    assert!(error.to_string().contains("tempo"), "{error}");
}

#[test]
fn probe_output_must_be_the_exact_tuple_json() {
    let garbage = probe_node(&fake_node("garbage", "echo nao-e-json")).unwrap_err();
    assert_eq!(garbage.stable_code(), "TYRUS-E5004");
    let partial = parse_probe_output(r#"{"node":"26.11.1"}"#).unwrap_err();
    assert_eq!(partial.stable_code(), "TYRUS-E5004");
    let blank =
        r#"{"node":"26.11.1","v8":"","icu":"78.3","unicode":"17.0","tz":"2026e","cldr":"48.0"}"#;
    assert_eq!(
        parse_probe_output(blank).unwrap_err().stable_code(),
        "TYRUS-E5004"
    );
    let extra = r#"{"node":"1","v8":"1","icu":"1","unicode":"1","tz":"1","cldr":"1","x":1}"#;
    assert!(parse_probe_output(extra).is_err());
}
