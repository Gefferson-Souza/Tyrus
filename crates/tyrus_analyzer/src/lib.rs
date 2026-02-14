pub mod decorators;
pub mod graph;
pub mod lints;

use crate::lints::LintVisitor;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;
use tyrus_diagnostics::TyrusError;

pub struct Analyzer;

use crate::decorators::DecoratorVisitor;
use tyrus_di::graph::DiGraph;

pub struct AnalysisResult {
    pub errors: Vec<TyrusError>,
    pub graph: DiGraph,
}

impl Analyzer {
    pub fn analyze(program: &Program, source_code: String, file_name: String) -> AnalysisResult {
        let mut lint_visitor = LintVisitor::new(source_code, file_name);
        program.visit_with(&mut lint_visitor);

        let mut decorator_visitor = DecoratorVisitor::new();
        program.visit_with(&mut decorator_visitor);

        AnalysisResult {
            errors: lint_visitor.errors,
            graph: decorator_visitor.graph,
        }
    }
}
