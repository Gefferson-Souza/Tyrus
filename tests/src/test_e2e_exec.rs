#[cfg(test)]
mod executable_tests {
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use tempfile::TempDir;
    use tyrus_common::fs::FilePath;

    #[test]
    fn test_compile_and_execute_simple_functions() {
        // Transpile TypeScript to Rust
        let ts_path = PathBuf::from("fixtures/executable_simple/input.ts");
        let rust_code = tyrus_orchestrator::build(FilePath::from(ts_path))
            .expect("Failed to generate Rust code");

        // Create complete executable program
        let program = format!(
            r#"
{}

fn main() {{
    let result1 = add(5.0, 3.0);
    println!("add(5, 3) = {{}}", result1);
    assert_eq!(result1, 8.0);
    
    let result2 = multiply(4.0, 7.0);
    println!("multiply(4, 7) = {{}}", result2);
    assert_eq!(result2, 28.0);
    
    let result3 = calculate_total2(10.0, 20.0, 30.0);
    println!("calculate_total2(10, 20, 30) = {{}}", result3);
    assert_eq!(result3, 60.0);
    
    println!("✅ All tests passed!");
}}
"#,
            rust_code
        );

        // Compile and execute
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
            "Compilation failed:\n{}",
            String::from_utf8_lossy(&compile.stderr)
        );

        // EXECUTE
        let exec = Command::new(&exe_path).output().expect("Failed to execute");

        let stdout = String::from_utf8_lossy(&exec.stdout);
        println!("Execution output:\n{}", stdout);

        assert!(
            exec.status.success(),
            "Execution failed:\n{}",
            String::from_utf8_lossy(&exec.stderr)
        );

