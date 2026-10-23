#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::Mutex;

use tyrus_closure::{build_closure, BuildRequest};
use tyrus_facts::{parse_closure_lock, ClosureFile};

const OK_NPM: &str = "#!/bin/sh\nmkdir -p node_modules/pg\necho \"$@\" > node_modules/.args\necho 1 > node_modules/pg/index.js\n";
const DEV_NPM: &str =
    "#!/bin/sh\nmkdir -p node_modules/typescript\necho 1 > node_modules/typescript/package.json\n";
const FAILING_NPM: &str = "#!/bin/sh\necho 'registry fora do ar' >&2\nexit 7\n";

const PROD_TYPES_NPM: &str =
    "#!/bin/sh\nmkdir -p node_modules/@types/prod\necho 1 > node_modules/@types/prod/index.d.ts\n";
const DEV_TYPES_NPM: &str =
    "#!/bin/sh\nmkdir -p node_modules/devonly\necho 1 > node_modules/devonly/index.js\n";
const PROD_TYPES_LOCK: &[u8] =
    b"{\"lockfileVersion\":3,\"packages\":{\"node_modules/@types/prod\":{}}}\n";
const DEV_ONLY_LOCK: &[u8] =
    b"{\"lockfileVersion\":3,\"packages\":{\"node_modules/devonly\":{\"dev\":true}}}\n";

// Escrever o npm falso enquanto outra thread faz fork deixa o arquivo aberto no filho: ETXTBSY.
static SPAWN_LOCK: Mutex<()> = Mutex::new(());

struct Fixture {
    root: support::TempTree,
}

impl Fixture {
    fn new(label: &str, npm_script: &str) -> Self {
        Self::with_lockfile(label, npm_script, b"{\"lockfileVersion\":3}\n")
    }

    fn with_lockfile(label: &str, npm_script: &str, lockfile: &[u8]) -> Self {
        let _guard = SPAWN_LOCK
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let root = support::TempTree::new(label);
        root.write("project/package.json", b"{}\n");
        root.write("project/package-lock.json", lockfile);
        root.write("emit/main.js", b"require('pg');\n");
        root.write("emit/nested/util.js", b"exports.a = 1;\n");
        let npm = root.write("fake-npm", npm_script.as_bytes());
        fs::set_permissions(&npm, fs::Permissions::from_mode(0o755)).unwrap();
        Self { root }
    }
}

fn build(
    fixture: &Fixture,
    out_name: &str,
) -> Result<tyrus_closure::BuildReport, tyrus_closure::ClosureError> {
    let base = fixture.root.path();
    let out = base.join(out_name);
    let (project, emit, npm) = (
        base.join("project"),
        base.join("emit"),
        base.join("fake-npm"),
    );
    let _guard = SPAWN_LOCK
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    build_closure(&BuildRequest {
        project: &project,
        emit: &emit,
        out: &out,
        npm_cache: Some(Path::new("/cache/npm")),
        npm_program: Some(&npm),
    })
}

#[test]
fn builds_tree_with_app_emit_and_a_lock_that_matches_every_regular_file() {
    let fixture = Fixture::new("build-ok", OK_NPM);
    let report = build(&fixture, "out").unwrap();
    let paths: Vec<&str> = report.lock.files().iter().map(ClosureFile::path).collect();
    assert_eq!(
        paths,
        [
            "app/main.js",
            "app/nested/util.js",
            "node_modules/.args",
            "node_modules/pg/index.js",
            "package-lock.json",
            "package.json"
        ]
    );
    let on_disk = fs::read(&report.lock_path).unwrap();
    assert_eq!(parse_closure_lock(&on_disk).unwrap(), report.lock);
    report.lock.verify(&report.tree).unwrap();
}

#[test]
fn runs_npm_ci_omitting_dev_with_the_given_cache() {
    let fixture = Fixture::new("build-args", OK_NPM);
    let report = build(&fixture, "out").unwrap();
    let args = fs::read_to_string(report.tree.join("node_modules/.args")).unwrap();
    assert_eq!(
        args.trim(),
        "ci --omit=dev --no-audit --no-fund --cache /cache/npm"
    );
}

#[test]
fn two_builds_produce_byte_identical_locks() {
    let fixture = Fixture::new("build-det", OK_NPM);
    let first = build(&fixture, "out-a").unwrap();
    let second = build(&fixture, "out-b").unwrap();
    assert_eq!(
        fs::read(first.lock_path).unwrap(),
        fs::read(second.lock_path).unwrap()
    );
}

#[test]
fn dev_dependency_in_the_tree_is_refused_with_e7201() {
    let fixture = Fixture::new("build-dev", DEV_NPM);
    let error = build(&fixture, "out").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7201");
}

#[test]
fn npm_failure_is_e7204_with_its_stderr() {
    let fixture = Fixture::new("build-npm-falha", FAILING_NPM);
    let error = build(&fixture, "out").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7204");
    assert!(error.to_string().contains("registry fora do ar"), "{error}");
}

#[test]
fn non_empty_output_directory_is_refused() {
    let fixture = Fixture::new("build-sujo", OK_NPM);
    fixture.root.write("out/leftover", b"x");
    let error = build(&fixture, "out").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7205");
}

#[test]
fn missing_lockfile_is_refused() {
    let fixture = Fixture::new("build-sem-lock", OK_NPM);
    fs::remove_file(fixture.root.path().join("project/package-lock.json")).unwrap();
    assert_eq!(
        build(&fixture, "out").unwrap_err().stable_code(),
        "TYRUS-E7205"
    );
}

#[test]
fn types_package_listed_as_production_in_the_lockfile_builds() {
    let fixture = Fixture::with_lockfile("build-types-prod", PROD_TYPES_NPM, PROD_TYPES_LOCK);
    let report = build(&fixture, "out").unwrap();
    assert!(report
        .lock
        .files()
        .iter()
        .any(|file| file.path() == "node_modules/@types/prod/index.d.ts"));
}

#[test]
fn package_marked_dev_in_the_lockfile_is_refused_with_e7201() {
    let fixture = Fixture::with_lockfile("build-dev-lock", DEV_TYPES_NPM, DEV_ONLY_LOCK);
    let error = build(&fixture, "out").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7201");
    assert!(
        error.to_string().contains("node_modules/devonly"),
        "{error}"
    );
}
