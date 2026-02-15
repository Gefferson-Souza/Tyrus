use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle Math.* calls
pub fn handle(gen: &RustGenerator, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "max" => {
            if args.len() == 1 && args[0].spread.is_some() {
                // Math.max(...arr)
                let arg = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #arg.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b))
                })
            } else if args.len() == 2 {
                if args[0].spread.is_some() {
                    // Math.max(...arr, val)
                    let arr = gen.convert_expr_or_spread(&args[0]);
                    let val = gen.convert_expr_or_spread(&args[1]);
                    Some(quote! {
                        #arr.iter().fold(#val, |a, &b| a.max(b))
                    })
                } else {
                    // Math.max(a, b)
                    let a = gen.convert_expr_or_spread(&args[0]);
                    let b = gen.convert_expr_or_spread(&args[1]);
                    Some(quote! { #a.max(#b) })
                }
            } else {
                None
            }
        }
        "min" => {
            if args.len() == 1 && args[0].spread.is_some() {
                // Math.min(...arr) -> arr.iter().fold(f64::INFINITY, |a, &b| a.min(b))
                let arg = gen.convert_expr_or_spread(&args[0]);
                Some(quote! {
                    #arg.iter().fold(f64::INFINITY, |a, &b| a.min(b))
                })
            } else if args.len() == 2 {
                let a = gen.convert_expr_or_spread(&args[0]);
                let b = gen.convert_expr_or_spread(&args[1]);
                Some(quote! { #a.min(#b) })
            } else {
                None
            }
        }
        "round" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).round() })
            } else {
                None
            }
        }
        "floor" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).floor() })
            } else {
                None
            }
        }
        "ceil" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).ceil() })
            } else {
                None
            }
        }
        "abs" => {
            if args.len() == 1 {
                let x = gen.convert_expr_or_spread(&args[0]);
                Some(quote! { (#x).abs() })
            } else {
                None
            }
        }
        "random" => {
            if args.is_empty() {
                Some(quote! { rand::random::<f64>() })
            } else {
                None
            }
        }
        _ => None,
    }
}
