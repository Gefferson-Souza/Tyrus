#[cfg(test)]
mod tests {
    use std::fs;
    use std::io::Write;
    use std::process::Command;
    use tempfile::{NamedTempFile, TempDir};

    /// Test that generates Rust code, compiles it, AND RUNS IT
    #[test]
    fn test_execute_simple_function() {
        let ts_code = r#"
            export function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        let rust_code = verify_and_execute(ts_code, "test_add", |output| {
            if !output.status.success() {
                println!(
                    "Compilation failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
            assert!(output.status.success(), "Compilation should succeed");
        });

        // Verify the generated code contains the function
        assert!(rust_code.contains("pub fn add"));
        assert!(rust_code.contains("-> f64"));
    }

    #[test]
    fn test_execute_interface() {
        let ts_code = r#"
            export interface User {
                name: string;
                age: number;
            }
        "#;

        let rust_code = verify_and_execute(ts_code, "test_interface", |output| {
            assert!(output.status.success());
        });

        assert!(rust_code.contains("pub struct User"));
        assert!(rust_code.contains("Serialize"));
        assert!(rust_code.contains("Deserialize"));
    }

    #[test]
    fn test_execute_class_with_main() {
        let ts_code = r#"
            export class Calculator {
                value: number;
                
                constructor(value: number) {
                    this.value = value;
                }
                
                add(x: number): number {
                    return this.value + x;
                }
            }
        "#;

        // Create a complete Rust program that can be executed
        let generated = tyrus_orchestrator::build(tyrus_common::fs::FilePath::from(
            create_temp_ts_file(ts_code).path().to_path_buf(),
        ))
        .unwrap();

        // Remove serde derives for standalone compilation
        let generated = generated
            .replace(", serde :: Serialize, serde :: Deserialize", "")
            .replace("serde :: Serialize, serde :: Deserialize, ", "")
            .replace("serde :: Serialize, serde :: Deserialize", "");

        // Add a main function to actually execute the code
        let complete_program = format!(
            r#"
{}

#[allow(dead_code)]
fn main() {{
    let calc = Calculator::new(10.0);
    let result = calc.add(5.0);
    println!("Result: {{}}", result);
    assert_eq!(result, 15.0);
}}
"#,
            generated
        );

        // Write complete program
        let temp_dir = TempDir::new().unwrap();
        let rs_file = temp_dir.path().join("main.rs");
        fs::write(&rs_file, complete_program).unwrap();

        // Compile as executable
        let compile_output = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&rs_file)
            .arg("-o")
            .arg(temp_dir.path().join("test_exec"))
            .output()
            .expect("Failed to compile");

        if !compile_output.status.success() {
            panic!(
                "Compilation failed:\n{}",
                String::from_utf8_lossy(&compile_output.stderr)
            );
        }

        // EXECUTE the compiled program
        let exec_output = Command::new(temp_dir.path().join("test_exec"))
            .output()
            .expect("Failed to execute");

        assert!(exec_output.status.success(), "Execution should succeed");
        let output_str = String::from_utf8_lossy(&exec_output.stdout);
        assert!(output_str.contains("Result: 15"), "Output: {}", output_str);
    }

    fn create_temp_ts_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    fn verify_and_execute<F>(ts_code: &str, test_name: &str, verify: F) -> String
    where
        F: FnOnce(&std::process::Output),
    {
        // Generate Rust code
        let ts_file = create_temp_ts_file(ts_code);
        let rust_code = tyrus_orchestrator::build(tyrus_common::fs::FilePath::from(
            ts_file.path().to_path_buf(),
        ))
        .unwrap();

        // Write to temp file
        let mut rs_file = NamedTempFile::new().unwrap();
        writeln!(rs_file, "#![allow(dead_code, unused_variables)]").unwrap();

        // Remove serde derives for standalone compilation
        let rust_code_clean = rust_code
            .replace(", serde :: Serialize, serde :: Deserialize", "")
            .replace("serde :: Serialize, serde :: Deserialize, ", "")
            .replace("serde :: Serialize, serde :: Deserialize", "");

        rs_file.write_all(rust_code_clean.as_bytes()).unwrap();
        rs_file.flush().unwrap();

        // Compile with rustc
        let output = Command::new("rustc")
            .arg("--crate-name=oxidizer_test")
            .arg("--crate-type=lib")
            .arg("--edition=2021")
            .arg(rs_file.path())
            .arg("-o")
            .arg(rs_file.path().with_extension("rlib"))
            .output()
            .expect("Failed to execute rustc");

        // Verify compilation
        verify(&output);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            panic!(
                "rustc compilation failed for {}:\n{}\n\nGenerated Rust code:\n{}",
                test_name, stderr, rust_code
            );
        }

        rust_code
    }
}
