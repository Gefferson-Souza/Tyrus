use std::path::Path;

use crate::digest::{sha256, to_hex, Sha256Bytes};
use crate::error::{malformed, ArchiveError};
use crate::format::{
    set_digest, validate_path, Cursor, TableEntry, FOOTER_LEN, FORMAT_VERSION, HEADER_LEN,
    MAGIC_HEAD, MAGIC_TAIL, MAX_ENTRY_SIZE, MODE_EXEC, MODE_FILE,
};

/// Metadados públicos de uma entrada.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntryInfo {
    pub path: String,
    pub mode: u32,
    pub size: u64,
    pub compressed_size: u64,
    pub sha256: String,
}

#[derive(Debug)]
struct Slot {
    offset: usize,
    digest: Sha256Bytes,
}

#[derive(Debug)]
pub struct ArchiveReader {
    bytes: Vec<u8>,
    infos: Vec<EntryInfo>,
    slots: Vec<Slot>,
    set_digest: Sha256Bytes,
}

impl ArchiveReader {
    /// # Errors
    /// `TYRUS-E8105` na leitura; os demais, os de [`Self::from_bytes`].
    pub fn open(path: &Path) -> Result<Self, ArchiveError> {
        Self::from_bytes(std::fs::read(path)?)
    }

    /// Valida estrutura e o hash do conjunto; o hash de cada entrada é conferido em [`Self::read`].
    ///
    /// # Errors
    /// `TYRUS-E8102` (versão), `TYRUS-E8103` (estrutura), `TYRUS-E8101` (hash do conjunto).
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, ArchiveError> {
        if bytes.len() < HEADER_LEN + FOOTER_LEN {
            return Err(malformed("arquivo menor que cabeçalho mais rodapé"));
        }
        let mut head = Cursor::new(&bytes);
        if head.take(MAGIC_HEAD.len())? != MAGIC_HEAD {
            return Err(malformed("magic inicial ausente"));
        }
        let version = head.u32()?;
        if version != FORMAT_VERSION {
            return Err(ArchiveError::UnknownFormatVersion {
                found: version,
                supported: FORMAT_VERSION,
            });
        }
        let count = head.u32()?;
        let footer_start = bytes.len() - FOOTER_LEN;
        let mut footer = Cursor::new(
            bytes
                .get(footer_start..)
                .ok_or_else(|| malformed("rodapé"))?,
        );
        let table_offset = usize::try_from(footer.u64()?).map_err(|_| malformed("table_offset"))?;
        let stored_set: Sha256Bytes = footer.take(32)?.try_into().map_err(|_| malformed("hash"))?;
        if footer.take(MAGIC_TAIL.len())? != MAGIC_TAIL {
            return Err(malformed("magic final ausente"));
        }
        let table_bytes = bytes
            .get(table_offset..footer_start)
            .filter(|_| table_offset >= HEADER_LEN)
            .ok_or_else(|| malformed("table_offset fora do arquivo"))?;
        let table = parse_table(table_bytes, count, table_offset)?;
        if set_digest(&table) != stored_set {
            return Err(ArchiveError::HashMismatch {
                subject: "conjunto".to_owned(),
                reason: "o hash do rodapé não confere com a tabela".to_owned(),
            });
        }
        Ok(Self::assemble(bytes, &table, stored_set))
    }

    fn assemble(bytes: Vec<u8>, table: &[TableEntry], set_digest: Sha256Bytes) -> Self {
        let infos = table
            .iter()
            .map(|entry| EntryInfo {
                path: entry.path.clone(),
                mode: entry.mode,
                size: entry.size,
                compressed_size: entry.compressed_size,
                sha256: to_hex(&entry.sha256),
            })
            .collect();
        let slots = table
            .iter()
            .map(|entry| Slot {
                offset: usize::try_from(entry.offset).unwrap_or(usize::MAX),
                digest: entry.sha256,
            })
            .collect();
        Self {
            bytes,
            infos,
            slots,
            set_digest,
        }
    }

