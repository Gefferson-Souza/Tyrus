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
    format_code(generated_code)
}

pub fn build_project(input_dir: PathBuf, output_dir: PathBuf) -> Result<(), OxidizerError> {
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

                let generated_code = ox_codegen::generate(&program, is_index);
                let formatted_code = format_code(generated_code)?;

                // Change extension to .rs
                let output_file = output_path.with_extension("rs");

                // Ensure parent dir exists (just in case)
                if let Some(parent) = output_file.parent() {
                    fs::create_dir_all(parent).map_err(OxidizerError::IoError)?;
                }

                fs::write(output_file, formatted_code).map_err(OxidizerError::IoError)?;
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
                mod_content.push_str(&format!("pub mod {};\n", name));
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
                        mod_content.push_str(&format!("pub mod {};\n", stem));
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
