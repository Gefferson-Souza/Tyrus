use swc_ecma_ast::{ModuleDecl, ModuleItem};
use swc_ecma_visit::VisitWith;

use super::interface::RustGenerator;

impl RustGenerator {
    pub fn process_module_item(&mut self, n: &ModuleItem) {
        match n {
            ModuleItem::ModuleDecl(decl) => match decl {
                ModuleDecl::ExportDecl(export_decl) => {
                    self.is_exporting = true;
                    export_decl.decl.visit_with(self);
                    self.is_exporting = false;
                }
                ModuleDecl::ExportDefaultDecl(default_decl) => {
                    self.is_exporting = true;
                    match &default_decl.decl {
                        swc_ecma_ast::DefaultDecl::Class(class_expr) => {
                            // Convert ClassExpr to ClassDecl for visitor
                            // We need to construct a ClassDecl manually or just visit the ClassExpr
                            // But our visitor expects ClassDecl.
                            // Actually, ClassExpr has an ident (optional). If it's default export, it might be anonymous.
                            // If anonymous, we can't easily make it a named struct in Rust without a name.
                            // For now, assume it has a name or we skip.
                            if let Some(ident) = &class_expr.ident {
                                let decl = swc_ecma_ast::ClassDecl {
                                    ident: ident.clone(),
                                    declare: false,
                                    class: class_expr.class.clone(),
                                };
                                self.process_class_decl(&decl);
                            }
                        }
                        swc_ecma_ast::DefaultDecl::Fn(fn_expr) => {
                            if let Some(ident) = &fn_expr.ident {
                                let decl = swc_ecma_ast::FnDecl {
                                    ident: ident.clone(),
                                    declare: false,
                                    function: fn_expr.function.clone(),
                                };
                                self.process_fn_decl(&decl);
                            }
                        }
                        _ => {}
                    }
                    self.is_exporting = false;
                }
                ModuleDecl::Import(import_decl) => {
                    self.process_import_decl(import_decl);
                }
                _ => {
                    // Other module declarations
                }
            },
            ModuleItem::Stmt(stmt) => {
                stmt.visit_with(self);
            }
        }
    }

    fn process_import_decl(&mut self, n: &swc_ecma_ast::ImportDecl) {
        let src_atom = &n.src.value;
        let mut src = src_atom.as_str().unwrap_or("").to_string();

        // Strip /index suffix if present
        if src.ends_with("/index") {
            src = src.trim_end_matches("/index").to_string();
        }

        // Path resolution
        let module_path = if src.starts_with("./") {
            let path_str = src.trim_start_matches("./");
            if self.is_index {
                // In mod.rs (index.ts), ./foo refers to a sibling module which is a child of the current directory
                // So `use self::foo` or just `foo` works if we are inside the module definition.
                // But usually we want `use crate::path::to::foo`.
                // If we use relative paths:
                // `mod.rs` defines the module `utils`.
                // `foo` is `utils::foo`.
                // Inside `utils`, `foo` is accessible as `self::foo`.
                format!("self::{}", path_str.replace("/", "::"))
            } else {
                // In normal file (e.g. math.rs), ./foo refers to a sibling.
                // `math` and `foo` are siblings in `utils`.
                // To access `foo` from `math`, we use `super::foo`.
                format!("super::{}", path_str.replace("/", "::"))
            }
        } else if src.starts_with("../") {
            let path_str = src.trim_start_matches("../");
            if self.is_index {
                // ../foo from mod.rs -> super::foo
                format!("super::{}", path_str.replace("/", "::"))
            } else {
                // ../foo from normal file -> super::super::foo
                format!("super::super::{}", path_str.replace("/", "::"))
            }
        } else {
            // External crate or absolute path
            src.to_string()
        };

        for specifier in &n.specifiers {
            match specifier {
                swc_ecma_ast::ImportSpecifier::Named(named) => {
                    let imported_name = if let Some(imported) = &named.imported {
                        match imported {
                            swc_ecma_ast::ModuleExportName::Ident(ident) => ident.sym.to_string(),
                            swc_ecma_ast::ModuleExportName::Str(s) => {
                                s.value.as_str().unwrap_or("").to_string()
                            }
                        }
                    } else {
                        named.local.sym.to_string()
                    };

                    // Apply casing logic to imported name
                    // If starts with Uppercase, keep it (Class/Type)
                    // If lowercase, convert to snake_case (Function/Var)
                    let imported_rust_name = if imported_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_uppercase())
                    {
                        imported_name.clone()
                    } else {
                        super::func::to_snake_case(&imported_name)
                    };

                    let local_name = named.local.sym.to_string();
                    let local_rust_name = if local_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_uppercase())
                    {
                        local_name.clone()
                    } else {
                        super::func::to_snake_case(&local_name)
                    };

                    let use_stmt = if imported_rust_name == local_rust_name {
                        format!("use {}::{};", module_path, local_rust_name)
                    } else {
                        format!(
                            "use {}::{} as {};",
                            module_path, imported_rust_name, local_rust_name
                        )
                    };

                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
                swc_ecma_ast::ImportSpecifier::Default(default) => {
                    let local_name = default.local.sym.to_string();
                    let local_rust_name = if local_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_uppercase())
                    {
                        local_name.clone()
                    } else {
                        super::func::to_snake_case(&local_name)
                    };

                    // Default import usually implies importing the struct/fn with the same name as the module or file
                    // But here we just import the name from the module path.
                    // If module_path is `super::models`, and we import default as `User`,
                    // we assume `super::models::User` exists.
                    // But if `models.ts` has `export default class User`, it generates `pub struct User`.
                    // So `use super::models::User` is correct.

                    let use_stmt = format!("use {}::{};", module_path, local_rust_name);
                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
                swc_ecma_ast::ImportSpecifier::Namespace(ns) => {
                    let local_name = ns.local.sym.to_string();
                    let local_rust_name = if local_name
                        .chars()
                        .next()
                        .map_or(false, |c| c.is_uppercase())
                    {
                        local_name.clone()
                    } else {
                        super::func::to_snake_case(&local_name)
                    };
                    let use_stmt = format!("use {} as {};", module_path, local_rust_name);
                    self.code.push_str(&use_stmt);
                    self.code.push('\n');
                }
            }
        }
    }
}
