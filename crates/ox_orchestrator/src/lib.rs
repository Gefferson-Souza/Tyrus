use ox_common::fs::FilePath;
use ox_diagnostics::OxidizerError;

pub fn check(path: FilePath) -> Result<(), OxidizerError> {
    let program = ox_parser::parse(path.as_ref())?;

    // Read source code for error reporting
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
    let generated_code = ox_codegen::generate(&program);
    format_code(generated_code)
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
