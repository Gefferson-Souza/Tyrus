#![forbid(unsafe_code)]
#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::string_slice,
    dead_code,
    unused_imports
)]

mod cli;
mod cli_closure;
mod cli_gate;
mod compilation;
mod equivalence;
mod helpers;
mod snapshot;
mod trybuild_tests;
mod unit;
