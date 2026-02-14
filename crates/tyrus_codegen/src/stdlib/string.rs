use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle string method calls
pub fn handle(
    gen: &RustGenerator,
    obj: &Expr,
    method: &str,
    args: &[ExprOrSpread],
) -> Option<TokenStream> {
    let obj_tokens = gen.convert_expr(obj);
    match method {
        "includes" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.contains(#val) })
            } else {
                None
            }
        }
        "replace" => {
            if args.len() == 2 {
                let pattern = gen.convert_expr_or_spread(&args[0]);
                let replacement = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { #obj_tokens.replace(&#pattern, &#replacement) })
            } else {
                None
            }
        }
        "split" => {
            if let Some(arg) = args.first() {
                let delimiter = gen.convert_expr_or_spread(arg);
                Some(quote! { #obj_tokens.split(&#delimiter).collect::<Vec<_>>() })
            } else {
                None
            }
        }
        "toUpperCase" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_uppercase() })
            } else {
                None
            }
        }
        "toLowerCase" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_lowercase() })
            } else {
                None
            }
        }
        "trim" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.trim() })
            } else {
                None
            }
        }
        "toString" => {
            if args.is_empty() {
                Some(quote! { #obj_tokens.to_string() })
            } else {
                None
            }
        }
        _ => None,
    }
}
