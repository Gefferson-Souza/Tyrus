use quote::{format_ident, quote};
use swc_ecma_ast::{
    AwaitExpr, BinExpr, BinaryOp, CallExpr, Callee, Expr, ExprOrSpread, FnDecl, Lit, Pat,
    ReturnStmt, Stmt,
};

use super::type_mapper::{map_ts_type, unwrap_promise_type};

impl super::interface::RustGenerator {
    pub fn visit_fn_decl(&mut self, n: &FnDecl) {
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

        let fn_def = if is_async {
            quote! {
                pub async fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        } else {
            quote! {
                pub fn #fn_ident(#(#params),*) -> #return_type {
                    #(#body_stmts)*
                }
            }
        };

        self.code.push_str(&fn_def.to_string());
        self.code.push('\n');
    }
}

fn convert_stmt(stmt: &Stmt) -> proc_macro2::TokenStream {
    match stmt {
        Stmt::Return(ret) => convert_return_stmt(ret),
        _ => quote! {}, // Skip other statements for now
    }
}

fn convert_return_stmt(ret: &ReturnStmt) -> proc_macro2::TokenStream {
    if let Some(arg) = &ret.arg {
        let expr = convert_expr(arg);
        quote! { return #expr; }
    } else {
        quote! { return; }
    }
}

fn convert_expr(expr: &Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Bin(bin) => convert_bin_expr(bin),
        Expr::Ident(ident) => {
            let ident_name = format_ident!("{}", ident.sym.to_string());
            quote! { #ident_name }
        }
        Expr::Lit(lit) => {
            // Handle numeric literals
            match lit {
                Lit::Num(num) => {
                    let value = num.value;
                    quote! { #value }
                }
                _ => quote! { todo!("non-numeric literal") },
            }
        }
        Expr::Await(await_expr) => convert_await_expr(await_expr),
        Expr::Call(call_expr) => convert_call_expr(call_expr),
        _ => quote! { todo!() },
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
        _ => quote! { /* unsupported op */ },
    };

    quote! { #left #op #right }
}

fn convert_await_expr(await_expr: &AwaitExpr) -> proc_macro2::TokenStream {
    let inner = convert_expr(&await_expr.arg);
    quote! { #inner.await }
}

fn convert_call_expr(call: &CallExpr) -> proc_macro2::TokenStream {
    let callee = match &call.callee {
        Callee::Expr(expr) => convert_expr(expr),
        _ => quote! { unknown_callee },
    };

    let args: Vec<_> = call.args.iter().map(convert_expr_or_spread).collect();

    quote! { #callee(#(#args),*) }
}

fn convert_expr_or_spread(arg: &ExprOrSpread) -> proc_macro2::TokenStream {
    convert_expr(&arg.expr)
}

fn to_snake_case(s: &str) -> String {
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
