use proc_macro2::TokenStream;
use quote::quote;
use swc_ecma_ast::{TsType, TsTypeAnn};

/// Maps TypeScript types to Rust types
#[allow(clippy::borrowed_box)]
pub fn map_ts_type(type_ann: Option<&Box<TsTypeAnn>>) -> TokenStream {
    if let Some(type_ann) = type_ann {
        match &*type_ann.type_ann {
            TsType::TsKeywordType(k) => match k.kind {
                swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword => quote! { String },
                swc_ecma_ast::TsKeywordTypeKind::TsNumberKeyword => quote! { f64 },
                swc_ecma_ast::TsKeywordTypeKind::TsBooleanKeyword => quote! { bool },
                swc_ecma_ast::TsKeywordTypeKind::TsVoidKeyword => quote! { () },
                _ => quote! { serde_json::Value },
            },
            TsType::TsArrayType(array_type) => {
                let inner_type = map_inner_type(&array_type.elem_type);
                quote! { Vec<#inner_type> }
            }
            TsType::TsTypeRef(t) => {
                if let Some(ident) = t.type_name.as_ident() {
                    let name = ident.sym.as_str();
                    match name {
                        "Date" => quote! { String },
                        "Array" => {
                            if let Some(type_params) = &t.type_params {
                                if let Some(first_param) = type_params.params.first() {
                                    let inner = map_inner_type(first_param);
                                    quote! { Vec<#inner> }
                                } else {
                                    quote! { Vec<serde_json::Value> }
                                }
                            } else {
                                quote! { Vec<serde_json::Value> }
                            }
                        }
                        _ => {
                            // User defined type (Struct or Enum)
                            let type_ident =
                                proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
                            quote! { #type_ident }
                        }
                    }
                } else {
                    quote! { serde_json::Value }
                }
            }
            TsType::TsUnionOrIntersectionType(union_or_intersection) => {
                // Check for Optional (T | undefined)
                if let swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union) =
                    union_or_intersection
                {
                    let mut is_optional = false;
                    let mut inner_type = None;

                    for type_opt in &union.types {
                        match &**type_opt {
                            TsType::TsKeywordType(k)
                                if k.kind
                                    == swc_ecma_ast::TsKeywordTypeKind::TsUndefinedKeyword
                                    || k.kind == swc_ecma_ast::TsKeywordTypeKind::TsNullKeyword =>
                            {
                                is_optional = true;
                            }
                            _ => {
                                if inner_type.is_none() {
                                    inner_type = Some(map_inner_type(type_opt));
                                }
                            }
                        }
                    }

                    if is_optional {
                        if let Some(inner) = inner_type {
                            quote! { Option<#inner> }
                        } else {
                            quote! { Option<serde_json::Value> }
                        }
                    } else {
                        // Regular union - fallback to Value for now
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
#[allow(clippy::borrowed_box)]
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

pub fn is_optional_type(type_ann: Option<&TsTypeAnn>) -> bool {
    if let Some(type_ann) = type_ann {
        if let TsType::TsUnionOrIntersectionType(
            swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
        ) = &*type_ann.type_ann
        {
            for type_opt in &union.types {
                if let TsType::TsKeywordType(k) = &**type_opt {
                    if k.kind == swc_ecma_ast::TsKeywordTypeKind::TsUndefinedKeyword
                        || k.kind == swc_ecma_ast::TsKeywordTypeKind::TsNullKeyword
                    {
                        return true;
                    }
                }
            }
        }
    }
    false
}

fn map_inner_type(ts_type: &swc_ecma_ast::TsType) -> TokenStream {
    match ts_type {
        TsType::TsKeywordType(k) => match k.kind {
            swc_ecma_ast::TsKeywordTypeKind::TsStringKeyword => quote! { String },
            swc_ecma_ast::TsKeywordTypeKind::TsNumberKeyword => quote! { f64 },
            swc_ecma_ast::TsKeywordTypeKind::TsBooleanKeyword => quote! { bool },
            swc_ecma_ast::TsKeywordTypeKind::TsVoidKeyword => quote! { () },
            _ => quote! { serde_json::Value },
        },
        TsType::TsArrayType(array_type) => {
            let inner_type = map_inner_type(&array_type.elem_type);
            quote! { Vec<#inner_type> }
        }
        TsType::TsTypeRef(t) => {
            if let Some(ident) = t.type_name.as_ident() {
                let name = ident.sym.as_str();
                match name {
                    "Date" => quote! { String },
                    "Array" => {
                        if let Some(type_params) = &t.type_params {
                            if let Some(first_param) = type_params.params.first() {
                                let inner = map_inner_type(first_param);
                                quote! { Vec<#inner> }
                            } else {
                                quote! { Vec<serde_json::Value> }
                            }
                        } else {
                            quote! { Vec<serde_json::Value> }
                        }
                    }
                    _ => {
                        let type_ident =
                            proc_macro2::Ident::new(name, proc_macro2::Span::call_site());
                        quote! { #type_ident }
                    }
                }
            } else {
                quote! { serde_json::Value }
            }
        }
        _ => quote! { serde_json::Value },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use swc_common::DUMMY_SP;
    use swc_ecma_ast::{TsKeywordType, TsKeywordTypeKind};

    #[test]
    fn test_map_ts_type_string() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsStringKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "String");
    }

    #[test]
    fn test_map_ts_type_number() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsNumberKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "f64");
    }

    #[test]
    fn test_map_ts_type_boolean() {
        let ts_type = TsType::TsKeywordType(TsKeywordType {
            span: DUMMY_SP,
            kind: TsKeywordTypeKind::TsBooleanKeyword,
        });
        let type_ann = Box::new(TsTypeAnn {
            span: DUMMY_SP,
            type_ann: Box::new(ts_type),
        });
        let result = map_ts_type(Some(&type_ann));
        assert_eq!(result.to_string(), "bool");
    }

    #[test]
    fn test_map_ts_type_none() {
        let result = map_ts_type(None);
        assert_eq!(result.to_string(), "serde_json :: Value");
    }
}
