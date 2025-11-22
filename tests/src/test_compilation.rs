use std::fs;
use std::process::Command as StdCommand;

// NOTE: Compilation tests disabled for now
// These should be implemented using the `trybuild` crate which properly manages
// dependencies like `serde`. See Guidelines.md Nível 3.
// TODO: Add in v1.1 using trybuild

/*
#[test]
fn test_compilation_dto() {
    // Compilation Test (Nível 3 das Guidelines)
    // Gera código Rust e compila com rustc
    let output = StdCommand::new(assert_cmd::cargo::cargo_bin("ox_cli"))
        .arg("build")
        .arg("fixtures/pass_simple_dto/input.ts")
        .output()
        .expect("Failed to run ox_cli");

    assert!(output.status.success());
    let generated_code = String::from_utf8(output.stdout).unwrap();

    // Prepend extern crate declarations for standalone compilation
    let compilable_code = format!(
        "extern crate serde;\nextern crate serde_json;\n\n{}",
        generated_code
    );

    // Save to temp file
    let temp_file = "/tmp/oxidizer_test_dto.rs";
    fs::write(temp_file, compilable_code).expect("Failed to write temp file");

    // Try to compile with rustc
    let rustc_output = StdCommand::new("rustc")
        .args(&["--crate-type", "lib", "--edition", "2021", temp_file])
        .arg("-o")
        .arg("/tmp/oxidizer_test_dto.rlib")
        .output()
        .expect("Failed to run rustc");

    if !rustc_output.status.success() {
        eprintln!(
            "RUSTC STDERR:\n{}",
            String::from_utf8_lossy(&rustc_output.stderr)
        );
        eprintln!(
            "RUSTC STDOUT:\n{}",
            String::from_utf8_lossy(&rustc_output.stdout)
        );
    }

    assert!(
        rustc_output.status.success(),
        "Generated DTO code failed to compile with rustc"
    );

    // Cleanup
    let _ = fs::remove_file(temp_file);
    let _ = fs::remove_file("/tmp/oxidizer_test_dto.rlib");
}

#[test]
fn test_compilation_functions() {
    let output = StdCommand::new(assert_cmd::cargo::cargo_bin("ox_cli"))
        .arg("build")
        .arg("fixtures/build_simple_fn/input.ts")
        .output()
        .expect("Failed to run ox_cli");

    assert!(output.status.success());
    let generated_code = String::from_utf8(output.stdout).unwrap();

    let compilable_code = format!(
        "extern crate serde;\nextern crate serde_json;\n\n{}",
        generated_code
    );

    let temp_file = "/tmp/oxidizer_test_fn.rs";
    fs::write(temp_file, compilable_code).expect("Failed to write temp file");

    let rustc_output = StdCommand::new("rustc")
        .args(&["--crate-type", "lib", "--edition", "2021", temp_file])
        .arg("-o")
        .arg("/tmp/oxidizer_test_fn.rlib")
        .output()
        .expect("Failed to run rustc");

    if !rustc_output.status.success() {
        eprintln!(
            "RUSTC STDERR:\n{}",
            String::from_utf8_lossy(&rustc_output.stderr)
        );
    }

    assert!(
        rustc_output.status.success(),
        "Generated function code failed to compile with rustc"
    );

    let _ = fs::remove_file(temp_file);
    let _ = fs::remove_file("/tmp/oxidizer_test_fn.rlib");
}
*/
