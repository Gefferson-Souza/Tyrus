use quote::{format_ident, quote};
use swc_ecma_ast::{Lit, TsInterfaceDecl, TsTypeElement};
use swc_ecma_visit::{Visit, VisitWith};

use super::type_mapper::map_ts_type;

use crate::ControllerMetadata;

#[derive(Default)]
pub struct RustGenerator {
    pub code: String,
    pub is_exporting: bool,
    pub is_index: bool,
    pub is_controller: bool,
    pub controllers: Vec<ControllerMetadata>,
    pub main_body: String,
    pub current_class_state_fields: std::collections::HashMap<String, String>,
}

impl RustGenerator {
    pub fn new(is_index: bool) -> Self {
        Self {
            code: String::new(),
            is_exporting: false,
            is_index,
            is_controller: false,
            controllers: Vec::new(),
            main_body: String::new(),
            current_class_state_fields: std::collections::HashMap::new(),
        }
    }
}

impl Visit for RustGenerator {
    fn visit_ts_interface_decl(&mut self, n: &TsInterfaceDecl) {
        let interface_name = n.id.sym.to_string();
        let struct_name = format_ident!("{}", interface_name);

        let mut fields = Vec::new();

        for member in &n.body.body {
            if let TsTypeElement::TsPropertySignature(prop) = member {
                let field_name_str = if let Some(ident) = prop.key.as_ident() {
                    ident.sym.to_string()
                } else {
                    continue; // Skip non-identifier keys for now
                };
                let field_name = format_ident!("{}", super::func::to_snake_case(&field_name_str));

                let mut field_type = map_ts_type(prop.type_ann.as_ref());

                if prop.optional {
                    field_type = quote! { Option<#field_type> };
                }

                fields.push(quote! {
                    pub #field_name: #field_type
                });
            }
        }

        let generics = if let Some(type_params) = &n.type_params {
            let params: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    let ident = format_ident!("{}", name);
                    quote! { #ident: Clone }
                })
                .collect();
            quote! { <#(#params),*> }
        } else {
            quote! {}
        };

        let struct_def = quote! {
            #[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            #[serde(rename_all = "camelCase")]
            pub struct #struct_name #generics {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');
    }

    fn visit_fn_decl(&mut self, n: &swc_ecma_ast::FnDecl) {
        self.process_fn_decl(n);
    }

    fn visit_class_decl(&mut self, n: &swc_ecma_ast::ClassDecl) {
        self.process_class_decl(n);
    }

    fn visit_ts_type_alias_decl(&mut self, n: &swc_ecma_ast::TsTypeAliasDecl) {
        let alias_name_str = n.id.sym.to_string();
        let alias_name = format_ident!("{}", alias_name_str);

        // Check for String Union: type Status = "open" | "closed"
        // let mut is_string_union = false; (Unused)
        if let swc_ecma_ast::TsType::TsUnionOrIntersectionType(
            swc_ecma_ast::TsUnionOrIntersectionType::TsUnionType(union),
        ) = &*n.type_ann
        {
            // Check if all members are string literals
            if !union.types.is_empty()
                && union.types.iter().all(|t| {
                    matches!(
                        &**t,
                        swc_ecma_ast::TsType::TsLitType(lit)
                            if matches!(lit.lit, swc_ecma_ast::TsLit::Str(_))
                    )
                })
            {
                // is_string_union = true; (Unused variable removed)

                // Generate Enum
                let mut valid_variants = Vec::new();

                for t in &union.types {
                    if let swc_ecma_ast::TsType::TsLitType(lit) = &**t {
                        if let swc_ecma_ast::TsLit::Str(s) = &lit.lit {
                            let value = s.value.as_str().unwrap_or("").to_string();
                            let variant_name = super::func::to_pascal_case(&value);
                            let variant_ident = format_ident!("{}", variant_name);
                            valid_variants.push((value, variant_ident));
                        }
                    }
                }

                let variants: Vec<_> = valid_variants
                    .iter()
                    .enumerate()
                    .map(|(i, (value, variant_ident))| {
                        let default_attr = if i == 0 {
                            quote! { #[default] }
                        } else {
                            quote! {}
                        };
                        quote! {
                            #default_attr
                            #[serde(rename = #value)]
                            #variant_ident
                        }
                    })
                    .collect();

                let eq_arms_string: Vec<_> = valid_variants
                    .iter()
                    .map(|(value, variant_ident)| {
                        quote! {
                            #alias_name::#variant_ident => other == #value
                        }
                    })
                    .collect();

                let eq_arms_str: Vec<_> = valid_variants
                    .iter()
                    .map(|(value, variant_ident)| {
                        quote! {
                            #alias_name::#variant_ident => *other == #value
                        }
                    })
                    .collect();

                let vis = if self.is_exporting {
                    quote! { pub }
                } else {
                    quote! {}
                };

                // Add Default to derive
                let enum_def = quote! {
                    #[derive(Debug, Clone, Copy, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
                    #vis enum #alias_name {
                        #(#variants),*
                    }

                    impl PartialEq<String> for #alias_name {
                        fn eq(&self, other: &String) -> bool {
                            match self {
                                #(#eq_arms_string),*
                            }
                        }
                    }

                    impl PartialEq<&str> for #alias_name {
                        fn eq(&self, other: &&str) -> bool {
                             match self {
                                #(#eq_arms_str),*
                            }
                        }
                    }
                };
                self.code.push_str(&enum_def.to_string());
                self.code.push('\n');
                return;
            }
        }

        let alias_type = map_ts_type(Some(&Box::new(swc_ecma_ast::TsTypeAnn {
            span: swc_common::DUMMY_SP,
            type_ann: n.type_ann.clone(),
        })));

        let vis = if self.is_exporting {
            quote! { pub }
        } else {
            quote! {}
        };

        let type_def = quote! {
            #vis type #alias_name = #alias_type;
        };

        self.code.push_str(&type_def.to_string());
        self.code.push('\n');
    }

