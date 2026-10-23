#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::{open, replica_tree, sample_files, write_archive};

#[test]
fn replica_tree_survives_round_trip_with_every_entry_and_the_set_hash() {
    let files = replica_tree();
    assert!(
        files.len() > 20,
        "a árvore da réplica deveria ter dezenas de arquivos"
    );
    let reader = open(write_archive(&files));
    reader.verify().unwrap();
    assert_eq!(reader.entries().len(), files.len());
    for file in &files {
        assert_eq!(
            reader.read(&file.path).unwrap(),
            file.content,
            "{}",
            file.path
        );
    }
    let again = open(write_archive(&files));
    assert_eq!(reader.set_sha256(), again.set_sha256());
}

#[test]
fn entries_are_sorted_with_normalized_modes() {
    let reader = open(write_archive(&sample_files()));
    let paths: Vec<&str> = reader.entries().iter().map(|e| e.path.as_str()).collect();
    let mut sorted = paths.clone();
    sorted.sort_unstable();
    assert_eq!(paths, sorted);
    let script = reader
        .entries()
        .iter()
        .find(|e| e.path == "bin/run.sh")
        .unwrap();
    assert_eq!(script.mode, 0o755);
    let plain = reader
        .entries()
        .iter()
        .find(|e| e.path == "empty.txt")
        .unwrap();
    assert_eq!(plain.mode, 0o644);
}

#[test]
fn missing_entry_is_reported_as_invalid_entry() {
    let reader = open(write_archive(&sample_files()));
    let error = reader.read("nao/existe").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E8104");
}
