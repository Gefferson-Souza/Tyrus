//! Oráculo do Tyrus: tupla de referência pinada (`oracle.lock`) e álgebra de
//! vereditos fechada. Só o oráculo emite vereditos (D21).
#![forbid(unsafe_code)]

pub mod cli;
pub mod cli_run;
pub mod codes;
pub mod driver;
pub mod error;
pub mod lock;
pub mod probe;
pub mod record;
pub mod registry;
pub mod shares;
pub mod tuple;
pub mod verdict;