    fn visit_ts_enum_decl(&mut self, n: &swc_ecma_ast::TsEnumDecl) {
        let enum_name = format_ident!("{}", n.id.sym.to_string());

        // Detect if this is a string enum or numeric enum
        let is_string_enum = n.members.iter().any(|m| {
            m.init
                .as_ref()
                .is_some_and(|init| matches!(init.as_ref(), swc_ecma_ast::Expr::Lit(Lit::Str(_))))
        });

        if is_string_enum {
            // String enum → derive Serialize/Deserialize for JSON compat
            let variants: Vec<_> = n
                .members
                .iter()
                .map(|m| {
                    let variant_name_str = match &m.id {
                        swc_ecma_ast::TsEnumMemberId::Ident(i) => i.sym.to_string(),
                        swc_ecma_ast::TsEnumMemberId::Str(s) => {
                            s.value.as_str().unwrap_or("").to_string()
                        }
                    };
                    let variant_ident = format_ident!("{}", variant_name_str);

                    // Extract the string value for serde rename
                    let rename = m
                        .init
                        .as_ref()
                        .and_then(|init| {
                            if let swc_ecma_ast::Expr::Lit(Lit::Str(s)) = init.as_ref() {
                                Some(s.value.as_str().unwrap_or("").to_string())
                            } else {
                                None
                            }
                        })
                        .unwrap_or_else(|| variant_name_str.clone());

                    if rename == variant_name_str {
                        quote! { #variant_ident }
                    } else {
                        quote! {
                            #[serde(rename = #rename)]
                            #variant_ident
                        }
                    }
                })
                .collect();

            let vis = if self.is_exporting {
                quote! { pub }
            } else {
                quote! {}
            };

            let enum_def = quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
                #vis enum #enum_name {
                    #(#variants),*
                }
            };

            self.code.push_str(&enum_def.to_string());
            self.code.push('\n');
        } else {
            // Numeric enum → use repr(i32) with explicit discriminants
            let mut current_value: i64 = 0;
            let variants: Vec<_> = n
                .members
                .iter()
                .map(|m| {
                    let variant_name_str = match &m.id {
                        swc_ecma_ast::TsEnumMemberId::Ident(i) => i.sym.to_string(),
                        swc_ecma_ast::TsEnumMemberId::Str(s) => {
                            s.value.as_str().unwrap_or("").to_string()
                        }
                    };
                    let variant_ident = format_ident!("{}", variant_name_str);

                    // Check if there's an explicit numeric value
                    if let Some(init) = &m.init {
                        if let swc_ecma_ast::Expr::Lit(Lit::Num(num)) = init.as_ref() {
                            current_value = num.value as i64;
                        }
                    }

                    let val = current_value as i32;
                    current_value += 1;

                    quote! { #variant_ident = #val }
                })
                .collect();

            let vis = if self.is_exporting {
                quote! { pub }
            } else {
                quote! {}
            };

            let enum_def = quote! {
                #[derive(Debug, Clone, Copy, PartialEq, Eq)]
                #[repr(i32)]
                #vis enum #enum_name {
                    #(#variants),*
                }
            };

            self.code.push_str(&enum_def.to_string());
            self.code.push('\n');
        }
    }

    fn visit_module_item(&mut self, n: &swc_ecma_ast::ModuleItem) {
        self.process_module_item(n);
    }

    fn visit_stmt(&mut self, n: &swc_ecma_ast::Stmt) {
        // This is called for top-level statements via process_module_item -> visit_with(self)
        match n {
            swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Fn(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::Class(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsInterface(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsTypeAlias(_))
            | swc_ecma_ast::Stmt::Decl(swc_ecma_ast::Decl::TsEnum(_)) => {
                // Top-level declarations: let visitor handle them (writes to self.code)
                n.visit_children_with(self);
            }
            _ => {
                // Script statements (ExprStmt, VarDecl, If, Loop, etc.): write to self.main_body
                let stmt_code = self.convert_stmt(n);
                self.main_body.push_str(&stmt_code.to_string());
                self.main_body.push('\n');
            }
        }
    }
}
