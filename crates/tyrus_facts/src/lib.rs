//! Fatos do front-end (ST1): tipos de `facts.json`, `closure.lock` e `partition.lock`,
//! JSON canônico e validação de versão de esquema e de hash. Não depende de nenhum
//! crate legado nem de runtime (direção `facts -> place -> assemble`).
#![forbid(unsafe_code)]

pub mod canonical;
pub mod closure;
pub mod document;
pub mod error;
pub mod facts;
pub mod file_fact;
pub mod partition;
pub mod sha256;
pub mod verify;

pub use canonical::{canonical_bytes, canonicalize_json};
pub use closure::{ClosureFile, ClosureLock, FileMode};
pub use document::{parse_closure_lock, parse_facts, parse_partition_lock, SchemaVersion};
pub use error::FactsError;
pub use facts::{Facts, TypescriptInfo};
pub use file_fact::{FileFact, RelativePath};
pub use partition::{PartitionLock, PartitionUnit, Placement};
pub use sha256::{sha256_hex, Sha256Hex};
