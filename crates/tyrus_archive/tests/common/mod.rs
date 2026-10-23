#![allow(
    dead_code,
    unreachable_pub,
    clippy::indexing_slicing,
    clippy::expect_used,
    clippy::unwrap_used
)]

use std::path::{Path, PathBuf};

use tyrus_archive::{ArchiveReader, ArchiveWriter};

pub struct SourceFile {
    pub path: String,
    pub content: Vec<u8>,
    pub executable: bool,
}

pub fn sample_files() -> Vec<SourceFile> {
    vec![
        SourceFile {
            path: "node_modules/a/index.js".into(),
            content: b"module.exports = 1;\n".repeat(50),
            executable: false,
        },
        SourceFile {
            path: "bin/run.sh".into(),
            content: b"#!/bin/sh\necho ok\n".to_vec(),
            executable: true,
        },
        SourceFile {
            path: "empty.txt".into(),
            content: Vec::new(),
            executable: false,
        },
        SourceFile {
            path: "dist/main.js".into(),
            content: (0..=255u8).cycle().take(4096).collect(),
            executable: false,
        },
    ]
}

pub fn write_archive(files: &[SourceFile]) -> Vec<u8> {
    let mut writer = ArchiveWriter::new();
    for file in files {
        writer
            .add(&file.path, &file.content, file.executable)
            .unwrap();
    }
    writer.finish().unwrap()
}

pub fn open(bytes: Vec<u8>) -> ArchiveReader {
    ArchiveReader::from_bytes(bytes).unwrap()
}

fn walk(dir: &Path, root: &Path, out: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            if path.file_name().is_some_and(|n| n != "node_modules") {
                walk(&path, root, out);
            }
        } else {
            out.push(path.strip_prefix(root).unwrap().to_path_buf());
        }
    }
}

/// Árvore real da réplica de desenvolvimento (sem `node_modules`).
pub fn replica_tree() -> Vec<SourceFile> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../oracle/apps/replica");
    let mut paths = Vec::new();
    walk(&root, &root, &mut paths);
    paths
        .into_iter()
        .map(|rel| SourceFile {
            content: std::fs::read(root.join(&rel)).unwrap(),
            path: rel.to_string_lossy().replace('\\', "/"),
            executable: rel.extension().is_some_and(|e| e == "sh"),
        })
        .collect()
}
