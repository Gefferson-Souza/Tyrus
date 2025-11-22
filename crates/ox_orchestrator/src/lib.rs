use ox_diagnostics::OxidizerError;
use std::path::PathBuf;

pub fn check(path: PathBuf) -> Result<(), OxidizerError> {
    let program = ox_parser::parse(&path)?;

    // Read source code for error reporting
    let source_code = std::fs::read_to_string(&path).map_err(OxidizerError::IoError)?;
    let file_name = path.to_string_lossy().to_string();

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
