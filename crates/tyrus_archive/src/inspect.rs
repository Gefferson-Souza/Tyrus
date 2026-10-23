//! Relatório estável de `tyrus-archive inspect --json` (contrato: `schemas/closure-archive.schema.json`).
use serde::Serialize;

use crate::format::COMPRESSION_LEVEL;
use crate::reader::{ArchiveReader, EntryInfo};

#[derive(Debug, Serialize)]
pub struct Compression {
    pub codec: &'static str,
    pub level: i32,
}

#[derive(Debug, Serialize)]
pub struct InspectReport {
    pub format_version: u32,
    pub entry_count: usize,
    pub set_sha256: String,
    pub compression: Compression,
    pub total_size: u64,
    pub total_compressed_size: u64,
    pub entries: Vec<InspectEntry>,
}

#[derive(Debug, Serialize)]
pub struct InspectEntry {
    pub path: String,
    pub mode: u32,
    pub size: u64,
    pub compressed_size: u64,
    pub sha256: String,
}

impl From<&EntryInfo> for InspectEntry {
    fn from(info: &EntryInfo) -> Self {
        Self {
            path: info.path.clone(),
            mode: info.mode,
            size: info.size,
            compressed_size: info.compressed_size,
            sha256: info.sha256.clone(),
        }
    }
}

#[must_use]
pub fn report(reader: &ArchiveReader) -> InspectReport {
    let entries: Vec<InspectEntry> = reader.entries().iter().map(InspectEntry::from).collect();
    InspectReport {
        format_version: reader.format_version(),
        entry_count: entries.len(),
        set_sha256: reader.set_sha256(),
        compression: Compression {
            codec: "zstd",
            level: COMPRESSION_LEVEL,
        },
        total_size: entries.iter().map(|e| e.size).sum(),
        total_compressed_size: entries.iter().map(|e| e.compressed_size).sum(),
        entries,
    }
}
