use quote::{format_ident, quote};
use swc_ecma_ast::{TsInterfaceDecl, TsTypeElement};
use swc_ecma_visit::Visit;

use super::type_mapper::map_ts_type;

use crate::ControllerMetadata;

#[derive(Default)]
pub struct RustGenerator {
    pub code: String,
    pub is_exporting: bool,
    pub is_index: bool,
    pub controllers: Vec<ControllerMetadata>,
}

impl RustGenerator {
    pub fn new(is_index: bool) -> Self {
        Self {
            code: String::new(),
            is_exporting: false,
            is_index,
            controllers: Vec::new(),
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
                let field_name = format_ident!("{}", field_name_str);

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
        let alias_name = format_ident!("{}", n.id.sym.to_string());
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

    fn visit_module_item(&mut self, n: &swc_ecma_ast::ModuleItem) {
        self.process_module_item(n);
    }
}
