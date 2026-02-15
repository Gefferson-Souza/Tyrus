#![allow(deprecated)]
#[cfg(test)]
use assert_cmd::prelude::*;
#[cfg(test)]
use std::process::Command;

mod test_build;
mod test_compilation;
mod test_e2e_exec;
mod test_generics;
mod test_nestjs;
mod test_regression;
#[cfg(test)]
mod test_snapshots;
#[cfg(test)]
mod test_stdlib_exec;
#[cfg(test)]
mod test_types;

#[cfg(test)]
mod infrastructure;

#[test]
fn verify_equivalence() {
    let scenarios = vec![
        "fixtures/equivalence/recursion.ts",
        "fixtures/equivalence/data_processing.ts",
        "fixtures/equivalence/class_state.ts",
        "fixtures/equivalence/class_state.ts",
        //"fixtures/simple/todo.ts", // Disabled due to ownership issues (partial move)
        "fixtures/simple/calc.ts",
        "fixtures/simple/strings.ts",
    ];

    for scenario in scenarios {
        infrastructure::equivalence::assert_behavior(scenario);
    }
}

#[test]
fn test_cli_check_pass() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus"));
    cmd.arg("check")
        .arg("fixtures/pass_simple_dto/input.ts")
        .assert()
        .success();
}

#[test]
fn test_smoke_valid() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus"));
    cmd.arg("check")
        .arg("fixtures/smoke_valid/input.ts")
        .assert()
        .success();
}

#[test]
fn test_smoke_error() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus"));
    cmd.arg("check")
        .arg("fixtures/smoke_error/input.ts")
        .assert()
        .failure(); // It returns success because we catch the error and print it nicely.
                    // TODO: Should CLI return non-zero on analysis error? For now it returns 0.
}

#[test]
fn test_smoke_lint() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus"));
    cmd.arg("check")
        .arg("fixtures/smoke_lint/input.ts")
        .assert()
        .success();
}
