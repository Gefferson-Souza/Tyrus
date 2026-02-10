use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::func::convert_expr_or_spread;

/// Handle console.* calls
pub fn handle(method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "log" => {
            let args_tokens: Vec<_> = args.iter().map(convert_expr_or_spread).collect();
            let fmt_str = "{} ".repeat(args.len()).trim_end().to_string();
            Some(quote! { println!(#fmt_str, #(#args_tokens),*) })
        }
        "error" => {
            let args_tokens: Vec<_> = args.iter().map(convert_expr_or_spread).collect();
            let fmt_str = "{} ".repeat(args.len()).trim_end().to_string();
            Some(quote! { eprintln!(#fmt_str, #(#args_tokens),*) })
        }
        _ => None,
    }
}
