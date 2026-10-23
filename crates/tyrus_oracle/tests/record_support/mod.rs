#![allow(
    dead_code,
    clippy::expect_used,
    clippy::unwrap_used,
    clippy::indexing_slicing
)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tyrus_oracle::record::run_record;

pub(crate) struct Run {
    pub(crate) code: u8,
    pub(crate) stdout: String,
    pub(crate) stderr: String,
}

pub(crate) fn lock_arg() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../oracle/oracle.lock")
        .display()
        .to_string()
}

pub(crate) fn record(items: &[&str], env: impl Fn(&str) -> Option<OsString>) -> Run {
    let args: Vec<String> = items.iter().map(|s| (*s).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run_record(&args, &env, &mut out, &mut err).unwrap();
    Run {
        code,
        stdout: String::from_utf8(out).unwrap(),
        stderr: String::from_utf8(err).unwrap(),
    }
}

pub(crate) fn process_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

pub(crate) fn workdir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("record-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

pub(crate) fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

pub(crate) type Probe = (&'static str, &'static str, &'static str, &'static str);

pub(crate) const DEFAULT_PROBES: [Probe; 4] = [
    (
        "probes",
        "A01_stable",
        "console.log(1);\n",
        "console.log(1);\n",
    ),
    (
        "probes",
        "A02_flaky",
        "console.log(1);\n",
        "console.log(Math.random());\n",
    ),
    (
        "probes",
        "A03_throws",
        "console.log(1);\n",
        "throw new RangeError('boom');\n",
    ),
    (
        "calib",
        "K_changed",
        "console.log(1);\n",
        "console.log(2);\n",
    ),
];

/// Corpus mínimo: fontes `.ts`, JS "emitido" à mão e um `typescript` de mentira.
pub(crate) fn fixture_with(name: &str, probes: &[Probe]) -> (PathBuf, PathBuf) {
    let root = workdir(name);
    let corpus = root.join("corpus");
    let emit = root.join("out/emit");
    for set in ["probes", "calib"] {
        fs::create_dir_all(corpus.join(set)).unwrap();
    }
    for (set, id, ts, js) in probes {
        write(&corpus.join(set).join(format!("{id}.ts")), ts);
        write(&emit.join(set).join(format!("{id}.js")), js);
    }
    write(
        &corpus.join("node_modules/typescript/package.json"),
        "{\"version\": \"5.1.6\"}\n",
    );
    (corpus, root.join("out"))
}

pub(crate) fn fixture(name: &str) -> (PathBuf, PathBuf) {
    fixture_with(name, &DEFAULT_PROBES)
}

pub(crate) fn base_args<'a>(corpus: &'a str, out: &'a str, lock: &'a str) -> Vec<&'a str> {
    vec![
        "--corpus",
        corpus,
        "--emit-dir",
        out,
        "--no-emit",
        "--lock",
        lock,
    ]
}

pub(crate) fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

pub(crate) fn pathdiff(path: &Path, base: &Path) -> String {
    let up = base.components().count();
    let mut relative = PathBuf::new();
    for _ in 0..up {
        relative.push("..");
    }
    relative.push(path.strip_prefix("/").unwrap());
    relative.display().to_string()
}

pub(crate) fn strings(corpus: &Path, out: &Path) -> (String, String, String) {
    (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    )
}

pub(crate) fn diff_entry(corpus: &Path, id: &str) -> Value {
    json(&corpus.join("rerecord-diff.json"))["probes"]
        .as_array()
        .unwrap()
        .iter()
        .find(|p| p["id"] == id)
        .unwrap()
        .clone()
}

pub(crate) fn node_script(dir: &Path, name: &str, body: &str) -> OsString {
    let path = dir.join(name);
    write(&path, &format!("#!/bin/sh\n{body}\n"));
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path.into_os_string()
}