        assert!(stdout.contains("✅ All tests passed!"));
    }

    #[test]
    fn test_compile_and_execute_class() {
        // Transpile class
        let ts_path = PathBuf::from("fixtures/exec_class/input.ts");
        let mut rust_code = tyrus_orchestrator::build(FilePath::from(ts_path))
            .expect("Failed to generate Rust code");

        // Remove serde derives for standalone compilation
        rust_code = rust_code.replace(", serde :: Serialize, serde :: Deserialize", "");
        rust_code = rust_code.replace("serde :: Serialize, serde :: Deserialize, ", "");
        rust_code = rust_code.replace("serde :: Serialize, serde :: Deserialize", "");
        // Remove serde attributes (rename_all, etc.) that require the serde crate
        rust_code = remove_serde_attributes(&rust_code);

        // Create executable with class instantiation and method calls
        let program = format!(
            r#"
{}

fn main() {{
    let calc = Calculator::new(10.0);
    
    let result1 = calc.add(5.0);
    println!("calc.add(5.0) = {{}}", result1);
    assert_eq!(result1, 15.0, "Expected add to return 15.0");
    
    let result2 = calc.multiply(3.0);
    println!("calc.multiply(3.0) = {{}}", result2);
    assert_eq!(result2, 30.0, "Expected multiply to return 30.0");
    
    let value = calc.get_value();
    println!("calc.get_value() = {{}}", value);
    assert_eq!(value, 10.0, "Expected get_value to return 10.0");
    
    println!("✅ Class execution test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "Class execution");
    }

    #[test]
    fn test_compile_and_execute_integration() {
        // Complex integration test combining functions and logic
        let ts_code = r#"
            function square(x: number): number {
                return x * x;
            }
            
            function sumSquares(a: number, b: number): number {
                return square(a) + square(b);
            }
            
            function pythagoras(a: number, b: number): number {
                return sumSquares(a, b);
            }
        "#;

        let temp_dir = std::env::temp_dir();
        let ts_file = temp_dir.join("integration_test.ts");
        std::fs::write(&ts_file, ts_code).unwrap();

        let rust_code = tyrus_orchestrator::build(FilePath::from(ts_file))
            .expect("Failed to generate Rust code");

        let program = format!(
            r#"
{}

fn main() {{
    let result = pythagoras(3.0, 4.0);
    println!("pythagoras(3, 4) = {{}}", result);
    assert_eq!(result, 25.0, "Expected 3² + 4² = 25");
    
    let result2 = sum_squares(5.0, 12.0);  
    println!("sum_squares(5, 12) = {{}}", result2);
    assert_eq!(result2, 169.0, "Expected 5² + 12² = 169");
    
    println!("✅ Integration test passed!");
}}
"#,
            rust_code
        );

        execute_rust_program(&program, "Integration");
    }

    #[test]
    fn test_compile_and_execute_multi_file_project() {
        // This test simulates a real multi-file project
        // 1. math.ts: Exports functions
        // 2. models.ts: Exports interface/class
        // 3. main.ts: Imports and uses them

        let temp_dir = TempDir::new().unwrap();

        // --- 1. math.ts ---
        let math_ts = r#"
            export function add(a: number, b: number): number {
                return a + b;
            }
            export function sub(a: number, b: number): number {
                return a - b;
            }
        "#;
        let math_ts_path = temp_dir.path().join("math.ts");
        fs::write(&math_ts_path, math_ts).unwrap();
        let math_rs = tyrus_orchestrator::build(FilePath::from(math_ts_path)).unwrap();
        fs::write(temp_dir.path().join("math.rs"), math_rs).unwrap();

        // --- 2. models.ts ---
        let models_ts = r#"
            export class User {
                name: string;
                age: number;
                
                constructor(name: string, age: number) {
                    this.name = name;
                    this.age = age;
                }
                
                greet(): void {
                    console.log("Hello,", this.name);
                }
            }
        "#;
        let models_ts_path = temp_dir.path().join("models.ts");
        fs::write(&models_ts_path, models_ts).unwrap();
        let models_rs = tyrus_orchestrator::build(FilePath::from(models_ts_path))
            .unwrap()
            .replace(", serde :: Serialize, serde :: Deserialize", "")
            .replace("serde :: Serialize, serde :: Deserialize, ", "")
            .replace("serde :: Serialize, serde :: Deserialize", "");
        let models_rs = remove_serde_attributes(&models_rs);
        fs::write(temp_dir.path().join("models.rs"), models_rs).unwrap();

        // --- 3. main.ts ---
        let main_ts = r#"
            import { add, sub } from './math';
            import { User } from './models';

            export function main_logic(): void {
                let sum = add(10, 5);
                let diff = sub(10, 5);
                
                console.log("Sum:", sum);
                console.log("Diff:", diff);
                
                let user = new User("Alice", 30);
                user.greet();
                
                if (sum == 15 && diff == 5) {
                    console.log("✅ Multi-file test passed!");
                } else {
                    console.log("❌ Test failed");
                }
            }
        "#;
        let main_ts_path = temp_dir.path().join("main.ts");
        fs::write(&main_ts_path, main_ts).unwrap();
        let main_rs_body = tyrus_orchestrator::build(FilePath::from(main_ts_path)).unwrap();
        fs::write(temp_dir.path().join("main_module.rs"), main_rs_body).unwrap();

        // --- 4. Construct entry.rs (The Root) ---
        // This acts as the crate root (lib.rs/main.rs) that declares all modules as siblings.
        // This matches the structure generated by build_project (where mod.rs is the parent).

        let entry_rs = r#"
            mod math;
            mod models;
            mod main_module;
            
            fn main() {
                main_module::main_logic();
            }
        "#;

        let entry_rs_path = temp_dir.path().join("entry.rs");
        fs::write(&entry_rs_path, entry_rs).unwrap();

        // --- 5. Compile and Execute ---
        let exe_path = temp_dir.path().join("test_exec");
        let compile = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&entry_rs_path)
            .arg("-o")
            .arg(&exe_path)
            .output()
            .expect("Failed to compile");

        if !compile.status.success() {
            println!(
                "Compilation failed:\n{}",
                String::from_utf8_lossy(&compile.stderr)
            );
            // Print generated files for debugging
            println!(
                "--- math.rs ---\n{}",
                fs::read_to_string(temp_dir.path().join("math.rs")).unwrap()
            );
            println!(
                "--- models.rs ---\n{}",
                fs::read_to_string(temp_dir.path().join("models.rs")).unwrap()
            );
            println!(
                "--- main_module.rs ---\n{}",
                fs::read_to_string(temp_dir.path().join("main_module.rs")).unwrap()
            );
        }
        assert!(compile.status.success(), "Compilation failed");

        let exec = Command::new(&exe_path).output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&exec.stdout);
        println!("Execution output:\n{}", stdout);

        assert!(stdout.contains("Sum: 15"));
        assert!(stdout.contains("Diff: 5"));
        assert!(stdout.contains("Hello, Alice"));
        assert!(stdout.contains("✅ Multi-file test passed!"));
    }

    fn execute_rust_program(program: &str, test_name: &str) {
        {
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
                {
                    println!("stderr: {}", stderr);
                }
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

    /// Remove `#[serde(...)]` attribute lines from generated Rust code.
    /// Needed for tests that compile with plain `rustc` (no serde crate).
    fn remove_serde_attributes(code: &str) -> String {
        code.lines()
            .filter(|line| !line.trim().starts_with("#[serde("))
            .collect::<Vec<_>>()
            .join("\n")
    }
}
