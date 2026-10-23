#![allow(clippy::expect_used, clippy::unwrap_used)]
use tyrus_closure::dev_check::ensure_no_dev;
use tyrus_facts::{sha256_hex, ClosureFile, FileMode};

fn files(paths: &[&str]) -> Vec<ClosureFile> {
    paths
        .iter()
        .map(|path| ClosureFile::new(path, FileMode::Regular, sha256_hex(b"x")).unwrap())
        .collect()
}

#[test]
fn clean_tree_passes() {
    ensure_no_dev(&files(&["app/main.js", "node_modules/pg/index.js"]), None).unwrap();
}

#[test]
fn typescript_is_e7201() {
    let error = ensure_no_dev(&files(&["node_modules/typescript/lib/tsc.js"]), None).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7201");
    assert!(
        error.to_string().contains("node_modules/typescript"),
        "{error}"
    );
}

#[test]
fn types_scope_is_e7201() {
    let error = ensure_no_dev(&files(&["node_modules/@types/node/index.d.ts"]), None).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7201");
}

#[test]
fn nested_typescript_is_e7201() {
    let nested = files(&["node_modules/a/node_modules/typescript/package.json"]);
    assert!(ensure_no_dev(&nested, None).is_err());
}

#[test]
fn similarly_named_package_is_allowed() {
    ensure_no_dev(&files(&["node_modules/typescript-eslint-x/index.js"]), None).unwrap();
}

#[test]
fn lockfile_dev_entry_present_in_tree_is_e7201() {
    let lockfile =
        br#"{"packages":{"":{},"node_modules/webpack":{"dev":true},"node_modules/pg":{}}}"#;
    let tree = files(&["node_modules/webpack/lib/x.js", "node_modules/pg/index.js"]);
    let error = ensure_no_dev(&tree, Some(lockfile)).unwrap_err();
    assert!(
        error.to_string().contains("node_modules/webpack"),
        "{error}"
    );
}

#[test]
fn lockfile_dev_entry_absent_from_tree_passes() {
    let lockfile = br#"{"packages":{"node_modules/webpack":{"dev":true}}}"#;
    ensure_no_dev(&files(&["node_modules/pg/index.js"]), Some(lockfile)).unwrap();
}

#[test]
fn malformed_lockfile_is_refused() {
    assert!(ensure_no_dev(&files(&["app/a.js"]), Some(b"{")).is_err());
}

#[test]
fn types_package_that_the_lockfile_lists_as_production_is_allowed() {
    let lockfile = br#"{"packages":{"node_modules/@types/validator":{},"node_modules/@types/node":{"dev":true}}}"#;
    let tree = files(&["node_modules/@types/validator/index.d.ts"]);
    ensure_no_dev(&tree, Some(lockfile)).unwrap();
}

#[test]
fn types_package_that_the_lockfile_marks_dev_is_e7201() {
    let lockfile = br#"{"packages":{"node_modules/@types/node":{"dev":true}}}"#;
    let tree = files(&["node_modules/@types/node/index.d.ts"]);
    let error = ensure_no_dev(&tree, Some(lockfile)).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7201");
}

#[test]
fn types_package_absent_from_the_lockfile_is_e7201() {
    let lockfile = br#"{"packages":{"node_modules/pg":{}}}"#;
    let tree = files(&["node_modules/@types/stray/index.d.ts"]);
    assert!(ensure_no_dev(&tree, Some(lockfile)).is_err());
}

#[test]
fn typescript_is_refused_even_when_the_lockfile_lists_it_as_production() {
    let lockfile = br#"{"packages":{"node_modules/typescript":{}}}"#;
    let tree = files(&["node_modules/typescript/lib/tsc.js"]);
    assert!(ensure_no_dev(&tree, Some(lockfile)).is_err());
}
