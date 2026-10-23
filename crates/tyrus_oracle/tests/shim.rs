#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};
use std::process::Command;

const SCRIPT: &str =
    "console.log(Date.now(), new Date().toISOString(), String(Date()).length > 0, \
Math.random(), Math.random(), process.pid, new Date(0).getTime(), typeof Date.parse)";

fn shim_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/shim/fix-env.js")
}

fn node() -> String {
    std::env::var("TYRUS_NODE_REFERENCE").expect("TYRUS_NODE_REFERENCE")
}

fn run(with_shim: bool) -> String {
    let mut command = Command::new(node());
    command.env_clear();
    if with_shim {
        command.arg("--require").arg(shim_path());
    }
    let output = command.arg("-e").arg(SCRIPT).output().unwrap();
    assert!(output.status.success(), "{output:?}");
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn shim_makes_clock_rng_and_pid_identical_across_runs() {
    let first = run(true);
    let second = run(true);
    assert_eq!(first, second);
    assert!(
        first.starts_with("1700000000000 2023-11-14T22:13:20.000Z true "),
        "{first}"
    );
    assert!(first.contains(" 4242 0 function"), "{first}");
}

#[test]
fn without_the_shim_the_environment_leaks() {
    let first = run(false);
    let second = run(false);
    assert_ne!(first, second, "relógio/RNG/pid reais deveriam variar");
}
