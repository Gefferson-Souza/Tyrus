use std::path::PathBuf;
use std::process::Command;
use tempfile::TempDir;

pub fn assert_behavior(fixture_path: &str) {
    println!("CWD: {:?}", std::env::current_dir());
    let fixture_path = PathBuf::from(fixture_path);
    let absolute_fixture_path = std::fs::canonicalize(&fixture_path)
        .unwrap_or_else(|_| panic!("Fixture not found: {:?}", fixture_path));

    // Step A: TypeScript Execution
    println!("Running TypeScript fixture: {:?}", absolute_fixture_path);
    let ts_output = Command::new("npx")
        .arg("ts-node")
        .arg("--compiler-options")
        .arg("{\"module\":\"commonjs\"}")
        .arg(&absolute_fixture_path)
        .output()
        .expect("Failed to execute ts-node");

    if !ts_output.status.success() {
        let stderr = String::from_utf8_lossy(&ts_output.stderr);
        panic!("TypeScript execution failed:\n{}", stderr);
    }

    let ts_stdout = String::from_utf8_lossy(&ts_output.stdout)
        .trim()
        .to_string();
    println!("TypeScript Output:\n{}", ts_stdout);

    // Step B: TypeRust Compilation
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let output_dir = temp_dir.path().join("output");
    let input_dir = temp_dir.path().join("input");
    std::fs::create_dir_all(&input_dir).expect("Failed to create input dir");

    // Copy fixture to input/index.ts so it's treated as the main entry point
    let input_file = input_dir.join("index.ts");
    std::fs::copy(&absolute_fixture_path, &input_file).expect("Failed to copy fixture");

    println!("Compiling to Rust in: {:?}", output_dir);
    // We use build_project to generate a full cargo project
    ox_orchestrator::build_project(input_dir, output_dir.clone())
        .expect("TypeRust compilation failed");

    // Step C: Rust Execution
    println!("Running Rust generated code...");
    let rust_output = Command::new("cargo")
        .arg("run")
        .arg("-q")
        .current_dir(&output_dir)
        .output()
        .expect("Failed to execute cargo run");

    if !rust_output.status.success() {
        let stderr = String::from_utf8_lossy(&rust_output.stderr);
        panic!("Rust execution failed:\n{}", stderr);
    }

    let rust_stdout = String::from_utf8_lossy(&rust_output.stdout)
        .trim()
        .to_string();
    println!("Rust Output:\n{}", rust_stdout);

    // Step D: Assertion
    assert_eq!(
        ts_stdout, rust_stdout,
        "Behavior mismatch for {:?}",
        fixture_path
    );
}
