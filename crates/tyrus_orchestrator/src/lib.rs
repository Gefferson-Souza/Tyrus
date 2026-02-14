use std::fs;
use std::path::{Path, PathBuf};
use tyrus_diagnostics::TyrusError;

use walkdir::WalkDir;

use tyrus_common::fs::FilePath;

pub fn check(path: FilePath) -> Result<(), TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;

    // Read source code for error reporting
    let source_code = std::fs::read_to_string(path.as_ref()).map_err(TyrusError::IoError)?;
    let file_name = path.as_ref().to_string_lossy().to_string();

    let errors = tyrus_analyzer::Analyzer::analyze(&program, source_code, file_name);

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

pub fn pipeline() -> Result<(), TyrusError> {
    // Stub implementation
    Ok(())
}

pub fn build(path: FilePath) -> Result<String, TyrusError> {
    let program = tyrus_parser::parse(path.as_ref())?;
    // Default to false for single file build
    let generated_code = tyrus_codegen::generate(&program, false);
    let mut code = generated_code.code;

    // Conditionally inject AppError boilerplate:
    // Only needed when async functions generate `Result<T, crate::AppError>` return types
    if code.contains("crate::AppError") {
        // Replace crate:: prefix with local reference since we're inlining the struct
        code = code.replace("crate::AppError", "AppError");
        code.push_str(get_app_error_code());
    } else if code.contains("crate :: AppError") {
        code = code.replace("crate :: AppError", "AppError");
        code.push('\n');
        code.push_str(get_app_error_code());
    }

    format_code(code)
}

pub fn build_project(input_dir: PathBuf, output_dir: PathBuf) -> Result<(), TyrusError> {
    let mut controllers: Vec<String> = Vec::new(); // Just names of controllers
    let mut class_module_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut generic_classes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut programs = Vec::new();
    let mut file_paths = Vec::new();

    // 1. Walk, Parse, and Collect Info
    for entry in WalkDir::new(&input_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ts") {
            let program = tyrus_parser::parse(path)?;

            // Calculate module path
            let relative_path = path.strip_prefix(&input_dir).unwrap_or(path);
            let file_stem = path
                .file_stem()
                .ok_or_else(|| {
                    TyrusError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid file path",
                    ))
                })?
                .to_string_lossy();
            let sanitized_stem = file_stem.replace(['.', '-'], "_");

            let mut module_parts = Vec::new();
            module_parts.push("tyrus_app".to_string()); // Use lib crate name

            if let Some(parent) = relative_path.parent() {
                for part in parent.components() {
                    if let std::path::Component::Normal(s) = part {
                        let s_str = s.to_string_lossy();
                        if s_str != "src" {
                            module_parts.push(s_str.replace('-', "_"));
                        }
                    }
                }
            }
            module_parts.push(sanitized_stem);
            let module_path = module_parts.join("::");

            // Extract classes to map them
            // We use a simple visitor or just iterate top level statements
            if let swc_ecma_ast::Program::Module(m) = &program {
                for item in &m.body {
                    if let swc_ecma_ast::ModuleItem::ModuleDecl(
                        swc_ecma_ast::ModuleDecl::ExportDecl(export),
                    ) = item
                    {
                        if let swc_ecma_ast::Decl::Class(class_decl) = &export.decl {
                            let class_name = class_decl.ident.sym.to_string();
                            class_module_map.insert(class_name.clone(), module_path.clone());

                            if let Some(type_params) = &class_decl.class.type_params {
                                if !type_params.params.is_empty() {
                                    generic_classes.insert(class_name);
                                }
                            }
                        }
                    }
                }
            }

            programs.push(program);
            file_paths.push(path.to_path_buf());
        }
    }

    // 2. Analyze (Build Dependency Graph)
    let graph = tyrus_analyzer::graph::build_graph(&programs);
    let init_order = graph
        .get_initialization_order()
        .map_err(TyrusError::FormattingError)?; // Using FormattingError as generic error for now

    // 3. Transpile
    for (i, program) in programs.iter().enumerate() {
        let path = &file_paths[i];
        let relative_path = path.strip_prefix(&input_dir).unwrap_or(path);
        let relative_path = relative_path.strip_prefix("src").unwrap_or(relative_path);
        let output_path = output_dir.join("src").join(relative_path);

        // Calculate module path for this file
        let file_stem = path.file_stem().unwrap_or_default().to_string_lossy();
        let sanitized_stem = file_stem.replace(['.', '-'], "_");

        // Check if it's index.ts
        let is_index = path.file_stem().and_then(|s| s.to_str()) == Some("index");

        let generated = tyrus_codegen::generate(program, is_index);
        let formatted_code = format_code(generated.code)?;

        let output_file = output_path.with_file_name(format!("{}.rs", sanitized_stem));

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).map_err(TyrusError::IoError)?;
        }

        fs::write(output_file, formatted_code).map_err(TyrusError::IoError)?;

        // Collect controllers
        for controller in generated.controllers {
            controllers.push(controller.struct_name);
        }
    }

    // 4. Generate mod.rs
    for entry in WalkDir::new(&output_dir) {
        let entry = entry.map_err(|e| TyrusError::IoError(e.into()))?;
        let path = entry.path();
        if path.is_dir() {
            generate_mod_rs(path)?;
        }
    }

    // Rename src/mod.rs to src/lib.rs for the crate root
    let src_mod = output_dir.join("src").join("mod.rs");
    let src_lib = output_dir.join("src").join("lib.rs");
    if src_mod.exists() {
        fs::rename(src_mod, src_lib.clone()).map_err(TyrusError::IoError)?;
    }

    // Generate error.rs
    let error_rs = output_dir.join("src").join("error.rs");
    let error_content = get_app_error_code();
    fs::write(error_rs, error_content).map_err(TyrusError::IoError)?;

    // Append mod error; pub use error::AppError; to lib.rs
    let mut lib_content = fs::read_to_string(&src_lib).map_err(TyrusError::IoError)?;
    lib_content.push_str("\npub mod error;\npub use error::AppError;\n");
    fs::write(&src_lib, lib_content).map_err(TyrusError::IoError)?;

    // 5. Generate main.rs
    let main_content = generate_main_rs(
        &init_order,
        &class_module_map,
        &controllers,
        &graph,
        &generic_classes,
    )?;

    // Ensure src directory exists
    let src_dir = output_dir.join("src");
    if !src_dir.exists() {
        fs::create_dir_all(&src_dir).map_err(TyrusError::IoError)?;
    }

    let main_rs = src_dir.join("main.rs");
    fs::write(main_rs, main_content).map_err(TyrusError::IoError)?;

    // 6. Generate Cargo.toml
    generate_cargo_toml(&output_dir)?;

    Ok(())
}

