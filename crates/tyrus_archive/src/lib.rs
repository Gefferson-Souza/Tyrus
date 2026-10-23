//! Arquivo do fecho: escritor e leitor determinísticos (runtime, sem crate do compilador).
#![forbid(unsafe_code)]

pub mod cli;
mod digest;
pub mod error;
mod format;
pub mod inspect;
mod reader;
mod writer;

pub use error::ArchiveError;
pub use format::{COMPRESSION_LEVEL, FORMAT_VERSION};
pub use reader::{ArchiveReader, EntryInfo};
pub use writer::ArchiveWriter;
