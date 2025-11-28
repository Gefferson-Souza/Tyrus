#[cfg(test)]
mod type_tests {
    use ox_common::fs::FilePath;
    use std::fs;
    use std::process::Command;
    use tempfile::TempDir;

    #[test]
    fn test_compile_and_execute_advanced_types() {
        let ts_code = r#"
            export type ID = string;
            export type NumberList = number[];

            export interface Container {
                id: ID;
                values: number[];
                optional?: string;
                maybe: number | undefined;
            }

            export function createContainer(id: ID, values: NumberList): Container {
                return {
                    id: id,
                    values: values,
                    optional: "present",
                    maybe: undefined
                }; // Object literal conversion is not fully supported yet in codegen for complex types?
                   // Wait, we haven't implemented ObjectLiteral -> Struct conversion in codegen yet!
                   // We only support `new Class()`.
                   // So we should use a class or just test type definitions.
            }
            
            export class ContainerClass {
                id: ID;
                values: number[];
                optional?: string;
                maybe: number | undefined;
                
                constructor(id: ID, values: number[]) {
                    this.id = id;
                    this.values = values;
                    this.optional = "present";
                    // this.maybe is undefined by default or we can set it
                }
            }
            
            export function processList(list: number[]): number {
                return 42;
            }
        "#;

        // We need to use a class because we don't support object literals returning structs yet (that's likely Phase 4 or 5).
        // But we can test the type definitions themselves.

        let temp_dir = TempDir::new().unwrap();
        let ts_file = temp_dir.path().join("types.ts");
        fs::write(&ts_file, ts_code).unwrap();

        let rust_code = ox_orchestrator::build(FilePath::from(ts_file)).unwrap();

        // Remove serde for standalone test
        let rust_code = rust_code
            .replace(", serde :: Serialize, serde :: Deserialize", "")
            .replace("serde :: Serialize, serde :: Deserialize, ", "")
            .replace("serde :: Serialize, serde :: Deserialize", "");

        let program = format!(
            r#"
            {}
            
            fn main() {{
                let list: NumberList = vec![1.0, 2.0, 3.0];
                let result = process_list(list.clone());
                println!("Result: {{}}", result);
                assert_eq!(result, 42.0);
                
                let container = ContainerClass::new(std::sync::Arc::new(String::from("123")), list);
                println!("Container ID: {{}}", container.id);
                assert_eq!(container.id.as_ref(), "123");
                assert_eq!(container.values.len(), 3);
                
                // Check optional fields
                if let Some(opt) = container.optional {{
                    println!("Optional: {{}}", opt);
                    assert_eq!(opt, "present");
                }} else {{
                    println!("Optional is None");
                }}
                
                if container.maybe.is_none() {{
                    println!("Maybe is None");
                }} else {{
                    panic!("Maybe should be None");
                }}
                
                println!("✅ Advanced Types Test Passed!");
            }}
            "#,
            rust_code
        );

        // Create Cargo.toml
        let cargo_toml = r#"
[package]
name = "test_types"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
"#;
        fs::write(temp_dir.path().join("Cargo.toml"), cargo_toml).unwrap();

        // Create src directory
        let src_dir = temp_dir.path().join("src");
        fs::create_dir(&src_dir).unwrap();
        let src_file = src_dir.join("main.rs");
        fs::write(&src_file, program).unwrap();

        let compile = Command::new("cargo")
            .arg("build")
            .current_dir(temp_dir.path())
            .output()
            .expect("Failed to compile");

        if !compile.status.success() {
            println!(
                "Compilation failed:\nStderr:\n{}\nStdout:\n{}",
                String::from_utf8_lossy(&compile.stderr),
                String::from_utf8_lossy(&compile.stdout)
            );
            println!("Generated Code:\n{}", rust_code);
        }
        assert!(compile.status.success(), "Compilation failed");

        let exe_path = temp_dir.path().join("target/debug/test_types");
        let exec = Command::new(&exe_path).output().expect("Failed to execute");
        let stdout = String::from_utf8_lossy(&exec.stdout);
        println!("Output:\n{}", stdout);

        assert!(stdout.contains("✅ Advanced Types Test Passed!"));
    }
}
