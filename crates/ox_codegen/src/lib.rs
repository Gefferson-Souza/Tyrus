pub mod convert;
pub mod stdlib;

use convert::interface::RustGenerator;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;

pub fn generate(program: &Program, is_index: bool) -> String {
    let mut generator = RustGenerator::new(is_index);
    program.visit_with(&mut generator);
    generator.code
}
