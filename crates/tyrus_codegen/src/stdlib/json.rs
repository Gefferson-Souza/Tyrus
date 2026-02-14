use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle JSON.* calls
pub fn handle(gen: &RustGenerator, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "stringify" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { serde_json::to_string(&#val).unwrap() })
            } else {
                None
            }
        }
        "parse" => {
            if let Some(arg) = args.first() {
                let val = gen.convert_expr_or_spread(arg);
                Some(quote! { serde_json::from_str::<serde_json::Value>(&#val).unwrap() })
            } else {
                None
            }
        }
        _ => None,
    }
}
