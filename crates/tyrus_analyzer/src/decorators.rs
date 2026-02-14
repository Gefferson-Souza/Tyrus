use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};
use tyrus_di::graph::DiGraph;
use tyrus_di::module::Module;
use tyrus_di::provider::{InjectionScope, Provider};

pub struct DecoratorVisitor {
    pub graph: DiGraph,
    pub current_module: Option<Module>,
}

impl DecoratorVisitor {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            current_module: None,
        }
    }

    fn extract_decorator_args(&self, decorator: &Decorator) -> Option<ObjectLit> {
        if let Expr::Call(call_expr) = &*decorator.expr {
            if let Some(arg) = call_expr.args.first() {
                if let Expr::Object(obj_lit) = &*arg.expr {
                    return Some(obj_lit.clone());
                }
            }
        }
        None
    }
}

impl Visit for DecoratorVisitor {
    fn visit_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        let mut is_injectable = false;

        for decorator in &n.class.decorators {
            if let Expr::Call(call_expr) = &*decorator.expr {
                if let Callee::Expr(expr) = &call_expr.callee {
                    if let Expr::Ident(ident) = &**expr {
                        if ident.sym == *"Module" {
                            // Extract Module Metadata
                            let mut module = Module {
                                name: class_name.clone(),
                                imports: vec![],
                                providers: vec![],
                                controllers: vec![],
                                exports: vec![],
                            };

                            if let Some(obj) = self.extract_decorator_args(decorator) {
                                for prop in obj.props {
                                    if let PropOrSpread::Prop(prop) = prop {
                                        if let Prop::KeyValue(kv) = *prop {
                                            if let PropName::Ident(key) = kv.key {
                                                if let Expr::Array(arr) = *kv.value {
                                                    let values: Vec<String> = arr
                                                        .elems
                                                        .iter()
                                                        .filter_map(|e| {
                                                            if let Some(ExprOrSpread {
                                                                expr, ..
                                                            }) = e
                                                            {
                                                                if let Expr::Ident(i) = &**expr {
                                                                    return Some(i.sym.to_string());
                                                                }
                                                            }
                                                            None
                                                        })
                                                        .collect();

                                                    match key.sym.as_ref() {
                                                        "imports" => module.imports = values,
                                                        "providers" => {
                                                            module.providers = values
                                                                .into_iter()
                                                                .map(|v| Provider {
                                                                    token: v.clone(),
                                                                    implementation: v,
                                                                    scope:
                                                                        InjectionScope::Singleton,
                                                                    dependencies: vec![], // Will be filled later
                                                                })
                                                                .collect();
                                                        }
                                                        "controllers" => {
                                                            module.controllers = values
                                                        }
                                                        "exports" => module.exports = values,
                                                        _ => {}
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                            self.graph.add_module(module);
                        } else if ident.sym == *"Injectable" || ident.sym == *"Controller" {
                            is_injectable = true;
                        }
                    }
                }
            }
        }

        if is_injectable {
            // Extract dependencies from constructor
            let mut deps = vec![];
            for member in &n.class.body {
                if let ClassMember::Constructor(cons) = member {
                    for param in &cons.params {
                        match param {
                            ParamOrTsParamProp::TsParamProp(prop) => {
                                if let TsParamPropParam::Ident(ident) = &prop.param {
                                    if let Some(type_ann) = &ident.type_ann {
                                        if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                            if let Some(type_name) = type_ref.type_name.as_ident() {
                                                deps.push(type_name.sym.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                            ParamOrTsParamProp::Param(param) => {
                                if let Pat::Ident(ident) = &param.pat {
                                    if let Some(type_ann) = &ident.type_ann {
                                        if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                            if let Some(type_name) = type_ref.type_name.as_ident() {
                                                deps.push(type_name.sym.to_string());
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            // Add to graph as definition
            self.graph
                .add_injectable(tyrus_di::provider::InjectableDefinition {
                    name: class_name.clone(),
                    dependencies: deps,
                    scope: InjectionScope::Singleton,
                });
        }
        n.visit_children_with(self);
    }
}
