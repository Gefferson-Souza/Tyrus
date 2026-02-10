use std::fs;
use std::process::Command;
use tempfile::TempDir;

/// Asserts that the provided Rust code compiles successfully as a **library**.
///
/// This function:
/// 1. Creates a temporary Cargo project.
/// 2. Sets up `Cargo.toml` with common dependencies (serde, tokio, etc.).
/// 3. Writes the code to `src/lib.rs` (not main.rs — generated code has no `main`).
/// 4. Runs `cargo check`.
///
/// # Panics
/// Panics if `cargo check` fails, printing the full `rustc` error output.
pub fn assert_rust_compiles(code: &str) {
    let temp_dir = TempDir::new().expect("Failed to create temp dir");
    let project_path = temp_dir.path();

    let src_dir = project_path.join("src");
    fs::create_dir(&src_dir).expect("Failed to create src dir");

    // Cargo.toml — library crate with common dependencies
    let cargo_toml = r#"
[package]
name = "tyrus_app"
version = "0.1.0"
edition = "2021"

[lib]
name = "tyrus_app"
path = "src/lib.rs"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
reqwest = { version = "0.11", features = ["json", "rustls-tls"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["trace"] }
"#;

    fs::write(project_path.join("Cargo.toml"), cargo_toml).expect("Failed to write Cargo.toml");

    // Wrap code with common allows to suppress dead_code warnings
    let wrapped_code = format!(
        "#![allow(dead_code, unused_variables, unused_imports)]\n{}",
        code
    );

    fs::write(src_dir.join("lib.rs"), &wrapped_code).expect("Failed to write lib.rs");

    let output = Command::new("cargo")
        .arg("check")
        .current_dir(project_path)
        .output()
        .expect("Failed to execute cargo check");

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        panic!(
            "\n╔══════════════════════════════════════════╗\n\
             ║   RUST COMPILATION FAILED                ║\n\
             ╚══════════════════════════════════════════╝\n\n\
             CODE:\n------\n{}\n------\n\n\
             STDERR:\n{}\n\nSTDOUT:\n{}",
            code, stderr, stdout
        );
    }
}
