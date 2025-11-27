#[cfg(test)]
mod generic_tests {
    use ox_common::fs::FilePath;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_compile_and_execute_generics() {
        let ts_code = r#"
            export interface Box<T> {
                value: T;
            }

            export class Wrapper<T> {
                value: T;
                constructor(value: T) {
                    this.value = value;
                }
                getValue(): T {
                    return this.value;
                }
            }

            export function identity<T>(arg: T): T {
                return arg;
            }
        "#;

        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("generics.ts");
        fs::write(&ts_file, ts_code).unwrap();

        let rust_code = ox_orchestrator::build(FilePath::from(ts_file)).unwrap();

        // Remove serde for standalone test (since we don't link against serde in this simple rustc invocation)
        let rust_code = rust_code
            .replace(", serde :: Serialize, serde :: Deserialize", "")
            .replace("serde :: Serialize, serde :: Deserialize, ", "")
            .replace("serde :: Serialize, serde :: Deserialize", "");

        let program = format!(
            r#"
            {}
            
            fn main() {{
                // Test Interface
                let b: Box<f64> = Box {{ value: 10.5 }};
                println!("Box value: {{}}", b.value);
                assert_eq!(b.value, 10.5);

                // Test Class
                // Note: In Rust we might need to specify type if inference isn't enough, 
                // but usually Wrapper::new(...) works if args are typed.
                // Let's be explicit: Wrapper::<String>::new(...)
                let w = Wrapper::<String>::new(String::from("hello generic"));
                println!("Wrapper value: {{}}", w.get_value());
                assert_eq!(w.get_value(), "hello generic");

                // Test Function
                let i = identity::<bool>(true);
                println!("Identity value: {{}}", i);
                assert!(i);
                
                println!("✅ Generics Test Passed!");
            }}
            "#,
            rust_code
        );

        let src_file = temp_dir.path().join("main.rs");
        fs::write(&src_file, program).unwrap();

        let exe_path = temp_dir.path().join("test_exec");
        let compile = Command::new("rustc")
            .arg("--edition=2021")
            .arg(&src_file)
            .arg("-o")
            .arg(&exe_path)
            .output()
            .expect("Failed to compile");

        if !compile.status.success() {
            println!(
                "Compilation failed:\n{}",
                String::from_utf8_lossy(&compile.stderr)
            );
            println!("Generated Code:\n{}", rust_code);
        }
        assert!(compile.status.success(), "Compilation failed");

        let exec = Command::new(&exe_path).output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&exec.stdout);
        println!("Output:\n{}", stdout);

        assert!(stdout.contains("✅ Generics Test Passed!"));
    }
}
