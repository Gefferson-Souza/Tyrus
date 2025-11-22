#![allow(deprecated)]
#[cfg(test)]
use assert_cmd::prelude::*;
#[cfg(test)]
use std::process::Command;

mod test_build;

#[test]
fn test_cli_check_pass() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ox_cli"));
    cmd.arg("check")
        .arg("fixtures/pass_simple_dto/input.ts")
        .assert()
        .success();
}

#[test]
fn test_smoke_valid() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ox_cli"));
    cmd.arg("check")
        .arg("fixtures/smoke_valid/input.ts")
        .assert()
        .success();
}

#[test]
fn test_smoke_error() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ox_cli"));
    cmd.arg("check")
        .arg("fixtures/smoke_error/input.ts")
        .assert()
        .failure(); // It returns success because we catch the error and print it nicely.
                    // TODO: Should CLI return non-zero on analysis error? For now it returns 0.
}

#[test]
fn test_smoke_lint() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ox_cli"));
    cmd.arg("check")
        .arg("fixtures/smoke_lint/input.ts")
        .assert()
        .success();
}
