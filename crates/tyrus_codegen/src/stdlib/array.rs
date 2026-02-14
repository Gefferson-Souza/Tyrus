use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle array method calls
pub fn handle(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "push" => {
            // Array push is tricky because we need the object.
            // But here we are just returning tokens for the *callee*? No, the whole call?
            // Actually try_handle_method_call in mod.rs doesn't seem to pass the object to us!
            // Wait, try_handle_method_call takes (obj, method, args).
            // But array::handle only accepts (method, args).
            // This logic seems flawed in the original code too if it intended to operate on 'obj'.
            // However, `func.rs` loop converts method calls like `arr.push(x)` -> `arr.push(x)`.
            // If this logic returns None, it falls back to generic conversion.
            // Rust Vec has .push().

            // If we just return None, generic conversion `callee(args)` works: `arr.push(x)`.
            // So maybe we don't need special handling for push unless we want to change it.
            None
        }
        "map" => {
            // arr.map(x => x+1) -> arr.iter().map(|x| x+1).collect::<Vec<_>>()
            // This requires context of the closure.
            None
        }
        "filter" => {
            // arr.filter(x => x > 1)
            None
        }
        "join" => {
            if args.len() == 1 {
                let separator = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.join(&#separator) })
            } else {
                None
            }
        }
        _ => None,
    }
}
