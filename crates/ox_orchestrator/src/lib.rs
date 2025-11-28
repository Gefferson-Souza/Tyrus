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
    let mut controllers: Vec<String> = Vec::new(); // Just names of controllers
    let mut class_module_map: std::collections::HashMap<String, String> =
        std::collections::HashMap::new();
    let mut programs = Vec::new();
    let mut file_paths = Vec::new();

    // 1. Walk, Parse, and Collect Info
    for entry in WalkDir::new(&input_dir) {
        let entry = entry.map_err(|e| OxidizerError::IoError(e.into()))?;
        let path = entry.path();

        if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("ts") {
            let program = ox_parser::parse(path)?;

            // Calculate module path
            let relative_path = path.strip_prefix(&input_dir).unwrap_or(path);
            let file_stem = path
                .file_stem()
                .ok_or_else(|| {
                    OxidizerError::IoError(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid file path",
                    ))
                })?
                .to_string_lossy();
            let sanitized_stem = file_stem.replace('.', "_");

            let mut module_parts = Vec::new();
            module_parts.push("typerust_app".to_string()); // Use lib crate name

            if let Some(parent) = relative_path.parent() {
                for part in parent.components() {
                    if let std::path::Component::Normal(s) = part {
                        let s_str = s.to_string_lossy();
                        if s_str != "src" {
                            module_parts.push(s_str.to_string());
                        }
                    }
                }
            }
            module_parts.push(sanitized_stem);
            let module_path = module_parts.join("::");

            // Extract classes to map them
            // We use a simple visitor or just iterate top level statements
            match &program {
                swc_ecma_ast::Program::Module(m) => {
                    for item in &m.body {
                        if let swc_ecma_ast::ModuleItem::ModuleDecl(
                            swc_ecma_ast::ModuleDecl::ExportDecl(export),
                        ) = item
                        {
                            if let swc_ecma_ast::Decl::Class(class_decl) = &export.decl {
                                let class_name = class_decl.ident.sym.to_string();
                                class_module_map.insert(class_name, module_path.clone());
                            }
                        }
                    }
                }
                _ => {}
            }

            programs.push(program);
            file_paths.push(path.to_path_buf());
        }
    }

    // 2. Analyze (Build Dependency Graph)
    let graph = ox_analyzer::graph::build_graph(&programs);
    let init_order = graph
        .get_initialization_order()
        .map_err(|e| OxidizerError::FormattingError(e))?; // Using FormattingError as generic error for now

    // 3. Transpile
    for (i, program) in programs.iter().enumerate() {
        let path = &file_paths[i];
        let relative_path = path.strip_prefix(&input_dir).unwrap_or(path);
        let output_path = output_dir.join("src").join(relative_path);

        // Check if it's index.ts
        let is_index = path.file_stem().and_then(|s| s.to_str()) == Some("index");

        let generated = ox_codegen::generate(program, is_index);
        let formatted_code = format_code(generated.code)?;

        let file_stem = path
            .file_stem()
            .ok_or_else(|| {
                OxidizerError::IoError(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid file path",
                ))
            })?
            .to_string_lossy();
        let sanitized_stem = file_stem.replace('.', "_");
        let output_file = output_path.with_file_name(format!("{}.rs", sanitized_stem));

        if let Some(parent) = output_file.parent() {
            fs::create_dir_all(parent).map_err(OxidizerError::IoError)?;
        }

        fs::write(output_file, formatted_code).map_err(OxidizerError::IoError)?;

        // Collect controllers
        for controller in generated.controllers {
            controllers.push(controller.struct_name);
        }
    }

    // 4. Generate mod.rs
    for entry in WalkDir::new(&output_dir) {
        let entry = entry.map_err(|e| OxidizerError::IoError(e.into()))?;
        let path = entry.path();
        if path.is_dir() {
            generate_mod_rs(path)?;
        }
    }

    // Rename src/mod.rs to src/lib.rs for the crate root
    let src_mod = output_dir.join("src").join("mod.rs");
    let src_lib = output_dir.join("src").join("lib.rs");
    if src_mod.exists() {
        fs::rename(src_mod, src_lib).map_err(OxidizerError::IoError)?;
    }

    // 5. Generate Cargo.toml
    generate_cargo_toml(&output_dir)?;

    // 6. Generate main.rs
    generate_main_rs(
        &output_dir,
        &init_order,
        &class_module_map,
        &controllers,
        &graph,
    )?;

    Ok(())
}

