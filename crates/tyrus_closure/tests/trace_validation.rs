#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use std::fs;

use tyrus_closure::{lock_from_tree, parse_trace, validate_trace, TraceKind};

const TRACE: &str = concat!(
    "{\"kind\":\"load\",\"path\":\"app/main.js\"}\n",
    "\n",
    "{\"kind\":\"read\",\"path\":\"node_modules/a/index.js\"}\n",
    "{\"kind\":\"external\",\"path\":\"/usr/lib/x.so\"}\n",
);

fn tree(label: &str) -> support::TempTree {
    let tree = support::TempTree::new(label);
    tree.write("app/main.js", b"require('a');\n");
    tree.write("node_modules/a/index.js", b"module.exports = 1;\n");
    tree
}

#[test]
fn parses_kinds_and_skips_blank_lines() {
    let entries = parse_trace(TRACE).unwrap();
    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].kind, TraceKind::Load);
    assert_eq!(entries[2].kind, TraceKind::External);
}

#[test]
fn invalid_line_is_e7203_with_line_number() {
    let error = parse_trace("{\"kind\":\"load\",\"path\":\"a.js\"}\nnão é json\n").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7203");
    assert!(error.to_string().contains("linha 2"), "{error}");
}

#[test]
fn escaping_path_is_e7203() {
    let error = parse_trace("{\"kind\":\"load\",\"path\":\"../x.js\"}\n").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7203");
}

#[test]
fn complete_closure_validates_and_counts_external() {
    let tree = tree("valido");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    let report = validate_trace(tree.path(), &lock, &parse_trace(TRACE).unwrap()).unwrap();
    assert_eq!((report.checked, report.external), (2, 1));
}

#[test]
fn file_removed_from_disk_is_listed() {
    let tree = tree("removido");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    fs::remove_file(tree.path().join("node_modules/a/index.js")).unwrap();
    let error = validate_trace(tree.path(), &lock, &parse_trace(TRACE).unwrap()).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7202");
    assert!(
        error.to_string().contains("node_modules/a/index.js"),
        "{error}"
    );
}

#[test]
fn traced_file_absent_from_lock_lists_all_missing() {
    let tree = tree("fora-do-lock");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    tree.write("node_modules/c/index.js", b"x");
    let text = format!(
        "{TRACE}{}\n{}\n",
        "{\"kind\":\"load\",\"path\":\"node_modules/c/index.js\"}",
        "{\"kind\":\"load\",\"path\":\"node_modules/d/index.js\"}"
    );
    let error = validate_trace(tree.path(), &lock, &parse_trace(&text).unwrap()).unwrap_err();
    let message = error.to_string();
    assert!(message.contains("node_modules/c/index.js"), "{message}");
    assert!(message.contains("node_modules/d/index.js"), "{message}");
}

#[test]
fn altered_byte_is_e7101() {
    let tree = tree("byte");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    tree.write("node_modules/a/index.js", b"module.exports = 2;\n");
    let error = validate_trace(tree.path(), &lock, &parse_trace(TRACE).unwrap()).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7101");
}

#[test]
fn untouched_file_with_altered_byte_is_still_e7101() {
    let tree = tree("byte-fora-do-rastro");
    tree.write("node_modules/z/unused.js", b"1");
    let (lock, _) = lock_from_tree(tree.path()).unwrap();
    tree.write("node_modules/z/unused.js", b"2");
    let error = validate_trace(tree.path(), &lock, &parse_trace(TRACE).unwrap()).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7101");
}
