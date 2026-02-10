use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::{convert_expr, convert_expr_or_spread};

/// Handle array method calls
pub fn handle_method(obj: &Expr, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    let obj_tokens = convert_expr(obj);

    match method {
        "push" => {
            if args.len() == 1 {
                let arg = convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.push(#arg) })
            } else {
                None
            }
        }
        "map" => {
            if args.len() == 1 {
                let callback = convert_expr_or_spread(&args[0]);
                let param_count = if let Expr::Arrow(arrow) = &*args[0].expr {
                    arrow.params.len()
                } else {
                    1
                };

                if param_count > 1 {
                    Some(quote! {
                        #obj_tokens.iter().cloned().enumerate().map(|(idx, val)| (#callback)(val, idx as f64)).collect::<Vec<_>>()
                    })
                } else {
                    Some(quote! { #obj_tokens.iter().cloned().map(#callback).collect::<Vec<_>>() })
                }
            } else {
                None
            }
        }
        "filter" => {
            if args.len() == 1 {
                let callback = convert_expr_or_spread(&args[0]);
                // Use filter on borrowed iter, then collect cloned values
                Some(
                    quote! { #obj_tokens.iter().filter(|x| (#callback)(*x)).cloned().collect::<Vec<_>>() },
                )
            } else {
                None
            }
        }
        "join" => {
            if args.len() == 1 {
                let separator = convert_expr_or_spread(&args[0]);
                Some(quote! { #obj_tokens.join(&#separator) })
            } else {
                None
            }
        }
        _ => None,
    }
}
