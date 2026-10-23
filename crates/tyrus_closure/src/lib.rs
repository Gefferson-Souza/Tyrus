//! Fecho de produção: árvore montada do lockfile (`npm ci --omit=dev`) mais o emit do
//! app, com `closure.lock` (tipos de `tyrus_facts`) e validação contra o rastro de carga
//! de módulos do oráculo. O rastro só valida; nunca participa do build do produto.
#![forbid(unsafe_code)]

pub mod build;
pub mod dev_check;
pub mod error;
pub mod trace;
pub mod validate;
pub mod walk;

pub use build::{build_closure, BuildReport, BuildRequest, APP_DIR};
pub use error::ClosureError;
pub use trace::{parse_trace, TraceEntry, TraceKind};
pub use validate::{validate_trace, ValidationReport};
pub use walk::{collect_inventory, lock_from_tree, Inventory};
