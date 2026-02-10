#![allow(deprecated)]
#[cfg(test)]
use std::process::Command;

#[test]
fn test_build_dto() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus_cli"));
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

#[test]
fn test_build_simple_fn() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus_cli"));
    let output = cmd
        .arg("build")
        .arg("fixtures/build_simple_fn/input.ts")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    assert!(stdout.contains("pub fn add"));
    assert!(stdout.contains("pub fn sub"));
    assert!(stdout.contains("a + &b") || stdout.contains("a + b") || stdout.contains("a + & b"));
    assert!(stdout.contains("x - y"));

    insta::assert_snapshot!(stdout);
}

#[test]
fn test_build_async_fn() {
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus_cli"));
    let output = cmd
        .arg("build")
        .arg("fixtures/build_async_fn/input.ts")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify async fn generation
    assert!(stdout.contains("async fn fetch_data"));
    assert!(stdout.contains("await") && (stdout.contains(".await") || stdout.contains(". await")));
    assert!(stdout.contains("pub async fn simple_call"));
}

#[test]
fn test_e2e_complex_scenario() {
    // This test validates ALL completed milestones working together:
    // - Milestone 2: Interfaces
    // - Milestone 3: Functions with math
    // - Milestone 4: Async/await
    let mut cmd = Command::new(assert_cmd::cargo::cargo_bin("tyrus_cli"));
    let output = cmd
        .arg("build")
        .arg("fixtures/e2e_complex/input.ts")
        .output()
        .expect("Failed to execute command");

    if !output.status.success() {
        println!("Build failed: {}", String::from_utf8_lossy(&output.stderr));
    }
    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();

    // Verify Milestone 2: Interfaces/DTOs
    assert!(stdout.contains("pub struct User"), "Missing User struct");
    assert!(
        stdout.contains("pub struct ApiResponse"),
        "Missing ApiResponse struct"
    );
    assert!(
        stdout.contains("id") && (stdout.contains("id: f64") || stdout.contains("id : f64")),
        "Missing User.id field"
    );
    assert!(
        stdout.contains("name")
            && (stdout.contains("name: String") || stdout.contains("name : String")),
        "Missing User.name field"
    );
    assert!(
        stdout.contains("isActive")
            && (stdout.contains("isActive: bool") || stdout.contains("isActive : bool")),
        "Missing User.isActive field"
    );

    // Verify Milestone 3: Functions with binary expressions
    assert!(
        stdout.contains("pub fn calculate_age"),
        "Missing calculate_age function"
    );
    assert!(stdout.contains("pub fn sum"), "Missing sum function");
    assert!(
        stdout.contains("current_year - birth_year") || stdout.contains("currentYear - birthYear"),
        "Missing math expression in calculate_age"
    );

    // Verify Milestone 4: Async/await
    assert!(
        stdout.contains("async fn fetch_user"),
        "Missing async fetch_user"
    );
    assert!(
        stdout.contains("async fn save_user"),
        "Missing async save_user"
    );
    assert!(
        stdout.contains("await") && (stdout.contains(".await") || stdout.contains(". await")),
        "Missing await expression"
    );

    // Verify mixed sync/async scenario
    assert!(
        stdout.contains("pub fn process_user"),
        "Missing sync process_user function"
    );
}
