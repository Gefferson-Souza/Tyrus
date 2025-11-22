use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{TsType, TsTypeAnn};

/// Maps TypeScript types to Rust types
pub fn map_ts_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    if let Some(type_ann) = type_ann {
        match &*type_ann.type_ann {
            TsType::TsKeywordType(k) => match k.kind {
                swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword => quote! { String },
                swc_ecma_ast::TsKeywordTypeKind::TsNumberKeyword => quote! { f64 },
                swc_ecma_ast::TsKeywordTypeKind::TsBooleanKeyword => quote! { bool },
                _ => quote! { serde_json::Value },
            },
            TsType::TsTypeRef(t) => {
                if let Some(ident) = t.type_name.as_ident() {
                    if ident.sym == "Date" {
                        quote! { String }
                    } else {
                        quote! { serde_json::Value }
                    }
                } else {
                    quote! { serde_json::Value }
                }
            }
            _ => quote! { serde_json::Value },
        }
    } else {
        quote! { serde_json::Value }
    }
}
