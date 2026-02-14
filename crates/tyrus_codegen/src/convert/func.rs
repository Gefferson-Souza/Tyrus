use quote::{format_ident, quote};
use swc_ecma_ast::{
    AwaitExpr, BinExpr, BinaryOp, CallExpr, Callee, Decl, Expr, ExprOrSpread, FnDecl, Lit,
    MemberExpr, Pat, Stmt, UpdateExpr, UpdateOp,
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
            if n.function.return_type.is_none() {
                // If no return type, assume void for async functions
                quote! { Result<(), crate::AppError> }
            } else {
                let inner = unwrap_promise_type(n.function.return_type.as_ref());
                quote! { Result<#inner, crate::AppError> }
            }
        } else if n.function.return_type.is_none() {
            quote! { () }
        } else {
            map_ts_type(n.function.return_type.as_ref())
        };

        // Check if void
        let is_void = if n.function.return_type.is_none() {
            true
        } else {
            super::type_mapper::is_void_or_promise_void(n.function.return_type.as_deref())
        };

        // Convert body
        let mut body_stmts = Vec::new();
        if let Some(block_stmt) = &n.function.body {
            if is_async {
                // Use recursive converter to handle return Ok(...)
                for stmt in &block_stmt.stmts {
                    body_stmts.push(convert_stmt_recursive(stmt, &|ret_stmt| {
                        if let Some(arg) = &ret_stmt.arg {
                            let expr = convert_expr(arg);

                            // Heuristic: If returning an object literal, it is converted to json!() (Value).
                            // But the function might return a struct (User).
                            // We need to deserialize Value -> Struct.
                            if !is_void && matches!(arg.as_ref(), swc_ecma_ast::Expr::Object(_)) {
                                quote! {
                                    return Ok(serde_json::from_value(#expr).unwrap_or_else(|e| panic!("Failed to convert return value: {}", e)));
                                }
                            } else {
                                quote! { return Ok(#expr); }
                            }
                        } else {
                            quote! { return Ok(()); }
                        }
                    }));
                }

                if is_void {
                    // Fallback for void functions
                    // Only append if the last statement isn't a return (though rustc handles unreachable code)
                    // But strictly, we need Ok(()) if control flow reaches end.
                    // Since we can't easily analyze control flow, appending Ok(()) is safe for void functions.
                    // But we must ensure it doesn't cause "unreachable expression" warnings if possible,
                    // or just accept the warning.
                    // The previous error was "expected bool, found ()".
                    // If we only append when is_void is true, we avoid that error.
                    // We might get "unreachable code" warning if there was an explicit return before, but that's fine (just a warning).
                    // Ideally we suppress it or check, but let's just append.
                    // Actually, to avoid "unreachable expression" warning which might be treated as error in some configs:
                    // We can't easily avoid it without CFG.
                    // Let's just append it.
                }
            } else {
                for stmt in &block_stmt.stmts {
                    body_stmts.push(convert_stmt_recursive(stmt, &|ret_stmt| {
                        if let Some(arg) = &ret_stmt.arg {
                            let expr = convert_expr(arg);
                            // Heuristic: same as async, needed for Struct return types
                            if !is_void && matches!(arg.as_ref(), swc_ecma_ast::Expr::Object(_)) {
                                quote! {
                                    return serde_json::from_value(#expr).unwrap_or_else(|e| panic!("Failed to convert return value: {}", e));
                                }
                            } else {
                                quote! { return #expr; }
                            }
                        } else {
                            quote! { return; }
                        }
                    }));
                }
            }
        }

        let vis = if self.is_exporting {
            quote! { pub }
        } else {
            quote! {}
        };

        let generics = if let Some(type_params) = &n.function.type_params {
            let params: Vec<_> = type_params
                .params
                .iter()
                .map(|p| {
                    let name = p.name.sym.to_string();
                    let ident = format_ident!("{}", name);
                    quote! { #ident: serde::de::DeserializeOwned + serde::Serialize + Clone }
                })
                .collect();
            quote! { <#(#params),*> }
        } else {
            quote! {}
        };

        let fn_def = if is_async {
            let fallback = if is_void {
                quote! { Ok(()) }
            } else {
                quote! {}
            };

            quote! {
                #vis async fn #fn_ident #generics (#(#params),*) -> #return_type {
                    #(#body_stmts)*
                    #fallback
                }
            }
        } else {
            quote! {
                #vis fn #fn_ident #generics (#(#params),*) -> #return_type {
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
                        // Use `let` for `const` (immutable) and `let mut` for `let`/`var`
                        if matches!(var_decl.kind, swc_ecma_ast::VarDeclKind::Const) {
                            declarations.push(quote! {
                                let #var_ident = #init_expr;
                            });
                        } else {
                            declarations.push(quote! {
                                let mut #var_ident = #init_expr;
                            });
                        }
                    } else {
                        // Uninitialized variable — always mut
                        declarations.push(quote! {
                            let mut #var_ident;
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
        Stmt::While(while_stmt) => {
            let test = convert_expr(&while_stmt.test);
            let body = convert_stmt(&while_stmt.body);
            let body_block = if matches!(*while_stmt.body, Stmt::Block(_)) {
                quote! { #body }
            } else {
                quote! { { #body } }
            };
            quote! {
                while #test #body_block
            }
        }
        _ => quote! { /* unsupported statement */ },
    }
}

pub fn convert_expr(expr: &Expr) -> proc_macro2::TokenStream {
    match expr {
        Expr::Bin(bin) => convert_bin_expr(bin),
        Expr::This(_) => quote! { self },
        Expr::Ident(ident) => {
            let name = ident.sym.as_str();
            // Handle special identifiers
            if name == "undefined" {
                return quote! { None };
            }
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
                Lit::Bool(b) => {
                    let value = b.value;
                    quote! { #value }
                }
                _ => quote! { todo!("unsupported literal") },
            }
        }
        Expr::Member(member) => convert_member_expr(member),
        Expr::Await(await_expr) => convert_await_expr(await_expr),
        Expr::Call(call_expr) => convert_call_expr(call_expr),
        Expr::New(new_expr) => convert_new_expr(new_expr),
        Expr::Tpl(tpl) => convert_tpl_expr(tpl),
        Expr::Arrow(arrow) => convert_arrow_expr(arrow),
        Expr::Object(obj) => convert_object_lit(obj),
        Expr::Array(arr) => convert_array_lit(arr),
        Expr::Update(update) => convert_update_expr(update),
        Expr::Assign(assign) => convert_assign_expr(assign),
        Expr::Unary(unary) => {
            let arg = convert_expr(&unary.arg);
            match unary.op {
                swc_ecma_ast::UnaryOp::Bang => quote! { !#arg },
                swc_ecma_ast::UnaryOp::Minus => quote! { -#arg },
                swc_ecma_ast::UnaryOp::Plus => quote! { +#arg },
                _ => quote! { todo!("unsupported unary op") },
            }
        }
        _ => quote! { todo!() },
    }
}

fn convert_assign_expr(assign: &swc_ecma_ast::AssignExpr) -> proc_macro2::TokenStream {
    let right = convert_expr(&assign.right);
    let left = match &assign.left {
        swc_ecma_ast::AssignTarget::Simple(simple) => match simple {
            swc_ecma_ast::SimpleAssignTarget::Ident(ident) => {
                let name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                quote! { #name }
            }
            swc_ecma_ast::SimpleAssignTarget::Member(member) => {
                // Handle LHS member access (no clone)
                let obj = convert_expr(&member.obj);
                match &member.prop {
                    swc_ecma_ast::MemberProp::Ident(ident) => {
                        let prop = format_ident!("{}", ident.sym.as_ref().to_string());
                        quote! { #obj.#prop }
                    }
                    swc_ecma_ast::MemberProp::Computed(computed) => {
                        if let swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Num(num)) =
                            &*computed.expr
                        {
                            let idx = num.value as usize;
                            quote! { #obj[#idx] }
                        } else {
                            let prop = convert_expr(&computed.expr);
                            quote! { #obj[#prop] }
                        }
                    }
                    _ => quote! { todo!("complex member assignment") },
                }
            }
            _ => quote! { todo!("unsupported assign target") },
        },
        _ => quote! { todo!("unsupported assign target") },
    };

    match assign.op {
        swc_ecma_ast::AssignOp::Assign => quote! { #left = #right },
        swc_ecma_ast::AssignOp::AddAssign => quote! { #left += #right },
        swc_ecma_ast::AssignOp::SubAssign => quote! { #left -= #right },
        _ => quote! { todo!("unsupported assign op") },
    }
}

fn convert_update_expr(update: &UpdateExpr) -> proc_macro2::TokenStream {
    let arg = convert_expr(&update.arg);
    match update.op {
        UpdateOp::PlusPlus => quote! { #arg += 1.0 },
        UpdateOp::MinusMinus => quote! { #arg -= 1.0 },
    }
}

fn convert_object_lit(obj: &swc_ecma_ast::ObjectLit) -> proc_macro2::TokenStream {
    let mut fields = Vec::new();
    for prop in &obj.props {
        if let swc_ecma_ast::PropOrSpread::Prop(prop) = prop {
            match &**prop {
                swc_ecma_ast::Prop::KeyValue(prop) => {
                    let key = match &prop.key {
                        swc_ecma_ast::PropName::Ident(ident) => {
                            format!("{:?}", ident.sym).trim_matches('"').to_string()
                        }
                        swc_ecma_ast::PropName::Str(s) => {
                            format!("{:?}", s.value).trim_matches('"').to_string()
                        }
                        swc_ecma_ast::PropName::Num(n) => n.to_string(),
                        swc_ecma_ast::PropName::Computed(_) => "computed_key".to_string(), // TODO
                        swc_ecma_ast::PropName::BigInt(n) => n.value.to_string(),
                    };

                    let value = if let Expr::Ident(ident) = prop.value.as_ref() {
                        if ident.sym == "undefined" {
                            quote! { serde_json::Value::Null }
                        } else {
                            convert_expr(&prop.value)
                        }
                    } else {
                        convert_expr(&prop.value)
                    };

                    fields.push(quote! { #key: #value });
                }
                swc_ecma_ast::Prop::Shorthand(ident) => {
                    let key = format!("{:?}", ident.sym).trim_matches('"').to_string();
                    let value = format_ident!("{}", to_snake_case(&key));
                    fields.push(quote! { #key: #value });
                }
                _ => {}
            }
        }
    }
    quote! { serde_json::json!({ #(#fields),* }) }
}

fn convert_array_lit(arr: &swc_ecma_ast::ArrayLit) -> proc_macro2::TokenStream {
    let elems: Vec<_> = arr
        .elems
        .iter()
        .map(|elem| {
            if let Some(elem) = elem {
                convert_expr_or_spread(elem)
            } else {
                quote! { serde_json::Value::Null }
            }
        })
        .collect();
    quote! { vec![#(#elems),*] }
}

fn convert_tpl_expr(tpl: &swc_ecma_ast::Tpl) -> proc_macro2::TokenStream {
    let mut format_str = String::new();
    let mut args = Vec::new();

    let quasis = &tpl.quasis;
    let exprs = &tpl.exprs;

    for (i, quasi) in quasis.iter().enumerate() {
        format_str.push_str(&quasi.raw);
        if i < exprs.len() {
            format_str.push_str("{}");
            args.push(convert_expr(&exprs[i]));
        }
    }

    quote! { format!(#format_str, #(#args),*) }
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
            let prop_name = to_snake_case(prop_ident.sym.as_ref());
            let field = format_ident!("{}", prop_name);
            return quote! { self.#field.clone() };
        }
    }
    // Handle other.prop or other[prop]
    let obj = convert_expr(&member.obj);

    match &member.prop {
        swc_ecma_ast::MemberProp::Ident(ident) => {
            let prop_name = to_snake_case(ident.sym.as_ref());
            let prop = format_ident!("{}", prop_name);
            quote! { #obj.#prop }
        }
        swc_ecma_ast::MemberProp::Computed(computed) => {
            // Check if expr is number literal
            if let swc_ecma_ast::Expr::Lit(swc_ecma_ast::Lit::Num(num)) = &*computed.expr {
                let idx = num.value as usize;
                quote! { #obj[#idx] }
            } else {
                let prop = convert_expr(&computed.expr);
                quote! { #obj[#prop] }
            }
        }
        _ => quote! { todo!("complex member access") },
    }
}

pub fn convert_bin_expr(bin: &BinExpr) -> proc_macro2::TokenStream {
    let left = convert_expr(&bin.left);
    let mut right = convert_expr(&bin.right);

    if bin.op == BinaryOp::Add {
        // Check if left is string
        let mut is_left_string = false;
        let mut left_expr = &*bin.left;
        while let Expr::Paren(p) = left_expr {
            left_expr = &p.expr;
        }

        if let Expr::Call(call) = left_expr {
            if let Callee::Expr(callee) = &call.callee {
                if let Expr::Member(member) = &**callee {
                    if let Some(obj) = member.obj.as_ident() {
                        if obj.sym == "String" {
                            if let Some(prop) = member.prop.as_ident() {
                                if prop.sym == "from" {
                                    is_left_string = true;
                                }
                            }
                        }
                    }
                }
            }
        } else if let Expr::Lit(Lit::Str(_)) = left_expr {
            is_left_string = true;
        }

        let mut handled = false;

        // Heuristic: If right side is a string method call, borrow it to allow String + &String
        if let Expr::Call(call) = &*bin.right {
            if let Callee::Expr(callee_expr) = &call.callee {
                if let Expr::Member(member) = &**callee_expr {
                    if let Some(ident) = member.prop.as_ident() {
                        let method_name = ident.sym.as_ref();
                        match method_name {
                            "toString" | "toUpperCase" | "toLowerCase" | "trim" | "replace"
                            | "join" | "repeat" | "slice" | "substring" | "substr" => {
                                right = quote! { &#right };
                                handled = true;
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        if !handled {
            if is_left_string {
                let mut right_expr = &*bin.right;
                while let Expr::Paren(p) = right_expr {
                    right_expr = &p.expr;
                }

                // If right is LitNum, .to_string().
                if let Expr::Lit(Lit::Num(_)) = right_expr {
                    right = quote! { &#right.to_string() };
                }
                // If right is a call (e.g. round), .to_string().
                else if let Expr::Call(_) = right_expr {
                    right = quote! { &#right.to_string() };
                }
                // If right is Ident, borrow it.
                else if let Expr::Ident(_) = right_expr {
                    right = quote! { &#right };
                }
            } else if let Expr::Ident(_) = &*bin.right {
                // Heuristic: If right side is an identifier, borrow it.
                right = quote! { &#right };
            }
        }
    }

    let op = match bin.op {
        BinaryOp::Add => quote! { + },
        BinaryOp::Sub => quote! { - },
        BinaryOp::Mul => quote! { * },
        BinaryOp::Div => quote! { / },
        BinaryOp::EqEq | BinaryOp::EqEqEq => quote! { == },
        BinaryOp::NotEq | BinaryOp::NotEqEq => quote! { != },
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

pub fn convert_stmt_recursive<F>(stmt: &Stmt, handler: &F) -> proc_macro2::TokenStream
where
    F: Fn(&swc_ecma_ast::ReturnStmt) -> proc_macro2::TokenStream,
{
    match stmt {
        Stmt::Return(ret_stmt) => handler(ret_stmt),
        Stmt::Block(block) => {
            let stmts: Vec<_> = block
                .stmts
                .iter()
                .map(|s| convert_stmt_recursive(s, handler))
                .collect();
            quote! {
                {
                    #(#stmts)*
                }
            }
        }
        Stmt::If(if_stmt) => {
            let test = convert_expr(&if_stmt.test);
            let cons = convert_stmt_recursive(&if_stmt.cons, handler);
            let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                quote! { #cons }
            } else {
                quote! { { #cons } }
            };

            let alt = if let Some(alt) = &if_stmt.alt {
                let alt_stmt = convert_stmt_recursive(alt, handler);
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
        // TODO: Add loops if needed. For now, delegate to convert_stmt for others,
        // BUT convert_stmt won't recurse with handler.
        // So we should implement loops here if we expect returns inside loops.
        // Assuming simple cases for now.
        _ => convert_stmt(stmt),
    }
}

fn convert_await_expr(await_expr: &AwaitExpr) -> proc_macro2::TokenStream {
    let arg = convert_expr(&await_expr.arg);
    quote! { #arg.await? }
}

fn convert_call_expr(call: &CallExpr) -> proc_macro2::TokenStream {
    let callee = &call.callee;
    let args = &call.args;

    // Handle JSON.stringify
    if let Callee::Expr(expr) = callee {
        if let Expr::Member(member) = &**expr {
            if let Expr::Ident(obj) = &*member.obj {
                if obj.sym == "JSON" {
                    if let Some(prop) = member.prop.as_ident() {
                        if prop.sym == "stringify" {
                            if let Some(arg) = args.first() {
                                let val = convert_expr_or_spread(arg);
                                return quote! { serde_json::to_string(&#val).unwrap() };
                            }
                        } else if prop.sym == "parse" {
                            if let Some(arg) = args.first() {
                                let val = convert_expr_or_spread(arg);
                                return quote! { serde_json::from_str::<serde_json::Value>(&#val).unwrap() };
                            }
                        }
                    }
                }
            }
        }
    }

    // Handle axios.get<T>(...)
    if let Callee::Expr(expr) = callee {
        if let Expr::Member(member) = &**expr {
            if let Expr::Ident(obj) = &*member.obj {
                if obj.sym == "axios" {
                    if let Some(prop) = member.prop.as_ident() {
                        if prop.sym == "get" {
                            if let Some(arg) = args.first() {
                                let url = convert_expr_or_spread(arg);
                                // Handle generics: axios.get<T>(...)
                                let generic_type = if let Some(type_params) = &call.type_args {
                                    if let Some(param) = type_params.params.first() {
                                        let t = super::type_mapper::map_inner_type(param);
                                        Some(t)
                                    } else {
                                        None
                                    }
                                } else {
                                    None
                                };

                                if let Some(t) = generic_type {
                                    return quote! {
                                        reqwest::Client::new().get(#url).send().await?.json::<#t>()
                                    };
                                } else {
                                    return quote! {
                                        reqwest::Client::new().get(#url).send().await?
                                    };
                                }
                            }
                        }
                    }
                }
            }
        }
    }

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
        return quote! { reqwest::get("") };
    }

    let url = convert_expr_or_spread(&args[0]);
    quote! { reqwest::get(#url) }
}

pub fn convert_arrow_expr(arrow: &swc_ecma_ast::ArrowExpr) -> proc_macro2::TokenStream {
    convert_arrow_expr_with_hint(arrow, None)
}

pub fn convert_arrow_expr_with_hint(
    arrow: &swc_ecma_ast::ArrowExpr,
    type_hint: Option<proc_macro2::TokenStream>,
) -> proc_macro2::TokenStream {
    let params = &arrow.params;
    let body = &arrow.body;

    let param_idents: Vec<_> = params
        .iter()
        .enumerate()
        .map(|(i, p)| {
            if let Pat::Ident(ident) = p {
                let name = format_ident!("{}", to_snake_case(&ident.sym));
                let ts_type = ident
                    .type_ann
                    .as_ref()
                    .map(|ann| super::type_mapper::map_ts_type(Some(ann)));

                if let Some(t) = ts_type {
                    quote! { #name: #t }
                } else if let Some(hint) = &type_hint {
                    // Only apply hint to the first parameter for now (common case for map/filter)
                    if i == 0 {
                        quote! { #name: #hint }
                    } else {
                        quote! { #name }
                    }
                } else {
                    quote! { #name }
                }
            } else {
                quote! { _ }
            }
        })
        .collect();

    let body_code = match &**body {
        swc_ecma_ast::BlockStmtOrExpr::BlockStmt(block) => {
            let stmts: Vec<_> = block.stmts.iter().map(convert_stmt).collect();
            quote! { { #(#stmts)* } }
        }
        swc_ecma_ast::BlockStmtOrExpr::Expr(expr) => {
            let expr_code = convert_expr(expr);
            quote! { #expr_code }
        }
    };

    let is_async = arrow.is_async;
    let _async_kw = if is_async {
        quote! { async }
    } else {
        quote! {}
    };

    quote! { |#(#param_idents),*| #body_code }
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
