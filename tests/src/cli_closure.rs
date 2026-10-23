//! `tyrus closure`: montagem do fecho (com um npm de mentira no PATH) e validação do
//! rastro de carga, incluindo a recusa com a lista de arquivos ausentes.

use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use predicates::prelude::*;
use tempfile::{tempdir, TempDir};

use crate::cli::tyrus_cmd;

const FAKE_NPM: &str = "#!/bin/sh\nmkdir -p node_modules/pg\necho 1 > node_modules/pg/index.js\n";
const TRACE: &str = concat!(
    "{\"kind\":\"load\",\"path\":\"app/main.js\"}\n",
    "{\"kind\":\"load\",\"path\":\"node_modules/pg/index.js\"}\n",
    "{\"kind\":\"external\",\"path\":\"/usr/lib/libx.so\"}\n",
);

struct Built {
    dir: TempDir,
    tree: PathBuf,
    lock: PathBuf,
}

fn write(path: &Path, content: &str) {
    std::fs::create_dir_all(path.parent().expect("parent")).expect("mkdir");
    std::fs::write(path, content).expect("write");
}

fn built_closure() -> Built {
    let dir = tempdir().expect("tempdir");
    let root = dir.path();
    write(&root.join("project/package.json"), "{}\n");
    write(
        &root.join("project/package-lock.json"),
        "{\"lockfileVersion\":3}\n",
    );
    write(&root.join("emit/main.js"), "require('pg');\n");
    write(&root.join("bin/npm"), FAKE_NPM);
    std::fs::set_permissions(root.join("bin/npm"), std::fs::Permissions::from_mode(0o755))
        .expect("chmod");
    let path = format!(
        "{}:{}",
        root.join("bin").display(),
        std::env::var("PATH").unwrap_or_default()
    );
    tyrus_cmd()
        .env("PATH", path)
        .args(["--quiet", "closure", "--build"])
        .arg("--project")
        .arg(root.join("project"))
        .arg("--emit")
        .arg(root.join("emit"))
        .arg("--out")
        .arg(root.join("out"))
        .assert()
        .success()
        .stdout(predicate::str::contains("closure: 4 arquivos"));
    let (tree, lock) = (root.join("out/tree"), root.join("out/closure.lock"));
    write(&root.join("trace.jsonl"), TRACE);
    Built { dir, tree, lock }
}

fn validate(built: &Built, tree: &Path) -> assert_cmd::assert::Assert {
    tyrus_cmd()
        .args(["--quiet", "closure", "--validate"])
        .arg("--root")
        .arg(tree)
        .arg("--lock")
        .arg(&built.lock)
        .arg("--trace")
        .arg(built.dir.path().join("trace.jsonl"))
        .assert()
}

#[test]
fn validate_passes_with_a_complete_closure() {
    let built = built_closure();
    validate(&built, &built.tree)
        .success()
        .stdout(predicate::str::contains("2 arquivos do rastro"));
}

#[test]
fn validate_exits_one_and_lists_the_missing_file() {
    let built = built_closure();
    let copy = built.dir.path().join("copy");
    write(&copy.join("app/main.js"), "require('pg');\n");
    validate(&built, &copy)
        .failure()
        .code(1)
        .stderr(predicate::str::contains("TYRUS-E7202"))
        .stderr(predicate::str::contains("node_modules/pg/index.js"));
}

#[test]
fn validate_rejects_a_tampered_byte_with_e7101() {
    let built = built_closure();
    write(&built.tree.join("node_modules/pg/index.js"), "2\n");
    validate(&built, &built.tree)
        .failure()
        .stderr(predicate::str::contains("TYRUS-E7101"));
}

#[test]
fn mode_flags_are_required_and_exclusive() {
    tyrus_cmd().args(["--quiet", "closure"]).assert().failure();
    tyrus_cmd()
        .args(["--quiet", "closure", "--build", "--validate"])
        .assert()
        .failure();
}

#[test]
fn build_requires_its_paths() {
    tyrus_cmd()
        .args(["--quiet", "closure", "--build"])
        .assert()
        .failure();
}
