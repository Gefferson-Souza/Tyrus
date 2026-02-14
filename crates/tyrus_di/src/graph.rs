use crate::module::Module;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph as PetDiGraph;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum DiError {
    #[error("Circular dependency detected: {0}")]
    CircularDependency(String),
    #[error("Provider not found: {0}")]
    ProviderNotFound(String),
}

use crate::provider::InjectableDefinition;

pub struct DiGraph {
    pub modules: HashMap<String, Module>,
    pub injectables: HashMap<String, InjectableDefinition>,
    // graph: PetDiGraph<String, ()>, // Removed unused field for now, it's local to resolve()
}

impl DiGraph {
    pub fn new() -> Self {
        Self {
            modules: HashMap::new(),
            injectables: HashMap::new(),
            // graph: PetDiGraph::new(),
        }
    }
}

impl Default for DiGraph {
    fn default() -> Self {
        Self::new()
    }

    pub fn add_module(&mut self, module: Module) {
        self.modules.insert(module.name.clone(), module);
    }

    pub fn add_injectable(&mut self, def: InjectableDefinition) {
        self.injectables.insert(def.name.clone(), def);
    }

    pub fn get_module(&self, name: &str) -> Option<&Module> {
        self.modules.get(name)
    }

    pub fn merge(&mut self, other: DiGraph) {
        for (k, v) in other.modules {
            self.modules.insert(k, v);
        }
        for (k, v) in other.injectables {
            self.injectables.insert(k, v);
        }
    }

    pub fn get_provider_dependencies(&self, token: &str) -> Option<Vec<String>> {
        for module in self.modules.values() {
            for provider in &module.providers {
                if provider.token == token {
                    // Found the provider
                    if !provider.dependencies.is_empty() {
                        return Some(provider.dependencies.clone());
                    } else if let Some(def) = self.injectables.get(token) {
                        return Some(def.dependencies.clone());
                    } else {
                        return Some(vec![]);
                    }
                }
            }
            for controller in &module.controllers {
                if controller == token {
                    if let Some(def) = self.injectables.get(token) {
                        return Some(def.dependencies.clone());
                    } else {
                        return Some(vec![]);
                    }
                }
            }
        }
        None
    }

    pub fn resolve(&self) -> Result<Vec<String>, DiError> {
        // Build the dependency graph
        let mut graph = PetDiGraph::<String, ()>::new();
        let mut node_indices = HashMap::new();

        // 1. Add all providers and controllers as nodes
        for module in self.modules.values() {
            for provider in &module.providers {
                if !node_indices.contains_key(&provider.token) {
                    let idx = graph.add_node(provider.token.clone());
                    node_indices.insert(provider.token.clone(), idx);
                }
            }
            for controller in &module.controllers {
                if !node_indices.contains_key(controller) {
                    let idx = graph.add_node(controller.clone());
                    node_indices.insert(controller.clone(), idx);
                }
            }
        }

        // 2. Add edges (dependencies)
        for module in self.modules.values() {
            // Process Providers
            for provider in &module.providers {
                if let Some(parent_idx) = node_indices.get(&provider.token) {
                    // Try to find dependencies from InjectableDefinition
                    let deps = if !provider.dependencies.is_empty() {
                        &provider.dependencies
                    } else if let Some(def) = self.injectables.get(&provider.token) {
                        &def.dependencies
                    } else {
                        &provider.dependencies // empty
                    };

                    for dep_token in deps {
                        if let Some(dep_idx) = node_indices.get(dep_token) {
                            graph.add_edge(*dep_idx, *parent_idx, ());
                        }
                    }
                }
            }

            // Process Controllers
            for controller in &module.controllers {
                if let Some(parent_idx) = node_indices.get(controller) {
                    // Controllers are Injectables (definitions exist)
                    if let Some(def) = self.injectables.get(controller) {
                        for dep_token in &def.dependencies {
                            if let Some(dep_idx) = node_indices.get(dep_token) {
                                graph.add_edge(*dep_idx, *parent_idx, ());
                            }
                        }
                    }
                }
            }
        }

        // 3. Topological Sort
        match toposort(&graph, None) {
            Ok(sorted_indices) => {
                let sorted_tokens: Vec<String> = sorted_indices
                    .into_iter()
                    .map(|idx| graph[idx].clone())
                    .collect();
                Ok(sorted_tokens)
            }
            Err(cycle) => {
                let node_weight = &graph[cycle.node_id()];
                Err(DiError::CircularDependency(node_weight.clone()))
            }
        }
    }
}
