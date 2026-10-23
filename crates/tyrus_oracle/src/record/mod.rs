//! `tyrus-oracle record`: regrava os goldens do corpus a partir do emit por arquivo do `tsc`
//! fixado, executado no Node de referência, e o diff contra a semente.
pub mod capture;
pub mod checker;
pub mod classify;
pub mod emit;
pub mod error;
pub mod exec;
pub mod model;
pub mod options;
pub mod recorder;
pub mod runner;
pub mod sources;
pub mod store;

pub use runner::run_record;
