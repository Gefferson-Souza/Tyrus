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

        // Collect fields from constructor (private/public params)
        if let Some(cons) = constructor {
            for param in &cons.params {
                if let swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) = param {
                    if let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param {
                        let field_name_str = ident.sym.to_string();
                        let field_name = format_ident!("{}", to_snake_case(&field_name_str));

                        let type_ann = ident.type_ann.as_ref();
                        let mut field_type = map_ts_type(type_ann);

                        // Heuristic: If it's a TypeRef (not primitive), wrap in Arc
                        let is_dependency = if let Some(ann) = type_ann {
                            if let Some(_type_ref) = ann.type_ann.as_ts_type_ref() {
                                let type_str = field_type.to_string();
                                !matches!(
                                    type_str.as_str(),
                                    "String" | "f64" | "bool" | "i32" | "Vec" | "Option"
                                )
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_dependency {
                            field_type = quote! { std::sync::Arc<#field_type> };
                        }

                        fields.push(quote! { pub #field_name: #field_type });
                    }
                }
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
        let mut routes: Vec<(String, String, String)> = Vec::new();
        for method in methods {
            let (method_tokens, route_info) = self.convert_method(method);
            impl_items.push(method_tokens);
            if let Some(info) = route_info {
                routes.push(info);
            }
        }

        // Generate router() if it's a controller
        // Check for @Controller decorator
        let mut is_controller = false;
        let mut controller_path = String::new();

        for decorator in &n.class.decorators {
            if let Expr::Call(call) = &*decorator.expr {
                if let swc_ecma_ast::Callee::Expr(expr) = &call.callee {
                    if let Expr::Ident(ident) = &**expr {
                        if ident.sym == "Controller" {
                            is_controller = true;
                            if let Some(arg) = call.args.first() {
                                if let Expr::Lit(Lit::Str(s)) = &*arg.expr {
                                    controller_path =
                                        s.value.as_str().unwrap_or_default().to_string();
                                }
                            }
                        }
                    }
                }
            }
        }

        if is_controller {
            // Generate FromRequestParts implementation to allow `self` injection
            // impl<S> axum::extract::FromRequestParts<S> for CatsController
            // where S: Send + Sync
            // {
            //     type Rejection = std::convert::Infallible;
            //     async fn from_request_parts(_parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
            //         let axum::Extension(controller) = axum::Extension::<std::sync::Arc<Self>>::from_request_parts(_parts, state).await.unwrap_or_default();
            //         Ok(controller.as_ref().clone())
            //     }
            // }

            let from_request_impl = quote! {
                #[axum::async_trait]
                impl<S> axum::extract::FromRequestParts<S> for #struct_name
                where S: Send + Sync
                {
                    type Rejection = std::convert::Infallible;
                    async fn from_request_parts(parts: &mut axum::http::request::Parts, state: &S) -> Result<Self, Self::Rejection> {
                        let axum::Extension(controller) = axum::Extension::<std::sync::Arc<Self>>::from_request_parts(parts, state)
                            .await
                            .expect("Controller extension missing");
                        Ok(controller.as_ref().clone())
                    }
                }
            };

            self.code.push_str(&from_request_impl.to_string());
            self.code.push('\n');

            // Generate router function
            // pub fn router() -> axum::Router {
            //     axum::Router::new()
            //         .route("/path", get(handler))
            //         .route("/path2", post(handler2))
            // }

            let mut route_calls = Vec::new();
            for (method_name, http_method, path) in &routes {
                let method_ident = format_ident!("{}", method_name);
                let axum_method = match http_method.as_str() {
                    "Get" => quote! { get },
                    "Post" => quote! { post },
                    "Put" => quote! { put },
                    "Delete" => quote! { delete },
                    "Patch" => quote! { patch },
                    _ => quote! { get },
                };

                // Combine controller path and method path
                // Controller: "cats", Method: "/" -> "/cats"
                // Controller: "cats", Method: "/:id" -> "/cats/:id"

                let full_path = if controller_path.is_empty() {
                    path.clone()
                } else {
                    let c_path = controller_path.trim_matches('/');
                    let m_path = path.trim_matches('/');
                    if m_path.is_empty() {
                        format!("/{}", c_path)
                    } else {
                        format!("/{}/{}", c_path, m_path)
                    }
                };

                // Ensure starts with /
                let full_path = if full_path.starts_with('/') {
                    full_path
                } else {
                    format!("/{}", full_path)
                };

                route_calls.push(quote! {
                    .route(#full_path, axum::routing::#axum_method(Self::#method_ident))
                });
            }

            // Note: We don't add .layer(Extension(Self::default())) here anymore because
            // we expect the controller to be fully constructed with dependencies in main.rs
            // and passed as an extension there.
            // Actually, if we use Self::default(), we bypass DI.
            // So we should REMOVE Self::default() from here if we want DI.
            // But for now, let's keep it but assume it will be overridden or unused if we inject properly in main.
            // Wait, if we add .layer(...) here, it overrides outer layers?
            // Axum layers are applied outside-in.
            // If main.rs does .merge(router).layer(Extension(controller)), that Extension is available to the router.
            // If router() does .layer(Extension(default)), that might shadow the one from main.
            // So we should REMOVE .layer(Extension(Self::default())) from here!
            // The controller instance should be provided by the caller (main.rs).

            impl_items.push(quote! {
                pub fn router() -> axum::Router {
                    axum::Router::new()
                        #(#route_calls)*
                }
            });

            // Add to metadata
            self.controllers.push(crate::ControllerMetadata {
                struct_name: class_name.clone(),
                route_path: controller_path.clone(),
            });
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
        let mut field_inits = Vec::new();
        let mut initialized_fields = std::collections::HashSet::new();

        for param in &constructor.params {
            match param {
                swc_ecma_ast::ParamOrTsParamProp::TsParamProp(prop) => {
                    if let swc_ecma_ast::TsParamPropParam::Ident(ident) = &prop.param {
                        let param_name_str = ident.sym.to_string();
                        let param_name = format_ident!("{}", to_snake_case(&param_name_str));

                        let type_ann = ident.type_ann.as_ref();
                        let mut param_type = map_ts_type(type_ann);

                        // Heuristic: If it's a TypeRef (not primitive), wrap in Arc
                        let is_dependency = if let Some(ann) = type_ann {
                            if let Some(_type_ref) = ann.type_ann.as_ts_type_ref() {
                                let type_str = param_type.to_string();
                                !matches!(
                                    type_str.as_str(),
                                    "String" | "f64" | "bool" | "i32" | "Vec" | "Option"
                                )
                            } else {
                                false
                            }
                        } else {
                            false
                        };

                        if is_dependency {
                            param_type = quote! { std::sync::Arc<#param_type> };
                        }

                        params.push(quote! { #param_name: #param_type });
                        field_inits.push(quote! { #param_name: #param_name });
                        initialized_fields.insert(param_name_str);
                    }
                }
                swc_ecma_ast::ParamOrTsParamProp::Param(pat_param) => {
                    if let Pat::Ident(ident) = &pat_param.pat {
                        let param_name = format_ident!("{}", ident.sym.to_string());
                        let param_type = map_ts_type(ident.type_ann.as_ref());
                        params.push(quote! { #param_name: #param_type });
                    }
                }
            }
        }

        // Try to extract field assignments from constructor body
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

    fn convert_method(
        &self,
        method: &swc_ecma_ast::ClassMethod,
    ) -> (proc_macro2::TokenStream, Option<(String, String, String)>) {
        let method_name_str = if let Some(ident) = method.key.as_ident() {
            ident.sym.to_string()
        } else {
            return (quote! { /* unsupported method key */ }, None);
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
        let mut params = Vec::new();

        // Handle self parameter
        if is_handler {
            // For handlers, we consume self (injected via FromRequest)
            params.push(quote! { self });
        } else {
            // For regular methods, use &self
            params.push(quote! { &self });
        }

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
                                body_stmts.push(quote! { return axum::Json(#expr.into()); });
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
            let method_str = if let Some(m) = http_method.as_ref() {
                m.to_uppercase()
            } else {
                "GET".to_string() // Default or unreachable if guarded
            };
            let route = if route_path.is_empty() {
                "/".to_string()
            } else {
                route_path.clone()
            };
            quote! {
                #[doc = concat!("Route: ", #method_str, " ", #route)]
            }
        } else {
            quote! {}
        };

        let tokens = quote! {
            #doc_comment
            pub #fn_keyword #method_name(#(#params),*) -> #return_type {
                #(#body_stmts)*
            }
        };

        let route_info = http_method.map(|method| (method_name.to_string(), method, route_path));

        (tokens, route_info)
    }
}
