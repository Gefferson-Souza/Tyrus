use petgraph::algo::toposort;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::HashMap;
use swc_ecma_ast::*;
use swc_ecma_visit::{Visit, VisitWith};

pub struct DependencyGraph {
    graph: DiGraph<String, ()>,
    node_map: HashMap<String, NodeIndex>,
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self {
            graph: DiGraph::new(),
            node_map: HashMap::new(),
        }
    }
}

impl Default for DependencyGraph {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyGraph {
    pub fn add_node(&mut self, name: String) -> NodeIndex {
        if let Some(&idx) = self.node_map.get(&name) {
            idx
        } else {
            let idx = self.graph.add_node(name.clone());
            self.node_map.insert(name, idx);
            idx
        }
    }

    pub fn add_dependency(&mut self, from: &str, to: &str) {
        let from_idx = self.add_node(from.to_string());
        let to_idx = self.add_node(to.to_string());
        // Edge from -> to means "from depends on to"
        // But for toposort, if A -> B, A comes before B.
        // If we want initialization order, we want B before A.
        // So we want the reverse.
        // Let's define Edge A -> B as "A depends on B".
        // Toposort will give [A, B] (if acyclic).
        // Wait, petgraph toposort: "If the graph is a DAG, returns a vector of nodes in topological order."
        // Topological order: for every edge u -> v, u comes before v.
        // If A depends on B, we want B initialized first.
        // So we want B before A.
        // If we add edge A -> B ("A depends on B"), toposort gives A then B.
        // So we need to reverse the result of toposort to get [B, A].
        // OR we add edge B -> A ("B must be created before A").
        // Let's stick to "A depends on B" (A -> B) and reverse the list.
        self.graph.update_edge(from_idx, to_idx, ());
    }

    pub fn get_initialization_order(&self) -> Result<Vec<String>, String> {
        match toposort(&self.graph, None) {
            Ok(nodes) => {
                let mut order: Vec<String> =
                    nodes.iter().map(|&idx| self.graph[idx].clone()).collect();
                order.reverse(); // Reverse to get initialization order (dependencies first)
                Ok(order)
            }
            Err(_) => Err("Cycle detected in dependency graph".to_string()),
        }
    }
    pub fn get_dependencies(&self, node_name: &str) -> Option<Vec<String>> {
        if let Some(&idx) = self.node_map.get(node_name) {
            let deps: Vec<String> = self
                .graph
                .neighbors(idx)
                .map(|neighbor_idx| self.graph[neighbor_idx].clone())
                .collect();
            Some(deps)
        } else {
            None
        }
    }
}

struct DependencyVisitor<'a> {
    graph: &'a mut DependencyGraph,
    current_class: Option<String>,
}

impl<'a> Visit for DependencyVisitor<'a> {
    fn visit_class_decl(&mut self, n: &ClassDecl) {
        let class_name = n.ident.sym.to_string();
        self.current_class = Some(class_name.clone());
        self.graph.add_node(class_name);
        n.visit_children_with(self);
        self.current_class = None;
    }

    fn visit_constructor(&mut self, n: &Constructor) {
        if let Some(current_class) = &self.current_class {
            for param in &n.params {
                match param {
                    ParamOrTsParamProp::TsParamProp(prop) => {
                        // constructor(private service: Service)
                        if let TsParamPropParam::Ident(ident) = &prop.param {
                            if let Some(type_ann) = &ident.type_ann {
                                if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                    if let Some(type_name) = type_ref.type_name.as_ident() {
                                        let dep_name = type_name.sym.to_string();
                                        self.graph.add_dependency(current_class, &dep_name);
                                    }
                                }
                            }
                        }
                    }
                    ParamOrTsParamProp::Param(param) => {
                        // Also handle constructor(service: Service)
                        if let Pat::Ident(ident) = &param.pat {
                            if let Some(type_ann) = &ident.type_ann {
                                if let Some(type_ref) = type_ann.type_ann.as_ts_type_ref() {
                                    if let Some(type_name) = type_ref.type_name.as_ident() {
                                        let dep_name = type_name.sym.to_string();
                                        self.graph.add_dependency(current_class, &dep_name);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn build_graph(programs: &[Program]) -> DependencyGraph {
    let mut graph = DependencyGraph::new();
    for program in programs {
        let mut visitor = DependencyVisitor {
            graph: &mut graph,
            current_class: None,
        };
        program.visit_with(&mut visitor);
    }
    graph
}