fn generate_main_rs(
    output_dir: &Path,
    init_order: &[String],
    class_module_map: &std::collections::HashMap<String, String>,
    controllers: &[String],
    graph: &ox_analyzer::graph::DependencyGraph,
) -> Result<(), OxidizerError> {
    let mut main_content = String::new();
    main_content.push_str("use axum::Router;\n");
    main_content.push_str("use tokio::net::TcpListener;\n");
    main_content.push_str("use std::sync::Arc;\n");
    main_content.push_str("use axum::Extension;\n\n");

    main_content.push_str("#[tokio::main]\n");
    main_content.push_str("async fn main() {\n");

    // Instantiate components in order
    let mut instantiated_vars = std::collections::HashMap::new();

    for class_name in init_order {
        if let Some(module_path) = class_module_map.get(class_name) {
            let var_name = ox_common::util::to_snake_case(class_name);

            // Get dependencies
            let deps = graph.get_dependencies(class_name).unwrap_or_default();
            let mut args = Vec::new();
            for dep in deps {
                let dep_var = ox_common::util::to_snake_case(&dep);
                args.push(format!("{}.clone()", dep_var));
            }
            let args_str = args.join(", ");

            main_content.push_str(&format!(
                "    let {} = Arc::new({}::{}::new({}));\n",
                var_name, module_path, class_name, args_str
            ));

            instantiated_vars.insert(class_name, var_name);
        }
    }

    main_content.push_str("\n    let app = Router::new()");

    // Merge controller routers
    // We need to find the instantiated controller variable
    for controller_name in controllers {
        if let Some(_var_name) = instantiated_vars.get(controller_name) {
            if let Some(module_path) = class_module_map.get(controller_name) {
                // .merge(typerust_app::cats::cats_controller::CatsController::router())
                // But wait, router() is static and creates a new Router.
                // It doesn't take the controller instance.
                // We need to pass the controller instance to the router via Extension.
                // The router() function we generated adds .layer(Extension(Self::default())).
                // We removed Self::default() in previous step (or intended to).
                // Actually, we kept it but we should override it.
                // If we do .merge(Controller::router()).layer(Extension(controller_instance)),
                // the Extension(controller_instance) will be available to the routes.

                main_content.push_str(&format!(
                    "\n        .merge({}::{}::router())",
                    module_path, controller_name
                ));
            }
        }
    }

    // 4. Add Extension layers
    for (_class_name, var_name) in &instantiated_vars {
        // Assuming instantiated_services is a typo and it should be instantiated_vars
        main_content.push_str(&format!(
            "        .layer(Extension({}.clone()))\n",
            var_name
        ));
    }

    main_content.push_str(";\n\n");

    main_content
        .push_str("    let listener = TcpListener::bind(\"0.0.0.0:3000\").await.unwrap();\n");
    main_content.push_str("    println!(\"Server running on http://0.0.0.0:3000\");\n");
    main_content.push_str("    axum::serve(listener, app).await.unwrap();\n");
    main_content.push_str("}\n");

    let main_path = output_dir.join("src").join("main.rs");
    if let Some(parent) = main_path.parent() {
        fs::create_dir_all(parent).map_err(OxidizerError::IoError)?;
    }
    fs::write(main_path, main_content).map_err(OxidizerError::IoError)?;
    Ok(())
}

fn generate_cargo_toml(output_dir: &Path) -> Result<(), OxidizerError> {
    let cargo_toml_content = r#"[package]
name = "typerust_app"
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
