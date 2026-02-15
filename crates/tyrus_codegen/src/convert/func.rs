use quote::{format_ident, quote};
use swc_ecma_ast::{
    ArrowExpr, BinExpr, BinaryOp, BlockStmtOrExpr, CallExpr, Callee, Decl, Expr, ExprOrSpread,
    FnDecl, Lit, MemberExpr, Pat, Stmt, UpdateExpr, UpdateOp,
};

use super::type_mapper::{map_ts_type, unwrap_promise_type};

// Helper function to snake case (keep standalone)
pub fn to_snake_case(str: &str) -> String {
    let mut s = String::with_capacity(str.len());
    let mut was_upper = false;
    for (i, c) in str.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 && !was_upper {
                s.push('_');
            }
            s.push(c.to_ascii_lowercase());
            was_upper = true;
        } else {
            s.push(c);
            was_upper = false;
        }
    }
    s
}

// Helper to pascal case (keep standalone)
pub fn to_pascal_case(str: &str) -> String {
    let mut s = String::with_capacity(str.len());
    let mut next_upper = true;
    for c in str.chars() {
        if c == '_' || c == '-' {
            next_upper = true;
        } else if next_upper {
            s.push(c.to_ascii_uppercase());
            next_upper = false;
        } else {
            s.push(c);
        }
    }
    s
}

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
                for stmt in &block_stmt.stmts {
                    body_stmts.push(self.convert_stmt_recursive(stmt, &mut |ret_stmt| {
                         if let Some(arg) = &ret_stmt.arg {
                            let expr = self.convert_expr(arg);
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
            } else {
                for stmt in &block_stmt.stmts {
                    body_stmts.push(self.convert_stmt_recursive(stmt, &mut |ret_stmt| {
                        if let Some(arg) = &ret_stmt.arg {
                            let expr = self.convert_expr(arg);
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

    // New helper method for recursive stmt conversion with callback
    pub fn convert_stmt_recursive<F>(
        &self,
        stmt: &Stmt,
        return_handler: &mut F,
    ) -> proc_macro2::TokenStream
    where
        F: FnMut(&swc_ecma_ast::ReturnStmt) -> proc_macro2::TokenStream,
    {
        match stmt {
            Stmt::Return(ret_stmt) => return_handler(ret_stmt),
            Stmt::Block(block) => {
                let stmts: Vec<_> = block
                    .stmts
                    .iter()
                    .map(|s| self.convert_stmt_recursive(s, return_handler))
                    .collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt_recursive(&if_stmt.cons, return_handler);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt_recursive(alt, return_handler);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            _ => self.convert_stmt(stmt),
        }
    }

    pub fn convert_stmt(&self, stmt: &Stmt) -> proc_macro2::TokenStream {
        match stmt {
            Stmt::Return(ret_stmt) => {
                if let Some(arg) = &ret_stmt.arg {
                    let expr = self.convert_expr(arg);
                    quote! { return #expr; }
                } else {
                    quote! { return; }
                }
            }
            Stmt::Expr(expr_stmt) => {
                let expr = self.convert_expr(&expr_stmt.expr);
                quote! { #expr; }
            }
            Stmt::Decl(Decl::Var(var_decl)) => {
                let mut declarations = Vec::new();
                for decl in &var_decl.decls {
                    let init_expr_opt = decl.init.as_ref().map(|init| self.convert_expr(init));

                    match &decl.name {
                        Pat::Ident(ident) => {
                            let var_name = to_snake_case(&ident.id.sym);
                            let var_ident = format_ident!("{}", var_name);

                            if let Some(init_expr) = init_expr_opt {
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
                                declarations.push(quote! {
                                    let mut #var_ident;
                                });
                            }
                        }
                        Pat::Object(_) | Pat::Array(_) => {
                            declarations.push(quote! { /* destructing not fully ported to method struct yet, todo */ });
                        }
                        _ => {
                            declarations.push(quote! { /* unsupported pattern */ });
                        }
                    }
                }
                quote! { #(#declarations)* }
            }
            Stmt::Block(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            Stmt::If(if_stmt) => {
                let test = self.convert_expr(&if_stmt.test);
                let cons = self.convert_stmt(&if_stmt.cons);
                let cons_block = if matches!(*if_stmt.cons, Stmt::Block(_)) {
                    quote! { #cons }
                } else {
                    quote! { { #cons } }
                };

                let alt = if let Some(alt) = &if_stmt.alt {
                    let alt_stmt = self.convert_stmt(alt);
                    let alt_block = if matches!(&**alt, Stmt::Block(_) | Stmt::If(_)) {
                        quote! { #alt_stmt }
                    } else {
                        quote! { { #alt_stmt } }
                    };
                    quote! { else #alt_block }
                } else {
                    quote! {}
                };

                quote! { if #test #cons_block #alt }
            }
            Stmt::While(while_stmt) => {
                let test = self.convert_expr(&while_stmt.test);
                let body = self.convert_stmt(&while_stmt.body);
                let body_block = if matches!(*while_stmt.body, Stmt::Block(_)) {
                    quote! { #body }
                } else {
                    quote! { { #body } }
                };
                quote! {
                    while #test #body_block
                }
            }
            Stmt::ForOf(for_of) => {
                let body = self.convert_stmt(&for_of.body);
                let right = self.convert_expr(&for_of.right);
                let body_block = if matches!(*for_of.body, Stmt::Block(_)) {
                    quote! { #body }
                } else {
                    quote! { { #body } }
                };

                // Extract the loop variable name from the declaration
                let var_ident = match &for_of.left {
                    swc_ecma_ast::ForHead::VarDecl(var_decl) => {
                        if let Some(decl) = var_decl.decls.first() {
                            if let Pat::Ident(ident) = &decl.name {
                                let name = to_snake_case(&ident.id.sym);
                                format_ident!("{}", name)
                            } else {
                                format_ident!("_item")
                            }
                        } else {
                            format_ident!("_item")
                        }
                    }
                    swc_ecma_ast::ForHead::Pat(pat) => {
                        if let Pat::Ident(ident) = pat.as_ref() {
                            let name = to_snake_case(&ident.id.sym);
                            format_ident!("{}", name)
                        } else {
                            format_ident!("_item")
                        }
                    }
                    _ => format_ident!("_item"),
                };

                quote! {
                    for #var_ident in #right #body_block
                }
            }
            Stmt::Throw(throw_stmt) => {
                let arg = self.convert_expr(&throw_stmt.arg);
                quote! {
                    return Err(#arg.into());
                }
            }
            _ => quote! { /* other stmts todo */ },
        }
    }

    pub fn convert_expr(&self, expr: &Expr) -> proc_macro2::TokenStream {
        match expr {
            Expr::Bin(bin) => self.convert_bin_expr(bin),
            Expr::This(_) => quote! { self },
            Expr::Ident(ident) => {
                let name = ident.sym.as_str();
                if name == "undefined" {
                    return quote! { None };
                }
                if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    let ident_token = format_ident!("{}", name);
                    quote! { #ident_token }
                } else {
                    let ident_name = to_snake_case(name);
                    let ident_token = format_ident!("{}", ident_name);
                    quote! { #ident_token }
                }
            }
            Expr::Lit(lit) => match lit {
                Lit::Num(num) => {
                    let value = num.value;
                    quote! { #value }
                }
                Lit::Str(s) => {
                    let v = s.value.as_str().unwrap_or("");
                    quote! { String::from(#v) }
                }
                Lit::Bool(b) => {
                    let v = b.value;
                    quote! { #v }
                }
                _ => quote! { todo!("unsupported literal") },
            },
            Expr::Member(member) => self.convert_member_expr(member),
            Expr::Call(call) => self.convert_call_expr(call),
            Expr::Object(obj) => self.convert_object_lit(obj),
            Expr::Assign(assign) => self.convert_assign_expr(assign),
            Expr::Update(update) => self.convert_update_expr(update),
            Expr::Await(await_expr) => {
                let arg = self.convert_expr(&await_expr.arg);
                quote! { #arg.await? }
            }
            Expr::New(new_expr) => {
                let callee = self.convert_expr(&new_expr.callee);
                let args = if let Some(args) = &new_expr.args {
                    args.iter()
                        .map(|a| self.convert_expr_or_spread(a))
                        .collect()
                } else {
                    Vec::new()
                };
                quote! { #callee::new(#(#args),*) }
            }
            Expr::Paren(paren) => self.convert_expr(&paren.expr),
            Expr::Arrow(arrow) => self.convert_arrow_expr(arrow),

            Expr::Array(arr) => self.convert_array_lit(arr),
            _ => quote! { todo!() },
        }
    }

    fn convert_array_lit(&self, arr: &swc_ecma_ast::ArrayLit) -> proc_macro2::TokenStream {
        let elems: Vec<_> = arr
            .elems
            .iter()
            .flatten()
            .map(|elem| self.convert_expr_or_spread(elem))
            .collect();
        quote! { vec![#(#elems),*] }
    }

    pub fn convert_expr_or_spread(&self, arg: &ExprOrSpread) -> proc_macro2::TokenStream {
        self.convert_expr(&arg.expr)
    }

    fn convert_member_expr(&self, member: &MemberExpr) -> proc_macro2::TokenStream {
        if member.obj.is_this() {
            if let Some(prop_ident) = member.prop.as_ident() {
                let prop_name = to_snake_case(prop_ident.sym.as_ref());
                let field_ident = format_ident!("{}", prop_name);

                // CHECK STATE FIELDS
                if self
                    .current_class_state_fields
                    .contains_key(prop_ident.sym.as_ref())
                {
                    let type_str = self
                        .current_class_state_fields
                        .get(prop_ident.sym.as_ref())
                        .unwrap();
                    let needs_deref = matches!(
                        type_str.as_str(),
                        "f64" | "bool" | "i32" | "usize" | "u64" | "i64"
                    );

                    if needs_deref {
                        return quote! { *self.#field_ident.lock().unwrap() };
                    } else if type_str == "String" {
                        // String is not Copy, so we must clone it to return/use as value from MutexGuard
                        return quote! { self.#field_ident.lock().unwrap().clone() };
                    } else {
                        // For non-primitives (like Vec), return the Guard (or ref?)
                        // If we return *guard, we move out of mutex? No, implementation of Deref.
                        // But accessing `self.users.lock().unwrap()` returns `MutexGuard`.
                        // If we want to call methods on it (`.push()`), we need the Guard.
                        // If we want to pass it to a function expecting `Vec`, we might need `&*` or `.clone()`.
                        // But here we return TokenStream.
                        return quote! { self.#field_ident.lock().unwrap() };
                    }
                } else {
                    return quote! { self.#field_ident.clone() };
                }
            }
        }

        let obj = self.convert_expr(&member.obj);
        match &member.prop {
            swc_ecma_ast::MemberProp::Ident(ident) => {
                let name = ident.sym.as_ref();
                // If it starts with uppercase, preserve it (Enum variant, etc.)
                let prop_name = if name.chars().next().is_some_and(|c| c.is_uppercase()) {
                    name.to_string()
                } else {
                    to_snake_case(name)
                };
                let prop = format_ident!("{}", prop_name);
                quote! { #obj.#prop }
            }
            swc_ecma_ast::MemberProp::Computed(computed) => {
                let expr = self.convert_expr(&computed.expr);
                quote! { #obj[#expr] }
            }
            _ => quote! { todo!() },
        }
    }

    fn convert_bin_expr(&self, bin: &BinExpr) -> proc_macro2::TokenStream {
        let left = self.convert_expr(&bin.left);
        let right = self.convert_expr(&bin.right);
        match bin.op {
            BinaryOp::EqEq | BinaryOp::EqEqEq => quote! { #left == #right },
            BinaryOp::NotEq | BinaryOp::NotEqEq => quote! { #left != #right },
            BinaryOp::Add => quote! { #left + #right },
            BinaryOp::Sub => quote! { #left - #right },
            BinaryOp::Mul => quote! { #left * #right },
            BinaryOp::Div => quote! { #left / #right },
            BinaryOp::Mod => quote! { #left % #right },
            BinaryOp::Lt => quote! { #left < #right },
            BinaryOp::LtEq => quote! { #left <= #right },
            BinaryOp::Gt => quote! { #left > #right },
            BinaryOp::GtEq => quote! { #left >= #right },
            BinaryOp::LogicalOr => quote! { #left || #right },
            BinaryOp::LogicalAnd => quote! { #left && #right },
            _ => {
                let op_str = format!("{:?}", bin.op);
                quote! { todo!("Unsupported binary op: {}", #op_str) }
            }
        }
    }

    fn convert_call_expr(&self, call: &CallExpr) -> proc_macro2::TokenStream {
        // Try stdlib handlers first - PASS SELF
        if let Some(stdlib_code) =
            crate::stdlib::try_handle_stdlib_call(self, &call.callee, &call.args)
        {
            return stdlib_code;
        }

        // Check for axios calls (axios.get, axios.post, etc.)
        if let Callee::Expr(expr) = &call.callee {
            if let Expr::Member(member) = &**expr {
                // Try stdlib method call - PASS SELF
                if let Some(method_ident) = member.prop.as_ident() {
                    let method_name = method_ident.sym.as_ref();
                    if let Some(stdlib_code) = crate::stdlib::try_handle_method_call(
                        self,
                        &member.obj,
                        method_name,
                        &call.args,
                    ) {
                        return stdlib_code;
                    }
                }
            }
        }

        let callee = match &call.callee {
            Callee::Expr(expr) => self.convert_expr(expr),
            _ => quote! { todo!("complex callee") },
        };
        let args: Vec<_> = call
            .args
            .iter()
            .map(|a| self.convert_expr(&a.expr))
            .collect();
        quote! { #callee(#(#args),*) }
    }

    fn convert_object_lit(&self, obj: &swc_ecma_ast::ObjectLit) -> proc_macro2::TokenStream {
        let mut fields = Vec::new();
        for prop in &obj.props {
            if let swc_ecma_ast::PropOrSpread::Prop(p) = prop {
                if let swc_ecma_ast::Prop::KeyValue(kv) = &**p {
                    if let swc_ecma_ast::PropName::Ident(ident) = &kv.key {
                        let key = ident.sym.as_ref();
                        let val = self.convert_expr(&kv.value);
                        fields.push(quote! { #key: #val });
                    }
                }
            }
        }
        quote! { serde_json::json!({ #(#fields),* }) }
    }

    fn convert_assign_expr(&self, assign: &swc_ecma_ast::AssignExpr) -> proc_macro2::TokenStream {
        let right = self.convert_expr(&assign.right);
        // Handle Left Side
        let left = match &assign.left {
            swc_ecma_ast::AssignTarget::Simple(simple) => {
                match simple {
                    swc_ecma_ast::SimpleAssignTarget::Member(member) => {
                        if member.obj.is_this() {
                            if let Some(prop_ident) = member.prop.as_ident() {
                                let prop_name = to_snake_case(prop_ident.sym.as_ref());
                                let field = format_ident!("{}", prop_name);

                                // CHECK STATE FIELDS
                                if let Some(_type_str) =
                                    self.current_class_state_fields.get(prop_ident.sym.as_ref())
                                {
                                    // Primitive check?
                                    // f64, bool, i32, usize etc. need dereference when used as value, but wait...
                                    // In Assign Expr:
                                    // LHS = RHS
                                    // If LHS is Mutex, we need `*lock() = value`.
                                    // This applies to ALL types inside Mutex if we are replacing the value?
                                    // No, only if we can assign to deref.
                                    // `Vec` inside Mutex: `*lock() = new_vec`. Yes.

                                    // So actually, ALL assignments to Mutex-wrapped field need `*`?
                                    // `*self.users.lock().unwrap() = ...`
                                    // Yes, otherwise we are trying to assign to `MutexGuard` temporary?
                                    // Assigning to `*guard` assigns to the inner value.

                                    quote! { *self.#field.lock().unwrap() }
                                } else {
                                    quote! { self.#field }
                                }
                            } else {
                                quote! { todo!() }
                            }
                        } else {
                            quote! { todo!() }
                        }
                    }
                    swc_ecma_ast::SimpleAssignTarget::Ident(ident) => {
                        let name = format_ident!("{}", to_snake_case(ident.sym.as_ref()));
                        quote! { #name }
                    }
                    _ => quote! { todo!() },
                }
            }
            _ => quote! { todo!() },
        };

        match assign.op {
            swc_ecma_ast::AssignOp::Assign => quote! { #left = #right },
            swc_ecma_ast::AssignOp::AddAssign => quote! { #left += #right },
            _ => quote! { #left op= #right },
        }
    }

    fn convert_update_expr(&self, update: &UpdateExpr) -> proc_macro2::TokenStream {
        let arg = self.convert_expr(&update.arg);
        // If arg is a mutex lock result, modifications work assuming deref mut
        match update.op {
            UpdateOp::PlusPlus => quote! { #arg += 1.0 },
            UpdateOp::MinusMinus => quote! { #arg -= 1.0 },
        }
    }

    fn convert_arrow_expr(&self, arrow: &ArrowExpr) -> proc_macro2::TokenStream {
        let params: Vec<_> = arrow
            .params
            .iter()
            .map(|pat| {
                if let Pat::Ident(ident) = pat {
                    let name = format_ident!("{}", to_snake_case(&ident.id.sym));
                    quote! { #name }
                } else {
                    quote! { _ }
                }
            })
            .collect();

        let body = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                let stmts: Vec<_> = block.stmts.iter().map(|s| self.convert_stmt(s)).collect();
                quote! { { #(#stmts)* } }
            }
            BlockStmtOrExpr::Expr(expr) => self.convert_expr(expr),
        };

        // Note: We might need type annotations for params in complex cases,
        // but for simple .find(|x| ...), inference usually works.
        quote! {
            |#(#params),*| #body
        }
    }
}
