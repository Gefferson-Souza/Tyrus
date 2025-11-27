use quote::{format_ident, quote};
use swc_ecma_ast::{
    AwaitExpr, BinExpr, BinaryOp, CallExpr, Callee, Decl, Expr, ExprOrSpread, FnDecl, Lit,
    MemberExpr, Pat, Stmt,
};

use super::type_mapper::{map_ts_type, unwrap_promise_type};

impl super::interface::RustGenerator {
    pub fn process_fn_decl(&mut self, n: &FnDecl) {
        let fn_name = to_snake_case(&n.ident.sym);
        let fn_ident = format_ident!("{}", fn_name);

        // Check if async
        let is_async = n.function.is_async;

        // Extract parameters
        let mut params = Vec::new();
        for param in &n.function.params {
            if let Pat::Ident(ident_pat) = &param.pat {
                let param_name = format_ident!("{}", ident_pat.sym.to_string());
                let param_type = map_ts_type(ident_pat.type_ann.as_ref());
                params.push(quote! { #param_name: #param_type });
            }
        }

        // Extract return type - unwrap Promise<T> for async functions
        let return_type = if is_async {
            unwrap_promise_type(n.function.return_type.as_ref())
        } else {
            map_ts_type(n.function.return_type.as_ref())
        };

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(block_stmt) = &n.function.body {
            for stmt in &block_stmt.stmts {
                body_stmts.push(convert_stmt(stmt));
            }
        }

        let vis = if self.is_exporting {
            quote! { pub }
        } else {
            quote! {}
        };

        let fn_def = if is_async {
            quote! {
                #vis async fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        } else {
            quote! {
                #vis fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        };

        self.code.push_str(&fn_def.to_string());
        self.code.push('\n');
    }
}

pub fn convert_stmt_pub(stmt: &Stmt) -> proc_macro2::TokenStream {
    convert_stmt(stmt)
}

pub fn convert_expr_pub(expr: &Expr) -> proc_macro2::TokenStream {
    convert_expr(expr)
}

pub fn convert_stmt(stmt: &Stmt) -> proc_macro2::TokenStream {
    match stmt {
        Stmt::Return(ret_stmt) => {
            if let Some(arg) = &ret_stmt.arg {
                let expr = convert_expr(arg);
                quote! { return #expr; }
            } else {
                quote! { return; }
            }
        }
        Stmt::Expr(expr_stmt) => {
            let expr = convert_expr(&expr_stmt.expr);
            quote! { #expr; }
        }
        Stmt::Decl(Decl::Var(var_decl)) => {
            // Handle variable declarations (const/let)
            let mut declarations = Vec::new();
            for decl in &var_decl.decls {
                if let Pat::Ident(ident) = &decl.name {
                    let var_name = to_snake_case(&ident.id.sym);
                    let var_ident = format_ident!("{}", var_name);

                    if let Some(init) = &decl.init {
                        let init_expr = convert_expr(init);
                        declarations.push(quote! {
                            let #var_ident = #init_expr;
                        });
                    } else {
                        // Uninitialized variable - maybe let x; -> let mut x; (but we need type)
                        // For now, skip or generate todo
                        declarations.push(quote! {
                            let #var_ident; // This might fail in Rust if type not inferred
                        });
                    }
                }
            }
            quote! {
                #(#declarations)*
            }
        }
        Stmt::Block(block) => {
            let stmts: Vec<_> = block.stmts.iter().map(convert_stmt).collect();
            quote! {
                {
                    #(#stmts)*
                }
            }
        }
        Stmt::If(if_stmt) => {
            let test = convert_expr(&if_stmt.test);
            let cons = convert_stmt(&if_stmt.cons);

            let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                quote! { #cons }
            } else {
                quote! { { #cons } }
            };

            let alt = if let Some(alt) = &if_stmt.alt {
                let alt_stmt = convert_stmt(alt);
                let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                    quote! { #alt_stmt }
                } else {
                    quote! { { #alt_stmt } }
                };
                quote! { else #alt_block }
            } else {
                quote! {}
            };

            quote! {
                if #test #cons_block #alt
            }
        }
        _ => quote! { /* unsupported statement */ },
    }
}

pub fn convert_expr(expr: &Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Bin(bin) => convert_bin_expr(bin),
        Expr::Ident(ident) => {
            let name = ident.sym.as_str();
            // If starts with uppercase, assume Class/Type and keep as is
            // If starts with lowercase, convert to snake_case (variable/function)
            if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                let ident_token = format_ident!("{}", name);
                quote! { #ident_token }
            } else {
                let ident_name = to_snake_case(name);
                let ident_token = format_ident!("{}", ident_name);
                quote! { #ident_token }
            }
        }
        Expr::Lit(lit) => {
            // Handle literals
            match lit {
                Lit::Num(num) => {
                    let value = num.value;
                    quote! { #value }
                }
                Lit::Str(str_lit) => {
                    let s = str_lit.value.as_str().unwrap_or("");
                    quote! { String::from(#s) }
                }
                _ => quote! { todo!("unsupported literal") },
            }
        }
        Expr::Member(member) => convert_member_expr(member),
        Expr::Await(await_expr) => convert_await_expr(await_expr),
        Expr::Call(call_expr) => convert_call_expr(call_expr),
        Expr::New(new_expr) => convert_new_expr(new_expr),
        _ => quote! { todo!() },
    }
}

fn convert_new_expr(new_expr: &swc_ecma_ast::NewExpr) -> proc_macro2::TokenStream {
    // Convert new Class(args) -> Class::new(args)
    let callee = convert_expr(&new_expr.callee);
    let args = if let Some(args) = &new_expr.args {
        args.iter().map(convert_expr_or_spread).collect()
    } else {
        Vec::new()
    };

    quote! { #callee::new(#(#args),*) }
}

fn convert_member_expr(member: &MemberExpr) -> proc_macro2::TokenStream {
    // Handle this.prop -> self.prop
    if member.obj.is_this() {
        if let Some(prop_ident) = member.prop.as_ident() {
            let field = format_ident!("{}", prop_ident.sym.to_string());
            return quote! { self.#field };
        }
    }
    // Handle other.prop
    let obj = convert_expr(&member.obj);
    if let Some(prop_ident) = member.prop.as_ident() {
        let prop = format_ident!("{}", prop_ident.sym.to_string());
        quote! { #obj.#prop }
    } else {
        quote! { todo!("complex member access") }
    }
}

fn convert_bin_expr(bin: &BinExpr) -> proc_macro2::TokenStream {
    let left = convert_expr(&bin.left);
    let right = convert_expr(&bin.right);

    let op = match bin.op {
        BinaryOp::Add => quote! { + },
        BinaryOp::Sub => quote! { - },
        BinaryOp::Mul => quote! { * },
        BinaryOp::Div => quote! { / },
        BinaryOp::EqEq => quote! { == },
        BinaryOp::NotEq => quote! { != },
        BinaryOp::Lt => quote! { < },
        BinaryOp::LtEq => quote! { <= },
        BinaryOp::Gt => quote! { > },
        BinaryOp::GtEq => quote! { >= },
        BinaryOp::LogicalAnd => quote! { && },
        BinaryOp::LogicalOr => quote! { || },
        _ => quote! { /* unsupported op */ },
    };

    quote! { #left #op #right }
}

fn convert_await_expr(await_expr: &AwaitExpr) -> proc_macro2::TokenStream {
    let inner = convert_expr(&await_expr.arg);
    quote! { #inner.await }
}

fn convert_call_expr(call: &CallExpr) -> proc_macro2::TokenStream {
    // Try stdlib handlers first
    if let Some(stdlib_code) = crate::stdlib::try_handle_stdlib_call(&call.callee, &call.args) {
        return stdlib_code;
    }

    // Check for axios calls (axios.get, axios.post, etc.)
    if let Callee::Expr(expr) = &call.callee {
        if let Expr::Member(member) = &**expr {
            // Try stdlib method call
            if let Some(method_ident) = member.prop.as_ident() {
                let method_name = method_ident.sym.as_ref();
                if let Some(stdlib_code) =
                    crate::stdlib::try_handle_method_call(&member.obj, method_name, &call.args)
                {
                    return stdlib_code;
                }
            }

            // Check if object is "axios"
            if let Expr::Ident(obj_ident) = &*member.obj {
                if obj_ident.sym == "axios" {
                    // Get the HTTP method
                    if let Some(method_ident) = member.prop.as_ident() {
                        let method = method_ident.sym.to_string();
                        return convert_axios_call(&method, &call.args);
                    }
                }
            }
        }

        // Check for fetch calls
        if let Expr::Ident(ident) = &**expr {
            if ident.sym == "fetch" {
                return convert_fetch_call(&call.args);
            }
        }
    }

    // Fallback to generic call conversion
    let callee = match &call.callee {
        Callee::Expr(expr) => convert_expr(expr),
        _ => quote! { unknown_callee },
    };

    let args: Vec<_> = call.args.iter().map(convert_expr_or_spread).collect();

    quote! { #callee(#(#args),*) }
}

fn convert_axios_call(method: &str, args: &[ExprOrSpread]) -> proc_macro2::TokenStream {
    let method_lower = method.to_lowercase();
    let method_ident = format_ident!("{}", method_lower);

    if args.is_empty() {
        return quote! { reqwest::Client::new().#method_ident("").send().await? };
    }

    // First argument is the URL
    let url = convert_expr_or_spread(&args[0]);

    // For POST/PUT, second argument might be data
    if (method_lower == "post" || method_lower == "put") && args.len() > 1 {
        let data = convert_expr_or_spread(&args[1]);
        quote! {
            reqwest::Client::new()
                .#method_ident(#url)
                .json(&#data)
                .send()
                .await?
        }
    } else {
        // GET/DELETE or POST/PUT without body
        quote! {
            reqwest::Client::new()
                .#method_ident(#url)
                .send()
                .await?
        }
    }
}

fn convert_fetch_call(args: &[ExprOrSpread]) -> proc_macro2::TokenStream {
    if args.is_empty() {
        return quote! { reqwest::get("").await? };
    }

    let url = convert_expr_or_spread(&args[0]);
    quote! { reqwest::get(#url).await? }
}

pub fn convert_expr_or_spread(arg: &ExprOrSpread) -> proc_macro2::TokenStream {
    convert_expr(&arg.expr)
}

pub fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.chars().enumerate() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap_or(ch));
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case_simple() {
        assert_eq!(to_snake_case("fetchData"), "fetch_data");
        assert_eq!(to_snake_case("getUserName"), "get_user_name");
        assert_eq!(to_snake_case("HTTPRequest"), "h_t_t_p_request");
    }

    #[test]
    fn test_to_snake_case_already_snake() {
        assert_eq!(to_snake_case("already_snake"), "already_snake");
    }

    #[test]
    fn test_to_snake_case_single_word() {
        assert_eq!(to_snake_case("simple"), "simple");
        assert_eq!(to_snake_case("Simple"), "simple");
    }

    #[test]
    fn test_to_snake_case_empty() {
        assert_eq!(to_snake_case(""), "");
    }
}
