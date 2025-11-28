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

    // Overwrite main.rs to call the generated main function (from index.ts)
    // instead of the default server main.rs
    let main_rs = output_dir.join("src").join("main.rs");
    let custom_main = r#"
fn main() {
    // Call the main function generated from index.ts
    // It should be exposed in the library root
    typerust_app::main();
}
"#;
    std::fs::write(main_rs, custom_main).expect("Failed to overwrite main.rs");

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
    // Try to compare as JSON first
    if let (Ok(ts_json), Ok(rust_json)) = (
        serde_json::from_str::<serde_json::Value>(&ts_stdout),
        serde_json::from_str::<serde_json::Value>(&rust_stdout),
    ) {
        if !json_equal(&ts_json, &rust_json) {
            panic!(
                "Behavior mismatch for {:?}\nLeft (TS): {}\nRight (Rust): {}",
                fixture_path, ts_stdout, rust_stdout
            );
        }
    } else {
        // Fallback to string comparison
        assert_eq!(
            ts_stdout, rust_stdout,
            "Behavior mismatch for {:?}",
            fixture_path
        );
    }
}

fn json_equal(a: &serde_json::Value, b: &serde_json::Value) -> bool {
    match (a, b) {
        (serde_json::Value::Number(n1), serde_json::Value::Number(n2)) => {
            // Compare numbers loosely (f64 vs i64)
            let f1 = n1.as_f64();
            let f2 = n2.as_f64();
            match (f1, f2) {
                (Some(v1), Some(v2)) => (v1 - v2).abs() < f64::EPSILON,
                _ => n1 == n2,
            }
        }
        (serde_json::Value::Object(o1), serde_json::Value::Object(o2)) => {
            if o1.len() != o2.len() {
                return false;
            }
            for (k, v1) in o1 {
                if let Some(v2) = o2.get(k) {
                    if !json_equal(v1, v2) {
                        return false;
                    }
                } else {
                    return false;
                }
            }
            true
        }
        (serde_json::Value::Array(a1), serde_json::Value::Array(a2)) => {
            if a1.len() != a2.len() {
                return false;
            }
            a1.iter().zip(a2.iter()).all(|(v1, v2)| json_equal(v1, v2))
        }
        _ => a == b,
    }
}
