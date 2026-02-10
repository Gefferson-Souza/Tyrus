use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::convert_expr_or_spread;

/// Handle JSON.* calls
pub fn handle(method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "stringify" => {
            if args.len() == 1 {
                let obj = convert_expr_or_spread(&args[0]);
                Some(quote! { serde_json::to_string(&#obj).unwrap() })
            } else {
                None
            }
        }
        "parse" => {
            if args.len() == 1 {
                let json_str = convert_expr_or_spread(&args[0]);
                Some(quote! { serde_json::from_str(#json_str).unwrap() })
            } else {
                None
            }
        }
        _ => None,
    }
}
