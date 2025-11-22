#![allow(deprecated)]
#[cfg(test)]
use std::process::Command;

#[test]
fn test_build_dto() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("ox_cli"));
    let output = cmd
        .arg("build")
        .arg("fixtures/pass_simple_dto/input.ts")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("pub struct User"));
    assert!(stdout.contains("pub name: String"));
    assert!(stdout.contains("pub age: f64"));

    insta::assert_snapshot!(stdout);
}
