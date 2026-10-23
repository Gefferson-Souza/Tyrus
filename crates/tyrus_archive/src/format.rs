//! Layout binário (inteiros little-endian): cabeçalho | blobs | tabela | rodapé.
use crate::digest::{sha256, Sha256Bytes};
use crate::error::{malformed, ArchiveError};

pub(crate) const MAGIC_HEAD: [u8; 8] = *b"TYRUSAR\0";
pub(crate) const MAGIC_TAIL: [u8; 8] = *b"TYRUSEND";
pub const FORMAT_VERSION: u32 = 1;
pub const COMPRESSION_LEVEL: i32 = 3;
pub(crate) const HEADER_LEN: usize = 16;
pub(crate) const FOOTER_LEN: usize = 48;
pub(crate) const MODE_FILE: u32 = 0o644;
pub(crate) const MODE_EXEC: u32 = 0o755;
pub(crate) const MAX_ENTRY_SIZE: u64 = 1 << 30;
pub(crate) const MAX_PATH_LEN: usize = 4096;

#[derive(Debug, Clone)]
pub(crate) struct TableEntry {
    pub(crate) path: String,
    pub(crate) mode: u32,
    pub(crate) size: u64,
    pub(crate) offset: u64,
    pub(crate) compressed_size: u64,
    pub(crate) sha256: Sha256Bytes,
}

impl TableEntry {
    pub(crate) fn encode(&self, out: &mut Vec<u8>) {
        out.extend_from_slice(&path_len(&self.path).to_le_bytes());
        out.extend_from_slice(self.path.as_bytes());
        out.extend_from_slice(&self.mode.to_le_bytes());
        out.extend_from_slice(&self.size.to_le_bytes());
        out.extend_from_slice(&self.offset.to_le_bytes());
        out.extend_from_slice(&self.compressed_size.to_le_bytes());
        out.extend_from_slice(&self.sha256);
    }

    pub(crate) fn decode(cursor: &mut Cursor<'_>) -> Result<Self, ArchiveError> {
        let len = usize::try_from(cursor.u32()?).map_err(|_| malformed("caminho grande demais"))?;
        if len > MAX_PATH_LEN {
            return Err(malformed("caminho grande demais"));
        }
        let path = String::from_utf8(cursor.take(len)?.to_vec())
            .map_err(|_| malformed("caminho que não é UTF-8"))?;
        Ok(Self {
            path,
            mode: cursor.u32()?,
            size: cursor.u64()?,
            offset: cursor.u64()?,
            compressed_size: cursor.u64()?,
            sha256: cursor
                .take(32)?
                .try_into()
                .map_err(|_| malformed("hash truncado"))?,
        })
    }
}

fn path_len(path: &str) -> u32 {
    u32::try_from(path.len()).unwrap_or(u32::MAX)
}

/// Hash da listagem canônica: não depende da compressão nem do deslocamento dos blobs.
pub(crate) fn set_digest(entries: &[TableEntry]) -> Sha256Bytes {
    let mut listing = Vec::new();
    for entry in entries {
        listing.extend_from_slice(&path_len(&entry.path).to_le_bytes());
        listing.extend_from_slice(entry.path.as_bytes());
        listing.extend_from_slice(&entry.mode.to_le_bytes());
        listing.extend_from_slice(&entry.size.to_le_bytes());
        listing.extend_from_slice(&entry.sha256);
    }
    sha256(&listing)
}

pub(crate) fn validate_path(path: &str) -> Result<(), &'static str> {
    if path.is_empty() || path.len() > MAX_PATH_LEN {
        return Err("caminho vazio ou grande demais");
    }
    if path.starts_with('/') || path.contains('\\') || path.contains('\0') {
        return Err("caminho deve ser relativo, com `/` como separador");
    }
    if path.split('/').any(|part| matches!(part, "" | "." | "..")) {
        return Err("caminho com componente vazio, `.` ou `..`");
    }
    Ok(())
}

pub(crate) struct Cursor<'a> {
    bytes: &'a [u8],
    position: usize,
}

impl<'a> Cursor<'a> {
    pub(crate) const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, position: 0 }
    }

    pub(crate) const fn is_done(&self) -> bool {
        self.position == self.bytes.len()
    }

    pub(crate) fn take(&mut self, len: usize) -> Result<&'a [u8], ArchiveError> {
        let end = self
            .position
            .checked_add(len)
            .ok_or_else(|| malformed("deslocamento inválido"))?;
        let slice = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| malformed("tabela truncada"))?;
        self.position = end;
        Ok(slice)
    }

    pub(crate) fn u32(&mut self) -> Result<u32, ArchiveError> {
        let raw = self
            .take(4)?
            .try_into()
            .map_err(|_| malformed("inteiro truncado"))?;
        Ok(u32::from_le_bytes(raw))
    }

    pub(crate) fn u64(&mut self) -> Result<u64, ArchiveError> {
        let raw = self
            .take(8)?
            .try_into()
            .map_err(|_| malformed("inteiro truncado"))?;
        Ok(u64::from_le_bytes(raw))
    }
}
