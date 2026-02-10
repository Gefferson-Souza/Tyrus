use proc_macro2::TokenStream;
use swc_ecma_ast::*;

pub mod array;
pub mod console;
pub mod json;
pub mod math;
pub mod string;

/// Main dispatcher for stdlib method calls
pub fn try_handle_stdlib_call(callee: &Callee, args: &[ExprOrSpread]) -> Option<TokenStream> {
    // Try to handle as stdlib call
    if let Callee::Expr(expr) = callee {
        if let Expr::Member(member) = &**expr {
            if let Expr::Ident(obj) = &*member.obj {
                let obj_name = obj.sym.as_ref();

                if obj_name == "Math" {
                    if let Some(method_ident) = member.prop.as_ident() {
                        return math::handle(&method_ident.sym, args);
                    }
                } else if obj_name == "JSON" {
                    if let Some(method_ident) = member.prop.as_ident() {
                        return json::handle(&method_ident.sym, args);
                    }
                } else if obj_name == "console" {
                    if let Some(method_ident) = member.prop.as_ident() {
                        return console::handle(&method_ident.sym, args);
                    }
                }
            }
        }
    }

    None
}

/// Try to handle method call on an expression (e.g., str.includes())
pub fn try_handle_method_call(
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    // For now, we'll handle this based on method name
    // In the future, we could use type inference

    match method {
        // String methods
        "includes" | "replace" | "split" | "toUpperCase" | "toLowerCase" | "trim" | "toString" => {
            string::handle_method(obj, method, args)
        }
        // Array methods
        "push" | "map" | "filter" | "join" => array::handle_method(obj, method, args),
        _ => None,
    }
}
