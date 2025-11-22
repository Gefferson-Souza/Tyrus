use miette::{NamedSource, SourceSpan};
use ox_diagnostics::OxidizerError;

use swc_ecma_ast::{
    CallExpr, Callee, Expr, TsKeywordType, TsKeywordTypeKind, VarDecl, VarDeclKind,
};
use swc_ecma_visit::{Visit, VisitWith};

pub struct LintVisitor {
    pub errors: Vec<OxidizerError>,
    pub source_code: String,
    pub file_name: String,
}

impl LintVisitor {
    pub fn new(source_code: String, file_name: String) -> Self {
        Self {
            errors: Vec::new(),
            source_code,
            file_name,
        }
    }

    fn create_span(&self, span: swc_common::Span) -> SourceSpan {
        let start = span.lo.0 as usize - 1;
        let end = span.hi.0 as usize - 1;
        let len = end - start;
        SourceSpan::new(start.into(), len.into())
    }
}

impl Visit for LintVisitor {
    fn visit_var_decl(&mut self, n: &VarDecl) {
        if n.kind == VarDeclKind::Var {
            self.errors.push(OxidizerError::UseOfVar {
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(n.span),
            });
        }
        n.visit_children_with(self);
    }

    fn visit_ts_keyword_type(&mut self, n: &TsKeywordType) {
        if n.kind == TsKeywordTypeKind::TsAnyKeyword {
            self.errors.push(OxidizerError::UseOfAny {
                src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                span: self.create_span(n.span),
            });
        }
        n.visit_children_with(self);
    }

    fn visit_call_expr(&mut self, n: &CallExpr) {
        if let Callee::Expr(expr) = &n.callee {
            if let Expr::Ident(ident) = &**expr {
                if ident.sym == "eval" {
                    self.errors.push(OxidizerError::UseOfEval {
                        src: NamedSource::new(self.file_name.clone(), self.source_code.clone()),
                        span: self.create_span(n.span),
                    });
                }
            }
        }
        n.visit_children_with(self);
    }
}
