#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tyrus_oracle::record::run_record;

struct Run {
    code: u8,
    stdout: String,
    stderr: String,
}

fn lock_arg() -> String {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../oracle/oracle.lock")
        .display()
        .to_string()
}

fn record(items: &[&str], env: impl Fn(&str) -> Option<OsString>) -> Run {
    let args: Vec<String> = items.iter().map(|s| (*s).to_owned()).collect();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run_record(&args, &env, &mut out, &mut err).unwrap();
    Run {
        code,
        stdout: String::from_utf8(out).unwrap(),
        stderr: String::from_utf8(err).unwrap(),
    }
}

fn process_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

fn workdir(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!("record-{name}"));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write(path: &Path, text: &str) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, text).unwrap();
}

/// Corpus mínimo: fontes `.ts`, JS "emitido" à mão e um `typescript` de mentira.
fn fixture(name: &str) -> (PathBuf, PathBuf) {
    let root = workdir(name);
    let corpus = root.join("corpus");
    let emit = root.join("out/emit");
    for (set, id, ts, js) in [
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
    ] {
        write(&corpus.join(set).join(format!("{id}.ts")), ts);
        write(&emit.join(set).join(format!("{id}.js")), js);
    }
    write(
        &corpus.join("node_modules/typescript/package.json"),
        "{\"version\": \"5.1.6\"}\n",
    );
    (corpus, root.join("out"))
}

fn base_args<'a>(corpus: &'a str, out: &'a str, lock: &'a str) -> Vec<&'a str> {
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

fn json(path: &Path) -> Value {
    serde_json::from_str(&fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn missing_corpus_flag_is_a_usage_error() {
    let run = record(&["--check"], process_env);
    assert_eq!(run.code, 64);
    assert!(
        run.stderr.contains("--corpus é obrigatório"),
        "{}",
        run.stderr
    );
}

#[test]
fn wrong_reference_tuple_aborts_with_exit_2() {
    let (corpus, out) = fixture("wrong-tuple");
    let fake = corpus.parent().unwrap().join("fake-node");
    write(
        &fake,
        "#!/bin/sh\necho '{\"node\":\"24.21.0\",\"v8\":\"13.6.233.17-node.53\",\"icu\":\"78.3\",\"unicode\":\"17.0\",\"tz\":\"2026c\",\"cldr\":\"48.0\"}'\n",
    );
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let fake_path = fake.into_os_string();
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    let run = record(&base_args(&c, &o, &l), |name| {
        (name == "TYRUS_NODE_REFERENCE").then(|| fake_path.clone())
    });
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("TYRUS-E5001"), "{}", run.stderr);
    assert!(!corpus.join("golden").exists(), "nada pode ser gravado");
}

#[test]
fn record_then_check_round_trip() {
    let (corpus, out) = fixture("round-trip");
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "record: total=4 same=1 changed=1 bad_probe=2 nondeterministic=1\n"
    );
    assert!(corpus.join("golden/probes/A01_stable.json").is_file());
    assert!(!corpus.join("golden/probes/A02_flaky.json").exists());
    assert!(!corpus.join("golden/probes/A03_throws.json").exists());
    let diff = json(&corpus.join("rerecord-diff.json"));
    let reasons: Vec<&str> = diff["probes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["reason"].as_str())
        .collect();
    assert!(
        reasons.contains(&"reference_exit_nonzero(1): RangeError: boom"),
        "{reasons:?}"
    );
    let mut check_args = base_args(&c, &o, &l);
    check_args.push("--check");
    let check = record(&check_args, process_env);
    assert_eq!(check.code, 0, "{}", check.stderr);
}

#[test]
fn check_fails_when_a_golden_no_longer_matches_the_reference() {
    let (corpus, out) = fixture("stale-golden");
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    write(&out.join("emit/probes/A01_stable.js"), "console.log(9);\n");
    let mut check_args = base_args(&c, &o, &l);
    check_args.push("--check");
    let check = record(&check_args, process_env);
    assert_eq!(check.code, 1);
    assert!(
        check.stderr.contains("probes/A01_stable: golden diverge"),
        "{}",
        check.stderr
    );
}

#[test]
fn relative_paths_work_like_absolute_ones() {
    let (corpus, out) = fixture("relative");
    let cwd = std::env::current_dir().unwrap();
    let rel = |p: &Path| pathdiff(p, &cwd);
    let (c, o, l) = (rel(&corpus), rel(&out), lock_arg());
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("same=1"), "{}", run.stdout);
}

fn pathdiff(path: &Path, base: &Path) -> String {
    let up = base.components().count();
    let mut relative = PathBuf::new();
    for _ in 0..up {
        relative.push("..");
    }
    relative.push(path.strip_prefix("/").unwrap());
    relative.display().to_string()
}
