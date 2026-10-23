#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::{sample_files, write_archive};
use tyrus_archive::ArchiveReader;

const HEADER_LEN: usize = 16;

fn blob_region(bytes: &[u8]) -> std::ops::Range<usize> {
    let tail = bytes.len() - 48;
    let table_offset = usize::try_from(u64::from_le_bytes(
        bytes[tail..tail + 8].try_into().unwrap(),
    ))
    .unwrap();
    HEADER_LEN..table_offset
}

fn code_of(bytes: Vec<u8>) -> Option<&'static str> {
    let reader = match ArchiveReader::from_bytes(bytes) {
        Ok(reader) => reader,
        Err(error) => return Some(error.stable_code()),
    };
    reader.verify().err().map(|e| e.stable_code())
}

#[test]
fn untouched_archive_verifies() {
    assert_eq!(code_of(write_archive(&sample_files())), None);
}

#[test]
fn any_changed_byte_in_an_entry_blob_is_refused_with_e8101() {
    let original = write_archive(&sample_files());
    let region = blob_region(&original);
    assert!(region.len() > 100);
    for position in region {
        let mut bytes = original.clone();
        bytes[position] ^= 0x01;
        assert_eq!(code_of(bytes), Some("TYRUS-E8101"), "posição {position}");
    }
}

#[test]
fn any_changed_byte_in_the_table_or_footer_hash_is_refused_with_e8101() {
    let original = write_archive(&sample_files());
    let table_start = blob_region(&original).end;
    // Rodapé: table_offset (8) + set_sha256 (32) + magic (8); o hash do conjunto é o miolo.
    let hash_start = original.len() - 40;
    for position in (table_start..original.len() - 48).chain(hash_start..original.len() - 8) {
        let mut bytes = original.clone();
        bytes[position] ^= 0x01;
        let code = code_of(bytes);
        assert!(
            matches!(code, Some("TYRUS-E8101" | "TYRUS-E8103")),
            "posição {position}: {code:?}"
        );
    }
}

#[test]
fn unknown_format_version_is_refused_with_e8102() {
    let mut bytes = write_archive(&sample_files());
    bytes[8..12].copy_from_slice(&2u32.to_le_bytes());
    assert_eq!(code_of(bytes), Some("TYRUS-E8102"));
}

#[test]
fn bad_magic_and_truncation_are_malformed() {
    let original = write_archive(&sample_files());
    let mut bad_magic = original.clone();
    bad_magic[0] ^= 0xFF;
    assert_eq!(code_of(bad_magic), Some("TYRUS-E8103"));
    for len in [0, 7, 15, 40, original.len() - 1] {
        assert_eq!(
            code_of(original[..len].to_vec()),
            Some("TYRUS-E8103"),
            "len {len}"
        );
    }
}
