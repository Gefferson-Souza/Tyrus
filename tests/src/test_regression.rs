use std::path::PathBuf;
use std::process::Command;
use tyrus_orchestrator::build_project;

#[allow(dead_code)]
fn run_gauntlet_test(fixture_name: &str) {
    let input_path = PathBuf::from(format!("fixtures/{}", fixture_name));
    let output_dir = PathBuf::from(format!("fixtures/{}/dist", fixture_name));

    // Clean previous output
    if output_dir.exists() {
        std::fs::remove_dir_all(&output_dir).unwrap();
    }

    // Build Project
    println!("Building {}...", fixture_name);
    build_project(input_path, output_dir.clone()).expect("Build failed");

    // Verify Compilation
    println!("Verifying compilation for {}...", fixture_name);
    let status = Command::new("cargo")
        .arg("check")
        .current_dir(&output_dir)
        .status()
        .expect("Failed to run cargo check");

    assert!(status.success(), "Cargo check failed for {}", fixture_name);
}

#[test]
fn test_scenario_1_complex_single() {
    run_gauntlet_test("complex_single");
}

#[test]
fn test_scenario_2_complex_node() {
    run_gauntlet_test("complex_node");
}

#[test]
fn test_scenario_3_complex_nestjs() {
    run_gauntlet_test("complex_nestjs");
}
