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
        assert!(rust_code.contains("-> Result < String , crate :: AppError >"));
        assert!(rust_code.contains("Route:"));
        assert!(rust_code.contains("GET"));

        assert!(rust_code.contains("pub async fn create"));
        assert!(rust_code.contains("axum :: Json (create_cat_dto)"));
        assert!(rust_code.contains("axum :: Json < CreateCatDto >"));
        assert!(
            rust_code.contains("-> Result < axum :: Json < CreateCatDto > , crate :: AppError >")
        );
        assert!(rust_code.contains("POST"));
        assert!(rust_code.contains("return Ok (axum :: Json (create_cat_dto . into ()))"));
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

    #[test]
    fn test_nestjs_project_compilation() {
        // This test actually tries to compile the generated project with cargo
        // It requires internet access to fetch dependencies (axum, tokio, etc.)
        // If this fails due to network, we might need to mock or skip it in CI.

        let _temp_dir = TempDir::new().unwrap();
        let _input_dir = PathBuf::from("fixtures/nestjs_controller/src");
        let temp_dir = TempDir::new().unwrap();
        let input_dir = PathBuf::from("fixtures/nestjs_controller/src");
        let output_dir = temp_dir.path().to_path_buf(); // Project root

        // Build project
        ox_orchestrator::build_project(input_dir, output_dir.clone())
            .expect("Failed to build project");

        // Verify error.rs exists in src
        let error_rs = output_dir.join("src").join("error.rs");
        assert!(error_rs.exists(), "error.rs was not generated");

        // Check Cargo.toml in root
        let root_cargo = output_dir.join("Cargo.toml");
        assert!(root_cargo.exists(), "Cargo.toml missing");

        // Rename root mod.rs to lib.rs to make it a library
        // build_project now handles this internally or generates lib.rs directly.
        // Let's check if lib.rs exists in src.
        let lib_rs = output_dir.join("src").join("lib.rs");
        let mod_rs = output_dir.join("src").join("mod.rs");

        if !lib_rs.exists() && mod_rs.exists() {
            std::fs::rename(mod_rs, lib_rs).expect("Failed to rename mod.rs");
        }

        // Run cargo build (stronger verification than check)
        let status = std::process::Command::new("cargo")
            .arg("build")
            .current_dir(output_dir)
            .status()
            .expect("Failed to run cargo build");

        assert!(status.success(), "Generated project failed to compile");
    }
}
