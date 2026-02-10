#[cfg(test)]
mod stdlib_tests {
    use tyrus_common::fs::FilePath;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_stdlib_math_execution() {
        let ts_code = r#"
            function testMath(): number {
                const max = Math.max(10.0, 20.0);
                const min = Math.min(5.0, 15.0);
                const rounded = Math.round(3.7);
                return max + min + rounded;
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let ts_file = temp_dir.join("stdlib_math_test.ts");
        std::fs::write(&ts_file, ts_code).unwrap();

        let rust_code =
            tyrus_orchestrator::build(FilePath::from(ts_file)).expect("Failed to generate Rust code");

        println!("Generated Rust code:\n{}", rust_code);

        let program = format!(
            r#"
{}

fn main() {{
    let result = test_math();
    println!("testMath() = {{}}", result);
    assert_eq!(result, 29.0, "Expected 20.0 + 5.0 + 4.0 = 29.0");
    println!("✅ Math stdlib test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "Math stdlib");
    }

    #[test]
    fn test_stdlib_string_execution() {
        let ts_code = r#"
            function testString(input: string): string {
                const upper = input.toUpperCase();
                return upper;
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let ts_file = temp_dir.join("stdlib_string_test.ts");
        std::fs::write(&ts_file, ts_code).unwrap();

        let rust_code =
            tyrus_orchestrator::build(FilePath::from(ts_file)).expect("Failed to generate Rust code");

        println!("Generated Rust code:\n{}", rust_code);

        let program = format!(
            r#"
{}

fn main() {{
    let result = test_string("hello".to_string());
    println!("testString('hello') = {{}}", result);
    assert_eq!(result, "HELLO", "Expected uppercase");
    println!("✅ String stdlib test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "String stdlib");
    }

    #[test]
    fn test_stdlib_console() {
        let ts_code = r#"
            function testConsole(): void {
                console.log("Test message");
                console.error("Error message");
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let ts_file = temp_dir.join("stdlib_console_test.ts");
        std::fs::write(&ts_file, ts_code).unwrap();

        let rust_code =
            tyrus_orchestrator::build(FilePath::from(ts_file)).expect("Failed to generate Rust code");

        println!("Generated Rust code:\n{}", rust_code);

        // Verify generated code contains println! and eprintln!
        assert!(rust_code.contains("println!"), "Should contain println!");
        assert!(rust_code.contains("eprintln!"), "Should contain eprintln!");

        println!("✅ Console stdlib mapping verified!");
    }

    fn execute_rust_program(program: &str, test_name: &str) {
        let temp_dir = TempDir::new().unwrap();
        let src_file = temp_dir.path().join("main.rs");
        fs::write(&src_file, program).unwrap();

        // Compile
        let exe_path = temp_dir.path().join("test_exec");
        let compile = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&src_file)
            .arg("-o")
            .arg(&exe_path)
            .output()
            .expect("Failed to compile");

        assert!(
            compile.status.success(),
            "{} - Compilation failed:\n{}",
            test_name,
            String::from_utf8_lossy(&compile.stderr)
        );

        // EXECUTE
        let exec = Command::new(&exe_path).output().expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&exec.stdout);
        let stderr = String::from_utf8_lossy(&exec.stderr);

        println!("{} output:", test_name);
        println!("{}", stdout);

        if !stderr.is_empty() {
            println!("stderr: {}", stderr);
        }

        assert!(
            exec.status.success(),
            "{} - Execution failed:\n{}",
            test_name,
            stderr
        );

        assert!(
            stdout.contains("✅"),
            "{} - Missing success marker",
            test_name
        );
    }
}
