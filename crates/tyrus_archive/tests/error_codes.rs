#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn codes_in(dir: &Path) -> Vec<String> {
    let mut files = Vec::new();
    collect_rust_files(&dir.join("src"), &mut files);
    let marker = "TYRUS-E";
    let mut found = Vec::new();
    for file in files {
        let text = std::fs::read_to_string(file).unwrap();
        for (start, _) in text.match_indices(marker) {
            if let Some(digits) = text.get(start + marker.len()..start + marker.len() + 4) {
                if digits.bytes().all(|b| b.is_ascii_digit()) {
                    found.push(format!("{marker}{digits}"));
                }
            }
        }
    }
    found
}

#[test]
fn archive_codes_are_in_the_e81_block_with_one_literal_each() {
    let source = include_str!("../src/error.rs");
    assert_eq!(source.matches("\"TYRUS-E8").count(), 5);
    let own: BTreeSet<String> = codes_in(Path::new(env!("CARGO_MANIFEST_DIR")))
        .into_iter()
        .collect();
    assert!(own.contains("TYRUS-E8101") && own.contains("TYRUS-E8102"));
    for code in &own {
        assert!(code.starts_with("TYRUS-E81"), "{code}");
    }
}

#[test]
fn no_other_crate_reuses_an_archive_code() {
    let own: BTreeSet<String> = codes_in(Path::new(env!("CARGO_MANIFEST_DIR")))
        .into_iter()
        .collect();
    let crates_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    for entry in std::fs::read_dir(crates_dir).unwrap() {
        let member = entry.unwrap().path();
        if member.file_name().is_some_and(|n| n == "tyrus_archive") || !member.join("src").is_dir()
        {
            continue;
        }
        for code in codes_in(&member) {
            assert!(
                !own.contains(&code),
                "{code} reusado em {}",
                member.display()
            );
        }
    }
}
