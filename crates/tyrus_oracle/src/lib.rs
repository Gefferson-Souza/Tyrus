//! Oráculo do Tyrus: tupla de referência pinada (`oracle.lock`) e álgebra de
//! vereditos fechada. Só o oráculo emite vereditos (D21).
#![forbid(unsafe_code)]

pub mod codes;
pub mod error;
pub mod lock;
pub mod probe;
pub mod registry;
pub mod shares;
pub mod tuple;
pub mod verdict;
