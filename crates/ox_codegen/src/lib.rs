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

    if !generator.main_body.is_empty() && is_index {
        generator.code.push_str("\npub fn main() {\n");
        generator.code.push_str(&generator.main_body);
        generator.code.push_str("}\n");
    }

    GeneratedCode {
        code: generator.code,
        controllers: generator.controllers,
    }
}
