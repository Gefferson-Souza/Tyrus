use swc_ecma_ast::Program;

use crate::module::TyrusModule;

/// Lower a SWC `Program` to a `TyrusModule`
///
/// Entry point for the lowering pass. Currently a foundation stub —
/// will be expanded incrementally as codegen migrates to use the IR.
pub fn lower_program(_program: &Program) -> TyrusModule {
    TyrusModule::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lower_empty_module() {
        let module = swc_ecma_ast::Module {
            span: swc_common::DUMMY_SP,
            body: Vec::new(),
            shebang: None,
        };
        let program = Program::Module(module);
        let result = lower_program(&program);
        assert_eq!(
            result,
            TyrusModule {
                declarations: Vec::new(),
                statements: Vec::new(),
                imports: Vec::new(),
            }
        );
    }

    #[test]
    fn test_lower_empty_script() {
        let script = swc_ecma_ast::Script {
            span: swc_common::DUMMY_SP,
            body: Vec::new(),
            shebang: None,
        };
        let program = Program::Script(script);
        let result = lower_program(&program);
        assert_eq!(
            result,
            TyrusModule {
                declarations: Vec::new(),
                statements: Vec::new(),
                imports: Vec::new(),
            }
        );
    }
}
