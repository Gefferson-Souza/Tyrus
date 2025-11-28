use ox_diagnostics::OxidizerError;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

use ox_common::fs::FilePath;

pub fn check(path: FilePath) -> Result<(), OxidizerError> {
    let program = ox_parser::parse(path.as_ref())?;

    // Read source code for error reporting
    let source_code = std::fs::read_to_string(path.as_ref()).map_err(OxidizerError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    let errors = ox_analyzer::Analyzer::analyze(&program, source_code, file_name);

    if !errors.is_empty() {
        for error in errors {
            println!("{:?}", miette::Report::new(error));
        }
        return Ok(()); // Or return Err if we want to stop execution
    }

    let count = match program {
        swc_ecma_ast::Program::Module(m) => m.body.len(),
        swc_ecma_ast::Program::Script(s) => s.body.len(),
    };
    println!("✅ AST parsed successfully with {} statements", count);
    Ok(())
}

pub fn pipeline() -> Result<(), OxidizerError> {
    // Stub implementation
    Ok(())
}

pub fn build(path: FilePath) -> Result<String, OxidizerError> {
    let program = ox_parser::parse(path.as_ref())?;
    // Default to false for single file build
    let generated_code = ox_codegen::generate(&program, false);
    format_code(generated_code.code)
}

pub fn build_project(input_dir: PathBuf, output_dir: PathBuf) -> Result<(), OxidizerError> {
    let mut controllers: Vec<(String, String)> = Vec::new();

    // 1. Walk and Transpile
    for entry in WalkDir::new(&input_dir) {
        let entry = entry.map_err(|e| OxidizerError::IoError(e.into()))?;
        let path = entry.path();

        // Calculate relative path to mirror structure
        let relative_path = path.strip_prefix(&input_dir).unwrap_or(path);
        let output_path = output_dir.join(relative_path);

        if path.is_dir() {
            fs::create_dir_all(&output_path).map_err(OxidizerError::IoError)?;
        } else if let Some(ext) = path.extension() {
            if ext == "ts" {
                // Transpile .ts file
                let program = ox_parser::parse(path)?;

                // Check if it's index.ts
                let is_index = path.file_stem().and_then(|s| s.to_str()) == Some("index");

                let generated = ox_codegen::generate(&program, is_index);
                let formatted_code = format_code(generated.code)?;

                // Change extension to .rs and sanitize filename
                // e.g. cats.controller.ts -> cats_controller.rs
                let file_stem = path.file_stem().unwrap().to_string_lossy();
                let sanitized_stem = file_stem.replace('.', "_");

                let output_file = output_path.with_file_name(format!("{}.rs", sanitized_stem));

                // Ensure parent dir exists (just in case)
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent).map_err(OxidizerError::IoError)?;
                }

                fs::write(output_file, formatted_code).map_err(OxidizerError::IoError)?;

                // Collect controllers
                if !generated.controllers.is_empty() {
                    // Construct module path
                    // e.g. relative_path = "cats/cats.controller.ts"
                    // parent = "cats"
                    // stem = "cats_controller"
                    // module = "crate::cats::cats_controller"

                    let mut module_parts = Vec::new();
                    module_parts.push("crate".to_string());

                    if let Some(parent) = relative_path.parent() {
                        for part in parent.components() {
                            if let std::path::Component::Normal(s) = part {
                                module_parts.push(s.to_string_lossy().to_string());
                            }
                        }
                    }
                    module_parts.push(sanitized_stem);

                    let module_path = module_parts.join("::");

                    for controller in generated.controllers {
                        controllers.push((module_path.clone(), controller.struct_name));
                    }
                }
            }
        }
    }

    // 2. Generate mod.rs (The Glue)
    // We walk the OUTPUT directory now to generate mod.rs based on what we created
    for entry in WalkDir::new(&output_dir) {
        let entry = entry.map_err(|e| OxidizerError::IoError(e.into()))?;
        let path = entry.path();

        if path.is_dir() {
            generate_mod_rs(path)?;
        }
    }

    // 3. Generate Cargo.toml
    generate_cargo_toml(&output_dir)?;

    // 4. Generate main.rs
    generate_main_rs(&output_dir, &controllers)?;

    Ok(())
}

