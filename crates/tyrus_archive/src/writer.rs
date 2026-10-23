use std::collections::BTreeMap;
use std::io::Write;

use crate::digest::sha256;
use crate::error::ArchiveError;
use crate::format::{
    set_digest, validate_path, TableEntry, COMPRESSION_LEVEL, FORMAT_VERSION, MAGIC_HEAD,
    MAGIC_TAIL, MAX_ENTRY_SIZE, MODE_EXEC, MODE_FILE,
};

struct Pending {
    content: Vec<u8>,
    executable: bool,
}

/// Acumula entradas em qualquer ordem; a saída é função só do conjunto de entradas.
#[derive(Default)]
pub struct ArchiveWriter {
    entries: BTreeMap<String, Pending>,
}

impl std::fmt::Debug for ArchiveWriter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ArchiveWriter")
            .field("entries", &self.entries.len())
            .finish()
    }
}

impl ArchiveWriter {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// # Errors
    /// `TYRUS-E8104` para caminho inseguro, duplicado ou conteúdo acima do limite.
    pub fn add(
        &mut self,
        path: &str,
        content: &[u8],
        executable: bool,
    ) -> Result<(), ArchiveError> {
        let invalid = |reason: &str| ArchiveError::InvalidEntry {
            path: path.to_owned(),
            reason: reason.to_owned(),
        };
        validate_path(path).map_err(invalid)?;
        if exceeds_entry_limit(content.len()) {
            return Err(invalid("conteúdo acima do limite por entrada"));
        }
        if self.entries.contains_key(path) {
            return Err(invalid("caminho duplicado"));
        }
        self.entries.insert(
            path.to_owned(),
            Pending {
                content: content.to_vec(),
                executable,
            },
        );
        Ok(())
    }

    /// # Errors
    /// `TYRUS-E8105` se a compressão falhar.
    pub fn finish(self) -> Result<Vec<u8>, ArchiveError> {
        let count = u32::try_from(self.entries.len()).map_err(|_| ArchiveError::InvalidEntry {
            path: String::new(),
            reason: "entradas demais".to_owned(),
        })?;
        let mut out = Vec::new();
        out.extend_from_slice(&MAGIC_HEAD);
        out.extend_from_slice(&FORMAT_VERSION.to_le_bytes());
        out.extend_from_slice(&count.to_le_bytes());
        let mut table = Vec::with_capacity(self.entries.len());
        for (path, pending) in self.entries {
            let blob = compress(&pending.content)?;
            table.push(TableEntry {
                path,
                mode: if pending.executable {
                    MODE_EXEC
                } else {
                    MODE_FILE
                },
                size: pending.content.len() as u64,
                offset: out.len() as u64,
                compressed_size: blob.len() as u64,
                sha256: sha256(&pending.content),
            });
            out.extend_from_slice(&blob);
        }
        let table_offset = out.len() as u64;
        for entry in &table {
            entry.encode(&mut out);
        }
        out.extend_from_slice(&table_offset.to_le_bytes());
        out.extend_from_slice(&set_digest(&table));
        out.extend_from_slice(&MAGIC_TAIL);
        Ok(out)
    }

    /// # Errors
    /// Os de [`Self::finish`] e `TYRUS-E8105` se a escrita falhar.
    pub fn write_to(self, sink: &mut impl Write) -> Result<(), ArchiveError> {
        sink.write_all(&self.finish()?)?;
        Ok(())
    }
}

/// Entrada vazia não tem blob: não há bytes de quadro que possam ser adulterados sem efeito.
fn compress(content: &[u8]) -> Result<Vec<u8>, ArchiveError> {
    if content.is_empty() {
        return Ok(Vec::new());
    }
    Ok(zstd::bulk::compress(content, COMPRESSION_LEVEL)?)
}

fn exceeds_entry_limit(len: usize) -> bool {
    exceeds_limit(len, MAX_ENTRY_SIZE)
}

fn exceeds_limit(len: usize, limit: u64) -> bool {
    u64::try_from(len).map_or(true, |len| len > limit)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    // Calculado por script Python independente (struct + hashlib).
    const SET_DIGEST_VECTOR: &str =
        "3b452c6ea58b6abb97dbabdf9474691960127cf0ad0acfdaf493312e4436ba6b";

    #[test]
    fn limit_is_inclusive_and_rejects_one_byte_over() {
        assert!(!exceeds_limit(10, 10));
        assert!(exceeds_limit(11, 10));
    }

    #[test]
    fn real_limit_is_one_gibibyte() {
        assert!(!exceeds_entry_limit(1 << 30));
        assert!(exceeds_entry_limit((1 << 30) + 1));
    }

    #[test]
    fn set_digest_of_a_fixed_listing_matches_the_independent_vector() {
        let entry = TableEntry {
            path: "a".to_owned(),
            mode: MODE_FILE,
            size: 1,
            offset: 999,
            compressed_size: 999,
            sha256: [7; 32],
        };
        let hex = crate::digest::to_hex(&set_digest(&[entry]));
        assert_eq!(hex, SET_DIGEST_VECTOR);
    }

    #[test]
    fn write_to_emits_exactly_the_finished_bytes() {
        let build = || {
            let mut writer = ArchiveWriter::new();
            writer.add("x/y", b"conteudo", false).unwrap();
            writer
        };
        let mut sink = Vec::new();
        build().write_to(&mut sink).unwrap();
        assert_eq!(sink, build().finish().unwrap());
        assert_eq!(format!("{:?}", build()), "ArchiveWriter { entries: 1 }");
    }
}
