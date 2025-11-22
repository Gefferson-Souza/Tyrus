mod convert_struct;

use convert_struct::RustGenerator;
use swc_ecma_ast::Program;
use swc_ecma_visit::VisitWith;

pub fn generate(program: &Program) -> String {
    let mut generator = RustGenerator::new();
    program.visit_with(&mut generator);
    generator.code
}
