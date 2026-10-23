#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use std::fs;
use std::os::unix::fs::{symlink, PermissionsExt};

use tyrus_closure::{collect_inventory, lock_from_tree};
use tyrus_facts::{canonical_bytes, parse_closure_lock, sha256_hex, ClosureFile, FileMode};

fn sample(label: &str) -> support::TempTree {
    let tree = support::TempTree::new(label);
    tree.write("package.json", b"{}\n");
    tree.write("node_modules/b/index.js", b"module.exports = 2;\n");
    tree.write("node_modules/a/index.js", b"module.exports = 1;\n");
    let cli = tree.write("node_modules/a/bin/cli.js", b"#!/usr/bin/env node\n");
    fs::set_permissions(&cli, fs::Permissions::from_mode(0o775)).unwrap();
    tree.write("app/main.js", b"require('a');\n");
    tree
}

#[test]
fn lists_exactly_the_regular_files_in_byte_order() {
    let tree = sample("ordem");
    let inventory = collect_inventory(tree.path()).unwrap();
    let paths: Vec<&str> = inventory.files.iter().map(ClosureFile::path).collect();
    assert_eq!(
        paths,
        [
            "app/main.js",
            "node_modules/a/bin/cli.js",
            "node_modules/a/index.js",
            "node_modules/b/index.js",
            "package.json"
        ]
    );
}

#[test]
fn normalizes_mode_and_records_sha256() {
    let tree = sample("modo");
    let inventory = collect_inventory(tree.path()).unwrap();
    let cli = &inventory.files[1];
    assert_eq!(cli.mode(), FileMode::Executable);
    assert_eq!(inventory.files[0].mode(), FileMode::Regular);
    assert_eq!(inventory.files[0].sha256(), &sha256_hex(b"require('a');\n"));
}

#[test]
fn reports_symlinks_instead_of_dropping_them_silently() {
    let tree = sample("symlink");
    fs::create_dir_all(tree.path().join("node_modules/.bin")).unwrap();
    symlink("../a/bin/cli.js", tree.path().join("node_modules/.bin/cli")).unwrap();
    let inventory = collect_inventory(tree.path()).unwrap();
    assert_eq!(inventory.symlinks, ["node_modules/.bin/cli"]);
    assert_eq!(inventory.files.len(), 5);
}

#[test]
fn lock_round_trips_through_tyrus_facts_and_verifies() {
    let tree = sample("ida-volta");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    let bytes = canonical_bytes(&lock).unwrap();
    let parsed = parse_closure_lock(&bytes).unwrap();
    assert_eq!(parsed, lock);
    parsed.verify(tree.path()).unwrap();
}
