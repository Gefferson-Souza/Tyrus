use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::*;

use super::super::convert::interface::RustGenerator;

/// Handle console.* calls
pub fn handle(gen: &RustGenerator, method: &str, args: &[ExprOrSpread]) -> Option<TokenStream> {
    match method {
        "log" => {
            let args_tokens: Vec<_> = args.iter().map(|a| gen.convert_expr_or_spread(a)).collect();
            let fmt_str = "{} ".repeat(args.len()).trim_end().to_string();
            Some(quote! { println!(#fmt_str, #(#args_tokens),*) })
        }
        "error" => {
            let args_tokens: Vec<_> = args.iter().map(|a| gen.convert_expr_or_spread(a)).collect();
            let fmt_str = "{} ".repeat(args.len()).trim_end().to_string();
            Some(quote! { eprintln!(#fmt_str, #(#args_tokens),*) })
        }
        _ => None,
    }
}