fn generate_main_rs(
    init_order: &[String],
    class_module_map: &std::collections::HashMap<String, String>,
    controllers: &[String],
    graph: &tyrus_analyzer::graph::DependencyGraph,
    generic_classes: &std::collections::HashSet<String>,
) -> Result<String, TyrusError> {
    let mut main_content = String::new();
    main_content.push_str("#![allow(unused)]\n\n");
    main_content.push_str("use axum::Router;\n");
    main_content.push_str("use tokio::net::TcpListener;\n");
    main_content.push_str("use std::sync::Arc;\n");
    main_content.push_str("use axum::Extension;\n\n");

    main_content.push_str("#[tokio::main]\n");
    main_content.push_str("async fn main() {\n");

    // Instantiate components in order
    let mut instantiated_vars = std::collections::HashMap::new();

    for class_name in init_order {
        if generic_classes.contains(class_name) {
            continue;
        }
        if let Some(module_path) = class_module_map.get(class_name) {
            let var_name = tyrus_common::util::to_snake_case(class_name);

            // Get dependencies
            let deps = graph.get_dependencies(class_name).unwrap_or_default();
            let mut args = Vec::new();
            for dep in deps {
                let dep_var = tyrus_common::util::to_snake_case(&dep);
                args.push(format!("{}.clone()", dep_var));
            }

            // Check if it has new_di
            // For now assume yes if it has dependencies, or just call new_di
            main_content.push_str(&format!(
                "    let {} = Arc::new({}::{}::new_di({}));\n",
                var_name,
                module_path,
                class_name,
                args.join(", ")
            ));

            instantiated_vars.insert(class_name.clone(), var_name);
        }
    }

    main_content.push_str("\n    // Build router\n");
    main_content.push_str("    let app = axum::Router::new()");

    // Register controllers
    for controller in controllers {
        if let Some(module_path) = class_module_map.get(controller) {
            main_content.push_str(&format!(
                "\n        .merge({}::{}::router())",
                module_path, controller
            ));
        }
    }

    // Add extensions
    for var_name in instantiated_vars.values() {
        main_content.push_str(&format!(
            "\n        .layer(Extension({}.clone()))",
            var_name
        ));
    }

    main_content.push_str(";\n\n");
    main_content
        .push_str("    let listener = TcpListener::bind(\"0.0.0.0:3000\").await.unwrap();\n");
    main_content.push_str("    println!(\"Server running on http://0.0.0.0:3000\");\n");
    main_content.push_str("    axum::serve(listener, app).await.unwrap();\n");
    main_content.push_str("}\n");

    Ok(main_content)
}

