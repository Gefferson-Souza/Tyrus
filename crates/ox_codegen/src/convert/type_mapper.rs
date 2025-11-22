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

/// Unwraps Promise<T> to T for async function return types
pub fn unwrap_promise_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    if let Some(type_ann) = type_ann {
        if let TsType::TsTypeRef(type_ref) = &*type_ann.type_ann {
            // Check if this is a Promise type
            if let Some(ident) = type_ref.type_name.as_ident() {
                if ident.sym == "Promise" {
                    // Extract the generic parameter T from Promise<T>
                    if let Some(type_params) = &type_ref.type_params {
                        if let Some(first_param) = type_params.params.first() {
                            // Recursively map the inner type
                            return map_inner_type(first_param);
                        }
                    }
                }
            }
        }
    }
    // If not a Promise or no generic, fall back to regular mapping
    map_ts_type(type_ann)
}

fn map_inner_type(ts_type: &swc_ecma_ast::TsType) -> TokenStream {
    match ts_type {
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
}
