use proc_macro2::TokenStream;
use swc_ecma_ast::{Callee, Expr, ExprOrSpread};

use crate::convert::interface::RustGenerator;

pub mod array;
pub mod console;
pub mod json;
pub mod math;
pub mod string;

/// Main dispatcher for stdlib method calls
pub fn try_handle_stdlib_call(
    gen: &RustGenerator,
    callee: &Callee,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    // Try to handle as stdlib call
    if let Callee::Expr(expr) = callee {
        if let Expr::Member(member) = &**expr {
            if let Expr::Ident(obj) = &*member.obj {
                match obj.sym.as_ref() {
                    "console" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return console::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "Math" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return math::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    "JSON" => {
                        if let Some(prop) = member.prop.as_ident() {
                            return json::handle(gen, prop.sym.as_ref(), args);
                        }
                    }
                    _ => {}
                }
            }
        }
    }
    None
}

/// Try to handle method call on an expression (e.g., str.includes())
pub fn try_handle_method_call(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    // Check if object is array-like or string-like based on heuristic or type (if available)
    // For now we try to apply array/string methods if the method name matches unique ones

    if let Some(res) = array::handle(gen, obj, method, args) {
        return Some(res);
    }
    if let Some(res) = string::handle(gen, obj, method, args) {
        return Some(res);
    }

    None
}
