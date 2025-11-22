use quote::{format_ident, quote};
use swc_ecma_ast::{AssignTarget, ClassDecl, ClassMember, Constructor, Expr, ExprStmt, Pat, Stmt};

use super::func::{convert_expr_pub, convert_stmt_pub};
use super::interface::RustGenerator;
use super::type_mapper::map_ts_type;

impl RustGenerator {
    pub fn visit_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        let struct_name = format_ident!("{}", class_name);

        // Step 1: Extract properties for struct
        let mut fields = Vec::new();
        for member in &n.class.body {
            if let ClassMember::ClassProp(prop) = member {
                let field_name_str = if let Some(ident) = prop.key.as_ident() {
                    ident.sym.to_string()
                } else {
                    continue;
                };
                let field_name = format_ident!("{}", field_name_str);
                let field_type = map_ts_type(prop.type_ann.as_ref());

                fields.push(quote! {
                    pub #field_name: #field_type
                });
            }
        }

        // Generate struct
        let struct_def = quote! {
            #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
            pub struct #struct_name {
                #(#fields),*
            }
        };

        self.code.push_str(&struct_def.to_string());
        self.code.push('\n');

        // Step 2: Extract methods for impl block
        let mut methods = Vec::new();

        for member in &n.class.body {
            match member {
                ClassMember::Constructor(constructor) => {
                    let method_def = self.convert_constructor(&struct_name, constructor);
                    methods.push(method_def);
                }
                ClassMember::Method(method) => {
                    let method_def = self.convert_method(method);
                    methods.push(method_def);
                }
                _ => {}
            }
        }

        if !methods.is_empty() {
            let impl_block = quote! {
                impl #struct_name {
                    #(#methods)*
                }
            };

            self.code.push_str(&impl_block.to_string());
            self.code.push('\n');
        }
    }

    fn convert_constructor(
        &self,
        struct_name: &proc_macro2::Ident,
        constructor: &Constructor,
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
        if let Some(body) = &constructor.body {
            for stmt in &body.stmts {
                if let Stmt::Expr(ExprStmt { expr, .. }) = stmt {
                    if let Expr::Assign(assign) = &**expr {
                        // Check if left side is this.field
                        if let AssignTarget::Simple(simple) = &assign.left {
                            if let Some(member) = simple.as_member() {
                                if member.obj.is_this() {
                                    if let Some(prop_ident) = member.prop.as_ident() {
                                        let field_name =
                                            format_ident!("{}", prop_ident.sym.to_string());
                                        let value = convert_expr_pub(&assign.right);
                                        field_inits.push(quote! { #field_name: #value });
                                    }
                                }
                            }
                        }
                    }
                }
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
        let method_name = format_ident!("{}", method_name_str);

        // Build parameters - always add &self for instance methods
        let mut params = vec![quote! { &self }];
        for param in &method.function.params {
            if let Pat::Ident(ident) = &param.pat {
                let param_name = format_ident!("{}", ident.sym.to_string());
                let param_type = map_ts_type(ident.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        let return_type = map_ts_type(method.function.return_type.as_ref());

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