    #[must_use]
    pub fn entries(&self) -> &[EntryInfo] {
        &self.infos
    }

    #[must_use]
    pub fn set_sha256(&self) -> String {
        to_hex(&self.set_digest)
    }

    #[must_use]
    pub const fn format_version(&self) -> u32 {
        FORMAT_VERSION
    }

    /// Descomprime a entrada e confere tamanho e sha256.
    ///
    /// # Errors
    /// `TYRUS-E8104` se o caminho não existe; `TYRUS-E8101` se o conteúdo não bate com o hash.
    pub fn read(&self, path: &str) -> Result<Vec<u8>, ArchiveError> {
        let index = self
            .infos
            .binary_search_by(|info| info.path.as_str().cmp(path))
            .map_err(|_| ArchiveError::InvalidEntry {
                path: path.to_owned(),
                reason: "não existe no arquivo".to_owned(),
            })?;
        let (info, slot) = self
            .infos
            .get(index)
            .zip(self.slots.get(index))
            .ok_or_else(|| malformed("índice"))?;
        let mismatch = |reason: &str| ArchiveError::HashMismatch {
            subject: path.to_owned(),
            reason: reason.to_owned(),
        };
        let content = if info.size == 0 {
            Vec::new()
        } else {
            let end = slot
                .offset
                .saturating_add(usize::try_from(info.compressed_size).unwrap_or(usize::MAX));
            let blob = self
                .bytes
                .get(slot.offset..end)
                .ok_or_else(|| malformed("blob fora do arquivo"))?;
            let capacity = usize::try_from(info.size).map_err(|_| malformed("tamanho"))?;
            zstd::bulk::decompress(blob, capacity)
                .map_err(|_| mismatch("a entrada não descomprime"))?
        };
        if content.len() as u64 != info.size || sha256(&content) != slot.digest {
            return Err(mismatch("o conteúdo não confere com o sha256 da tabela"));
        }
        Ok(content)
    }

    /// # Errors
    /// O primeiro erro de [`Self::read`] entre todas as entradas.
    pub fn verify(&self) -> Result<(), ArchiveError> {
        for info in &self.infos {
            self.read(&info.path)?;
        }
        Ok(())
    }
}

fn parse_table(
    bytes: &[u8],
    count: u32,
    blobs_end: usize,
) -> Result<Vec<TableEntry>, ArchiveError> {
    let mut cursor = Cursor::new(bytes);
    let mut table: Vec<TableEntry> = Vec::new();
    let mut next_offset = HEADER_LEN as u64;
    for _ in 0..count {
        let entry = TableEntry::decode(&mut cursor)?;
        check_entry(&entry, table.last(), next_offset)?;
        next_offset = entry.offset + entry.compressed_size;
        table.push(entry);
    }
    if !cursor.is_done() {
        return Err(malformed("bytes sobrando depois da tabela"));
    }
    if next_offset != blobs_end as u64 {
        return Err(malformed("blobs não cobrem a região até a tabela"));
    }
    Ok(table)
}

fn check_entry(
    entry: &TableEntry,
    previous: Option<&TableEntry>,
    expected_offset: u64,
) -> Result<(), ArchiveError> {
    validate_path(&entry.path).map_err(malformed)?;
    if previous.is_some_and(|prev| prev.path >= entry.path) {
        return Err(malformed("entradas fora de ordem ou duplicadas"));
    }
    if !matches!(entry.mode, MODE_FILE | MODE_EXEC) {
        return Err(malformed("modo não normalizado"));
    }
    if entry.size > MAX_ENTRY_SIZE {
        return Err(malformed("entrada acima do limite"));
    }
    if entry.offset != expected_offset || (entry.size == 0) != (entry.compressed_size == 0) {
        return Err(malformed("deslocamento ou tamanho de blob incoerente"));
    }
    entry
        .offset
        .checked_add(entry.compressed_size)
        .ok_or_else(|| malformed("deslocamento"))?;
    Ok(())
}
