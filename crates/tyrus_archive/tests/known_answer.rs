#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::{open, write_archive, SourceFile};
use std::fmt::Write;

use sha2::{Digest, Sha256};

// Vetor de resposta conhecida: o hash do conjunto foi calculado por script Python
// independente (struct + hashlib), não pelo escritor.
const SET_SHA256: &str = "00e841f87284514bc37e0bcaa9d6e004e0a8dbb53cea58796a81c094b0bf42e2";
const FILE_SHA256: &str = "c199eec20e847244c1cd1aee5a6aeb6fed7a2377b53fb5db9328790b95087b81";

fn log_text() -> String {
    (0..6000u64).fold(String::new(), |mut text, i| {
        let _ = writeln!(text, "linha {i}: {} {:x}", i * i % 9973, i * 7919 % 104_729);
        text
    })
}

fn vector() -> Vec<SourceFile> {
    vec![
        SourceFile {
            path: "src/data.txt".into(),
            content: b"tyrus\n".repeat(64),
            executable: false,
        },
        SourceFile {
            path: "bin/run.sh".into(),
            content: b"#!/bin/sh\necho ok\n".to_vec(),
            executable: true,
        },
        SourceFile {
            path: "src/log.txt".into(),
            content: log_text().into_bytes(),
            executable: false,
        },
        SourceFile {
            path: "a/empty".into(),
            content: Vec::new(),
            executable: false,
        },
    ]
}

fn file_sha256(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

#[test]
fn set_hash_matches_the_independently_computed_vector() {
    let reader = open(write_archive(&vector()));
    assert_eq!(reader.set_sha256(), SET_SHA256);
}

#[test]
fn whole_file_bytes_are_pinned_including_zstd_level_and_layout() {
    let bytes = write_archive(&vector());
    assert_eq!(file_sha256(&bytes), FILE_SHA256);
}
