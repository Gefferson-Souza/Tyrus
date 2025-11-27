#[cfg(test)]
mod nestjs_tests {
    use ox_common::fs::FilePath;
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_nestjs_controller_generation() {
        let ts_path = PathBuf::from("fixtures/nestjs_controller/src/cats.controller.ts");
        let rust_code =
            ox_orchestrator::build(FilePath::from(ts_path)).expect("Failed to generate Rust code");

        println!("Generated Rust Code:\n{}", rust_code);

        // Verify Axum handlers
        assert!(rust_code.contains("pub async fn find_all"));
        assert!(rust_code.contains("-> String"));
        assert!(rust_code.contains("Route:"));
        assert!(rust_code.contains("GET"));

        assert!(rust_code.contains("pub async fn create"));
        assert!(rust_code.contains("axum :: Json (create_cat_dto)"));
        assert!(rust_code.contains("axum :: Json < CreateCatDto >"));
        assert!(rust_code.contains("-> axum :: Json < CreateCatDto >"));
        assert!(rust_code.contains("POST"));
        assert!(rust_code.contains("return axum :: Json (create_cat_dto)"));
    }

    #[test]
    fn test_cargo_toml_generation() {
        let temp_dir = TempDir::new().unwrap();
        let input_dir = PathBuf::from("fixtures/nestjs_controller/src");
        let output_dir = temp_dir.path().to_path_buf();

        ox_orchestrator::build_project(input_dir, output_dir.clone())
            .expect("Failed to build project");

        let cargo_toml_path = output_dir.join("Cargo.toml");
        assert!(cargo_toml_path.exists());

        let content = std::fs::read_to_string(cargo_toml_path).unwrap();
        assert!(content.contains("[dependencies]"));
        assert!(content.contains("axum = \"0.7\""));
        assert!(content.contains("tokio = { version = \"1.0\", features = [\"full\"] }"));
    }
}
