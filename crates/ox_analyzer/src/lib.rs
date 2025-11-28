pub mod graph;
pub mod lints;

use crate::lints::LintVisitor;
use ox_diagnostics::OxidizerError;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;

pub struct Analyzer;

impl Analyzer {
    pub fn analyze(
        program: &Program,
        source_code: String,
        file_name: String,
    ) -> Vec<OxidizerError> {
        let mut visitor = LintVisitor::new(source_code, file_name);
        program.visit_with(&mut visitor);
        visitor.errors
    }
}
