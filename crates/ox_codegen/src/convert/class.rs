use quote::{format_ident, quote};
use swc_ecma_ast::{AssignTarget, ClassDecl, ClassMember, Constructor, Expr, ExprStmt, Pat, Stmt};

use super::func::{convert_expr_pub, convert_stmt_pub, to_snake_case};
use super::interface::RustGenerator;
use super::type_mapper::{is_optional_type, map_ts_type};

impl RustGenerator {
    pub fn process_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        let struct_name = format_ident!("{}", class_name);

        // 1. Generate Struct (Properties)
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        let mut constructor: Option<&Constructor> = None;
        let mut class_fields_meta = Vec::new();

        for member in &n.class.body {
            match member {
                ClassMember::ClassProp(prop) => {
                    if let Some((field_tokens, name, is_opt)) = self.convert_prop(prop) {
                        fields.push(field_tokens);
                        class_fields_meta.push((name, is_opt));
                    }
                }
                ClassMember::Method(method) => {
                    methods.push(method);
                }
                ClassMember::Constructor(cons) => {
                    constructor = Some(cons);
                }
                _ => {}
            }
        }

        let vis = if self.is_exporting {
            quote! { pub }
        } else {
            quote! {}
        };

        let (generics_decl, generics_use) = if let Some(type_params) = &n.class.type_params {
            let params_decl: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    let ident = format_ident!("{}", name);
                    quote! { #ident: Clone }
                })
                .collect();
            let params_use: Vec<_> = type_params
                .params
                .iter()
                .map(|p| format_ident!("{}", p.name.sym.to_string()))
                .collect();
            (
                quote! { <#(#params_decl),*> },
                quote! { <#(#params_use),*> },
            )
        } else {
            (quote! {}, quote! {})
        };

        let struct_def = quote! {
            #[derive(Default, Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
            #vis struct #struct_name #generics_decl {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');

        // 2. Generate Impl (Methods)
        let mut impl_items = Vec::new();

        // Constructor
        if let Some(cons) = constructor {
            impl_items.push(self.convert_constructor(&struct_name, cons, &class_fields_meta));
        } else {
            // Default constructor if none exists
            impl_items.push(quote! {
                pub fn new() -> Self {
                    Self::default()
                }
            });
        }

        // Methods
        for method in methods {
            impl_items.push(self.convert_method(method));
        }

        let impl_block = quote! {
            impl #generics_decl #struct_name #generics_use {
                #(#impl_items)*
            }
        };

        self.code.push_str(&impl_block.to_string());
        self.code.push('\n');
    }

    fn convert_prop(
        &self,
        prop: &swc_ecma_ast::ClassProp,
    ) -> Option<(proc_macro2::TokenStream, String, bool)> {
        let field_name_str = if let Some(ident) = prop.key.as_ident() {
            ident.sym.to_string()
        } else {
            return None;
        };
        let field_name = format_ident!("{}", field_name_str);
        let mut field_type = map_ts_type(prop.type_ann.as_ref());

        let is_optional_union = is_optional_type(prop.type_ann.as_deref());
        let is_effectively_optional = prop.is_optional || is_optional_union;

        if prop.is_optional {
            // If it's optional via `?`, we wrap in Option.
            // If it's optional via union `| undefined`, map_ts_type already wraps it in Option.
            field_type = quote! { Option<#field_type> };
        }

        Some((
            quote! {
                pub #field_name: #field_type
            },
            field_name_str,
            is_effectively_optional,
        ))
    }

    fn convert_constructor(
        &self,
        _struct_name: &proc_macro2::Ident,
        constructor: &Constructor,
        class_fields: &[(String, bool)],
    ) -> proc_macro2::TokenStream {
        let mut params = Vec::new();
        for param in &constructor.params {
            if let Some(pat_param) = param.as_param() {
                if let Pat::Ident(ident) = &pat_param.pat {
                    let param_name = format_ident!("{}", ident.sym.to_string());
                    let param_type = map_ts_type(ident.type_ann.as_ref());
                    params.push(quote! { #param_name: #param_type });
                }
            }
        }

        // Try to extract field assignments from constructor body
        let mut field_inits = Vec::new();
        let mut initialized_fields = std::collections::HashSet::new();

        if let Some(body) = &constructor.body {
            for stmt in &body.stmts {
                if let Stmt::Expr(ExprStmt { expr, .. }) = stmt {
                    if let Expr::Assign(assign) = &**expr {
                        // Check if left side is this.field
                        if let AssignTarget::Simple(simple) = &assign.left {
                            if let Some(member) = simple.as_member() {
                                if member.obj.is_this() {
                                    if let Some(prop_ident) = member.prop.as_ident() {
                                        let field_name_str = prop_ident.sym.to_string();
                                        let field_name = format_ident!("{}", field_name_str);
                                        let value = convert_expr_pub(&assign.right);

                                        // If field is optional but assigned value is not Option, wrap it?
                                        // Usually in Rust constructor we assign the value directly.
                                        // But if the field is Option<T>, and we assign T, we need Some(T).
                                        // This is tricky without type info of the expression.
                                        // For now, assume user assigns correct type or we wrap in Some if it's a literal?
                                        // Actually, if TS says `this.opt = "val"`, Rust expects `Option<String>`.
                                        // We might need to wrap in `Some(...)`.
                                        // Let's check if the field is optional.
                                        let is_optional = class_fields
                                            .iter()
                                            .find(|(n, _)| n == &field_name_str)
                                            .map(|(_, opt)| *opt)
                                            .unwrap_or(false);

                                        let value = if is_optional {
                                            quote! { Some(#value) } // Naive wrapping, might double wrap if already Some
                                        } else {
                                            value
                                        };

                                        field_inits.push(quote! { #field_name: #value });
                                        initialized_fields.insert(field_name_str);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Fill in missing optional fields with None
        for (name, is_optional) in class_fields {
            if *is_optional && !initialized_fields.contains(name) {
                let field_name = format_ident!("{}", name);
                field_inits.push(quote! { #field_name: None });
            }
        }

        if !field_inits.is_empty() {
            quote! {
                pub fn new(#(#params),*) -> Self {
                    Self {
                        #(#field_inits),*
                    }
                }
            }
        } else {
            // Fallback if we can't parse constructor body
            quote! {
                pub fn new(#(#params),*) -> Self {
                    todo!("Complex constructor not yet supported")
                }
            }
        }
    }

    fn convert_method(&self, method: &swc_ecma_ast::ClassMethod) -> proc_macro2::TokenStream {
        let method_name_str = if let Some(ident) = method.key.as_ident() {
            ident.sym.to_string()
        } else {
            return quote! { /* unsupported method key */ };
        };
        let method_name = format_ident!("{}", to_snake_case(&method_name_str));

        // Build parameters - always add &self for instance methods
        let mut params = vec![quote! { &self }];
        for param in &method.function.params {
            if let Pat::Ident(ident) = &param.pat {
                let param_name = format_ident!("{}", ident.sym.to_string());
                let param_type = map_ts_type(ident.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        let return_type = if method.function.is_async {
            super::type_mapper::unwrap_promise_type(method.function.return_type.as_ref())
        } else {
            map_ts_type(method.function.return_type.as_ref())
        };

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(body) = &method.function.body {
            for stmt in &body.stmts {
                body_stmts.push(convert_stmt_pub(stmt));
            }
        }

        quote! {
            pub fn #method_name(#(#params),*) -> #return_type {
                #(#body_stmts)*
            }
        }
    }
}
