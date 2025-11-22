use quote::{format_ident, quote};
use swc_ecma_ast::{TsInterfaceDecl, TsTypeElement};
use swc_ecma_visit::Visit;

use super::type_mapper::map_ts_type;

#[derive(Default)]
pub struct RustGenerator {
    pub code: String,
}

impl RustGenerator {
    pub fn new() -> Self {
        Self::default()
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

                let field_type = map_ts_type(prop.type_ann.as_ref());

                fields.push(quote! {
                    pub #field_name: #field_type
                });
            }
        }

        let struct_def = quote! {
            #[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            pub struct #struct_name {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');
    }

    fn visit_fn_decl(&mut self, n: &swc_ecma_ast::FnDecl) {
        self.visit_fn_decl(n);
    }
}
