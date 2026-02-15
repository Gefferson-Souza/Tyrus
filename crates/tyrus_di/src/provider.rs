use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum InjectionScope {
    Singleton,
    Request,
    Transient,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provider {
    pub token: String,
    pub implementation: String, // The struct name in Rust
    pub scope: InjectionScope,
    pub dependencies: Vec<String>, // Tokens this provider depends on
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InjectableDefinition {
    pub name: String,
    pub dependencies: Vec<String>,
    pub scope: InjectionScope,
}
