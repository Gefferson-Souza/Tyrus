pub mod convert;
pub mod stdlib;

use convert::interface::RustGenerator;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;

#[derive(Debug, Clone)]
pub struct ControllerMetadata {
    pub struct_name: String,
    pub route_path: String,
}

pub struct GeneratedCode {
    pub code: String,
    pub controllers: Vec<ControllerMetadata>,
}

pub fn generate(program: &Program, is_index: bool) -> GeneratedCode {
    let mut generator = RustGenerator::new(is_index);
    program.visit_with(&mut generator);
    GeneratedCode {
        code: generator.code,
        controllers: generator.controllers,
    }
}