fn generate_main_rs(
    output_dir: &Path,
    controllers: &[(String, String)],
) -> Result<(), OxidizerError> {
    let mut main_content = String::new();
    main_content.push_str("use axum::Router;\n");
    main_content.push_str("use tokio::net::TcpListener;\n\n");

    // We need to declare the modules here if main.rs is the crate root.
    // But usually lib.rs is the library root and main.rs is the binary root.
    // If we have both, main.rs can use the library.
    // But here we are generating a single crate that is both lib and bin?
    // Or main.rs includes mod.rs?
    // If we generate `lib.rs` (which we do by renaming mod.rs in the test), then `main.rs` can use `typerust_app::...`.
    // But `build_project` generates `mod.rs` in the root.
    // If we want `main.rs` to work, it should probably be `mod.rs` -> `lib.rs` and `main.rs` uses the lib.
    // OR `main.rs` declares `mod ...;`.

    // Let's assume `lib.rs` exists (created from `mod.rs` by the caller or we should do it here).
    // In `test_nestjs.rs`, we rename `mod.rs` to `lib.rs`.
    // So `main.rs` should use the crate name.
    // The crate name is defined in `Cargo.toml` as `typerust_app`.
    // So we should use `typerust_app::path::to::Controller`.
    // BUT, my module path construction used `crate::...`.
    // `crate::` refers to the current crate. If `main.rs` is in the same crate as `lib.rs`, it's tricky.
    // Usually `main.rs` and `lib.rs` are separate crate roots.
    // If `main.rs` uses `typerust_app`, it treats `lib.rs` as an external crate.

    // Let's change `crate::` to `typerust_app::` in the module path construction?
    // Or just use `typerust_app` prefix in `main.rs`.

    // Wait, if I use `crate::` in `main.rs`, it refers to `main.rs`'s module tree.
    // If `main.rs` does NOT declare modules, it can't see them via `crate::`.
    // So `main.rs` must use the library crate.

    // So I should replace `crate::` with `typerust_app::` when generating `main.rs`.

    main_content.push_str("#[tokio::main]\n");
    main_content.push_str("async fn main() {\n");
    main_content.push_str("    let app = Router::new()");

    for (module_path, struct_name) in controllers {
        // Replace crate:: with typerust_app::
        let lib_path = module_path.replacen("crate", "typerust_app", 1);
        main_content.push_str(&format!(
            "\n        .merge({}::{}::router())",
            lib_path, struct_name
        ));
    }
    main_content.push_str(";\n\n");

    main_content
        .push_str("    let listener = TcpListener::bind(\"0.0.0.0:3000\").await.unwrap();\n");
    main_content.push_str("    println!(\"Server running on http://0.0.0.0:3000\");\n");
    main_content.push_str("    axum::serve(listener, app).await.unwrap();\n");
    main_content.push_str("}\n");

    let main_path = output_dir.join("main.rs");
    fs::write(main_path, main_content).map_err(OxidizerError::IoError)?;
    Ok(())
}

fn generate_cargo_toml(output_dir: &Path) -> Result<(), OxidizerError> {
    let cargo_toml_content = r#"[package]
name = "typerust_app"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["trace"] }

[[bin]]
name = "server"
path = "src/main.rs"

[lib]
name = "typerust_app"
path = "src/lib.rs"
"#;

    let cargo_toml_path = output_dir.join("Cargo.toml");
    fs::write(cargo_toml_path, cargo_toml_content).map_err(OxidizerError::IoError)?;
    Ok(())
}

fn generate_mod_rs(dir: &Path) -> Result<(), OxidizerError> {
    let mut mod_content = String::new();
    let mut has_children = false;
    let mut index_content = String::new();

    // Read directory entries
    let entries = fs::read_dir(dir).map_err(OxidizerError::IoError)?;

    for entry in entries {
        let entry = entry.map_err(OxidizerError::IoError)?;
        let path = entry.path();

        // Skip mod.rs itself if it exists (though we are creating it)
        if path.file_name().and_then(|n| n.to_str()) == Some("mod.rs") {
            continue;
        }

        if path.is_dir() {
            // If it's a directory, it should have a mod.rs inside (handled by recursion/iteration),
            // so we expose it as a module.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let sanitized_name = name.replace('.', "_");
                mod_content.push_str(&format!("pub mod {};\n", sanitized_name));
                has_children = true;
            }
        } else if let Some(ext) = path.extension() {
            if ext == "rs" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == "index" {
                        // Capture index.rs content to append later
                        index_content =
                            fs::read_to_string(&path).map_err(OxidizerError::IoError)?;
                        // We will remove index.rs later or just ignore it in the final build?
                        // Usually we want to remove it so we don't have both mod.rs and index.rs
                        // But let's delete it after reading.
                    } else {
                        // Sanitize module name just in case, though we sanitized filename on write
                        let sanitized_stem = stem.replace('.', "_");
                        mod_content.push_str(&format!("pub mod {};\n", sanitized_stem));
                        has_children = true;
                    }
                }
            }
        }
    }

    // If we found index.rs, delete it and append its content
    let index_path = dir.join("index.rs");
    if index_path.exists() {
        fs::remove_file(index_path).map_err(OxidizerError::IoError)?;
    }

    // Append index content
    if !index_content.is_empty() {
        mod_content.push('\n');
        mod_content.push_str("// Content from index.ts\n");
        mod_content.push_str(&index_content);
        has_children = true;
    }

    // Only write mod.rs if there's something to put in it (or if it's the root output dir?)
    // Actually, Rust needs mod.rs to expose children. If a dir has children, it needs mod.rs.
    if has_children {
        let mod_path = dir.join("mod.rs");
        fs::write(mod_path, mod_content).map_err(OxidizerError::IoError)?;
    }

    Ok(())
}

fn format_code(code: String) -> Result<String, OxidizerError> {
    // Skip formatting for code containing async (edition compatibility)
    if code.contains("async fn") {
        return Ok(format!(
            "// Note: async/await code - formatting skipped for edition compatibility\n{}",
            code
        ));
    }

    use std::io::Write;
    use std::process::{Command, Stdio};

    let mut child = Command::new("rustfmt")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .map_err(OxidizerError::IoError)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(code.as_bytes())
            .map_err(OxidizerError::IoError)?;
    }

    let output = child.wait_with_output().map_err(OxidizerError::IoError)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(OxidizerError::FormattingError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}
