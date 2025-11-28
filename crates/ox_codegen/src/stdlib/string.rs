use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::{convert_expr, convert_expr_or_spread};

/// Handle string method calls
pub fn handle_method(obj: &Expr, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    let obj_tokens = convert_expr(obj);

    match method {
        "includes" => {
            if args.len() == 1 {
                let arg = convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.contains(&#arg) })
            } else {
                None
            }
        }
        "replace" => {
            if args.len() == 2 {
                let pattern = convert_expr_or_spread(&args[0]);
                let replacement = convert_expr_or_spread(&args[1]);
                Some(quote! { #obj_tokens.replace(&#pattern, &#replacement) })
            } else {
                None
            }
        }
        "split" => {
            if args.len() == 1 {
                let delimiter = convert_expr_or_spread(&args[0]);
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
