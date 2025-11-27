use quote::{format_ident, quote};
use swc_ecma_ast::{
    AssignTarget, ClassDecl, ClassMember, Constructor, Expr, ExprStmt, Lit, Pat, Stmt,
};

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

        // Check for NestJS decorators (@Get, @Post, etc.)
        let mut http_method = None;
        let mut route_path = String::new();

        for decorator in &method.function.decorators {
            if let Expr::Call(call) = &*decorator.expr {
                if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                    if let Expr::Ident(ident) = &**expr {
                        let name = ident.sym.as_str();
                        if matches!(name, "Get" | "Post" | "Put" | "Delete" | "Patch") {
                            http_method = Some(name.to_string());
                            // Extract route path if present
                            if let Some(arg) = call.args.first() {
                                if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                                    route_path = s.value.as_str().unwrap_or_default().to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        let is_handler = http_method.is_some();

        // Build parameters
        let mut params = vec![quote! { &self }];
        for param in &method.function.params {
            if let Pat::Ident(ident) = &param.pat {
                let param_name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                let param_type = map_ts_type(ident.type_ann.as_ref());

                // Check for @Body decorator on parameters
                let mut is_body = false;

                // Correctly check decorators on the Param node
                for decorator in &param.decorators {
                    if let Expr::Call(call) = &*decorator.expr {
                        if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                            if let Expr::Ident(ident) = &**expr {
                                if ident.sym == "Body" {
                                    is_body = true;
                                }
                            }
                        }
                    }
                }

                if is_body {
                    // Wrap in axum::Json
                    // Argument name needs to be destructured: axum::Json(name): axum::Json<Type>
                    // But we can't easily change the param name pattern here in the loop to a destructuring pattern
                    // without changing how we generate the function signature.
                    // For now, let's change the type to `axum::Json<T>` and keep the name.
                    // Inside the function, `name` will be `axum::Json<T>`, so we might need `name.0` to access it.
                    // OR, we use `axum::Json(param_name)` pattern in the argument list.
                    // Let's try to use the pattern matching in argument: `axum::Json(param_name): axum::Json<Type>`

                    // We need to change how we push to `params`.
                    // This requires a bit of a hack in how we construct the `quote!`.
                    // We'll construct the whole argument token stream.

                    params.push(quote! { axum::Json(#param_name): axum::Json<#param_type> });
                } else {
                    params.push(quote! { #param_name: #param_type });
                }
            }
        }

        let mut return_type = if method.function.is_async {
            super::type_mapper::unwrap_promise_type(method.function.return_type.as_ref())
        } else {
            map_ts_type(method.function.return_type.as_ref())
        };

        // If it's a handler, wrap return type in Json unless it's String
        if is_handler {
            let return_type_str = return_type.to_string();
            if return_type_str != "String" {
                return_type = quote! { axum::Json<#return_type> };
            }
        }

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(body) = &method.function.body {
            for stmt in &body.stmts {
                // We need to intercept the return statement if it's a handler
                if is_handler {
                    if let Stmt::Return(ret) = stmt {
                        if let Some(arg) = &ret.arg {
                            let expr = convert_expr_pub(arg);
                            // Check if we wrapped the return type
                            let is_wrapped = return_type.to_string().starts_with("axum :: Json");

                            if is_wrapped {
                                body_stmts.push(quote! { return axum::Json(#expr); });
                            } else {
                                body_stmts.push(quote! { return #expr; });
                            }
                            continue;
                        }
                    }
                }
                body_stmts.push(convert_stmt_pub(stmt));
            }
        }

        let fn_keyword = if is_handler || method.function.is_async {
            quote! { async fn }
        } else {
            quote! { fn }
        };

        let doc_comment = if is_handler {
            let method_str = http_method.unwrap().to_uppercase();
            let route = if route_path.is_empty() {
                "/".to_string()
            } else {
                route_path
            };
            quote! {
                #[doc = concat!("Route: ", #method_str, " ", #route)]
            }
        } else {
            quote! {}
        };

        quote! {
            #doc_comment
            pub #fn_keyword #method_name(#(#params),*) -> #return_type {
                #(#body_stmts)*
            }
        }
    }
}
