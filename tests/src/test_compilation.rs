#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;
    use tyrus_test_utils::assert_rust_compiles;

    fn create_temp_ts_file(content: &str) -> NamedTempFile {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(content.as_bytes()).unwrap();
        file
    }

    #[test]
    fn test_compile_simple_function() {
        let ts_code = r#"
            export function add(a: number, b: number): number {
                return a + b;
            }
        "#;

        let ts_file = create_temp_ts_file(ts_code);
        let rust_code = tyrus_orchestrator::build(tyrus_common::fs::FilePath::from(
            ts_file.path().to_path_buf(),
        ))
        .unwrap();

        // Check if it compiles with Cargo (dependencies included)
        assert_rust_compiles(&rust_code);

        // Basic string checks
        assert!(rust_code.contains("pub fn add"));
        assert!(rust_code.contains("-> f64"));
    }

    #[test]
    fn test_compile_interface() {
        let ts_code = r#"
            export interface User {
                name: string;
                age: number;
            }
        "#;

        let ts_file = create_temp_ts_file(ts_code);
        let rust_code = tyrus_orchestrator::build(tyrus_common::fs::FilePath::from(
            ts_file.path().to_path_buf(),
        ))
        .unwrap();

        // Check if it compiles (Serde is handled by test_utils cargo project)
        assert_rust_compiles(&rust_code);

        assert!(rust_code.contains("pub struct User"));
        assert!(rust_code.contains("Serialize"));
        assert!(rust_code.contains("Deserialize"));
    }

    #[test]
    fn test_compile_class() {
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

        let ts_file = create_temp_ts_file(ts_code);
        let rust_code = tyrus_orchestrator::build(tyrus_common::fs::FilePath::from(
            ts_file.path().to_path_buf(),
        ))
        .unwrap();

        assert_rust_compiles(&rust_code);
    }
}
