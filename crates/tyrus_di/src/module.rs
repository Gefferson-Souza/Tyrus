use crate::provider::Provider;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Module {
    pub name: String,
    pub imports: Vec<String>, // Defines module dependencies
    pub providers: Vec<Provider>,
    pub controllers: Vec<String>, // Controllers belonging to this module
    pub exports: Vec<String>,     // Tokens exported by this module
}
