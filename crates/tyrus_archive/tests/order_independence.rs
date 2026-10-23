#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::{replica_tree, write_archive, SourceFile};

/// Embaralhamento Fisher-Yates com xorshift: determinístico, sem dependência nova.
fn shuffled(files: &[SourceFile], seed: u64) -> Vec<SourceFile> {
    let mut state = seed | 1;
    let mut next = move || {
        state ^= state << 13;
        state ^= state >> 7;
        state ^= state << 17;
        state
    };
    let mut indices: Vec<usize> = (0..files.len()).collect();
    for i in (1..indices.len()).rev() {
        let j = usize::try_from(next() % (i as u64 + 1)).unwrap();
        indices.swap(i, j);
    }
    indices
        .into_iter()
        .map(|i| SourceFile {
            path: files[i].path.clone(),
            content: files[i].content.clone(),
            executable: files[i].executable,
        })
        .collect()
}

#[test]
fn insertion_order_never_changes_the_archive_bytes() {
    let files = replica_tree();
    let baseline = write_archive(&files);
    for seed in 1..=64u64 {
        assert_eq!(
            write_archive(&shuffled(&files, seed)),
            baseline,
            "seed {seed}"
        );
    }
}

#[test]
fn duplicate_paths_are_refused() {
    let mut writer = tyrus_archive::ArchiveWriter::new();
    writer.add("a.js", b"1", false).unwrap();
    let error = writer.add("a.js", b"2", false).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E8104");
}

#[test]
fn unsafe_paths_are_refused() {
    for bad in ["", "/abs", "../x", "a/../b", "a\\b", "a//b", "./a"] {
        let mut writer = tyrus_archive::ArchiveWriter::new();
        let error = writer.add(bad, b"x", false).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E8104", "{bad:?}");
    }
}