fn generate_cargo_toml(output_dir: &Path) -> Result<(), TyrusError> {
    let cargo_toml_content = r#"[package]
name = "tyrus_app"
version = "0.1.0"
edition = "2021"

[workspace]

[dependencies]
tokio = { version = "1.0", features = ["full"] }
axum = "0.7"
serde = { version = "1.0", features = ["derive", "rc"] }
serde_json = "1.0"
reqwest = { version = "0.11", features = ["json"] }
tower = { version = "0.4" }
tower-http = { version = "0.5", features = ["trace"] }
rand = "0.8"

[[bin]]
name = "server"
path = "src/main.rs"

[lib]
name = "tyrus_app"
path = "src/lib.rs"
"#;

    let cargo_toml_path = output_dir.join("Cargo.toml");
    fs::write(cargo_toml_path, cargo_toml_content).map_err(TyrusError::IoError)?;
    Ok(())
}

fn generate_mod_rs(dir: &Path) -> Result<(), TyrusError> {
    let mut mod_content = String::new();
    let mut has_children = false;
    let mut index_content = String::new();

    // Read directory entries
    let entries = fs::read_dir(dir).map_err(TyrusError::IoError)?;

    for entry in entries {
        let entry = entry.map_err(TyrusError::IoError)?;
        let path = entry.path();

        // Skip mod.rs, lib.rs, and main.rs
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            if name == "mod.rs" || name == "lib.rs" || name == "main.rs" {
                continue;
            }
        }

        if path.is_dir() {
            // If it's a directory, it should have a mod.rs inside (handled by recursion/iteration),
            // so we expose it as a module.
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                let sanitized_name = name.replace(['.', '-'], "_");
                mod_content.push_str(&format!("pub mod {};\n", sanitized_name));
                has_children = true;
            }
        } else if let Some(ext) = path.extension() {
            if ext == "rs" {
                if let Some(stem) = path.file_stem().and_then(|s| s.to_str()) {
                    if stem == "index" {
                        // Capture index.rs content to append later
                        index_content = fs::read_to_string(&path).map_err(TyrusError::IoError)?;
                        // We will remove index.rs later or just ignore it in the final build?
                        // Usually we want to remove it so we don't have both mod.rs and index.rs
                        // But let's delete it after reading.
                    } else {
                        // Sanitize module name just in case, though we sanitized filename on write
                        let sanitized_stem = stem.replace(['.', '-'], "_");
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
        fs::remove_file(index_path).map_err(TyrusError::IoError)?;
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
        fs::write(mod_path, mod_content).map_err(TyrusError::IoError)?;
    }

    Ok(())
}

fn format_code(code: String) -> Result<String, TyrusError> {
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
        .map_err(TyrusError::IoError)?;

    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(code.as_bytes())
            .map_err(TyrusError::IoError)?;
    }

    let output = child.wait_with_output().map_err(TyrusError::IoError)?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(TyrusError::FormattingError(
            String::from_utf8_lossy(&output.stderr).to_string(),
        ))
    }
}

fn get_app_error_code() -> &'static str {
    r#"
use axum::{response::{IntoResponse, Response}, http::StatusCode};

#[derive(Debug)]
pub struct AppError(Box<dyn std::error::Error + Send + Sync>);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            self.0.to_string(),
        )
            .into_response()
    }
}

impl<E> From<E> for AppError
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        Self(Box::new(err))
    }
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
"#
}
