#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

use std::ffi::OsString;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};

use serde_json::Value;
use tyrus_oracle::cli::{run, EXIT_ORACLE_ERROR, EXIT_USAGE};

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("cli_run")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn node_target(dir: &Path) -> PathBuf {
    let node = std::env::var("TYRUS_NODE_REFERENCE").unwrap();
    let path = dir.join("target.sh");
    fs::write(
        &path,
        format!("#!/bin/sh\nexec {node} --experimental-strip-types \"$1\"\n"),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

struct Run {
    code: u8,
    stdout: String,
    stderr: String,
}

fn run_cli(items: &[String], env: impl Fn(&str) -> Option<OsString>) -> Run {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(items, &env, &mut out, &mut err).unwrap();
    Run {
        code,
        stdout: String::from_utf8(out).unwrap(),
        stderr: String::from_utf8(err).unwrap(),
    }
}

fn base_args(name: &str, out: &Path) -> Vec<String> {
    let dir = scratch(name);
    let root = root();
    [
        "run",
        "--corpus",
        "seed",
        "--reference",
        "node",
        "--hosted-cmd",
        &node_target(&dir).display().to_string(),
        "--lock",
        &root.join("oracle/oracle.lock").display().to_string(),
        "--probes-dir",
        &root.join("oracle/corpus/seed/probes").display().to_string(),
        "--shim",
        &root.join("oracle/shim/fix-env.js").display().to_string(),
        "--filter",
        "C01_02",
        "--out",
        &out.display().to_string(),
    ]
    .iter()
    .map(|s| (*s).to_owned())
    .collect()
}

fn process_env(name: &str) -> Option<OsString> {
    std::env::var_os(name)
}

#[test]
fn run_against_node_writes_a_schema_valid_record_and_exits_zero() {
    let out = scratch("ok-out");
    let result = run_cli(&base_args("ok", &out), process_env);
    assert_eq!(result.code, 0, "{}", result.stderr);
    assert_eq!(result.stdout, "run: 1 sondas, 1 verdes, 0 não verdes\n");
    let text = fs::read_to_string(out.join("C01_02_negzero/equivalence.json")).unwrap();
    let record: Value = serde_json::from_str(&text).unwrap();
    let schema_path = root().join("schemas/equivalence.schema.json");
    let schema: Value = serde_json::from_str(&fs::read_to_string(schema_path).unwrap()).unwrap();
    let validator = jsonschema::validator_for(&schema).unwrap();
    let errors: Vec<String> = validator
        .iter_errors(&record)
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors, Vec::<String>::new(), "{text}");
    let verdict_schema = root().join("schemas/verdict.schema.json");
    let verdict_schema: Value =
        serde_json::from_str(&fs::read_to_string(verdict_schema).unwrap()).unwrap();
    let verdict_validator = jsonschema::validator_for(&verdict_schema).unwrap();
    assert!(verdict_validator.is_valid(&record["result"]), "{text}");
    assert_eq!(record["profile"], "exact");
}

#[test]
fn run_refuses_a_reference_whose_tuple_differs_from_the_lock() {
    let out = scratch("tuple-out");
    let seed_as_reference = |name: &str| {
        let wanted = if name == "TYRUS_NODE_REFERENCE" {
            "TYRUS_NODE_SEED"
        } else {
            name
        };
        std::env::var_os(wanted)
    };
    let result = run_cli(&base_args("tuple", &out), seed_as_reference);
    assert_eq!(result.code, EXIT_ORACLE_ERROR);
    assert!(
        result.stderr.starts_with("TYRUS-E5001: "),
        "{}",
        result.stderr
    );
    assert!(!out.join("C01_02_negzero").exists());
}

#[test]
fn seed_tuple_role_accepts_the_seed_binary() {
    let out = scratch("seed-out");
    let mut args = base_args("seed", &out);
    args.extend([
        "--tuple-role".to_owned(),
        "seed".to_owned(),
        "--profile".to_owned(),
        "legacy_trim".to_owned(),
    ]);
    let seed_as_reference = |name: &str| {
        let wanted = if name == "TYRUS_NODE_REFERENCE" {
            "TYRUS_NODE_SEED"
        } else {
            name
        };
        std::env::var_os(wanted)
    };
    let result = run_cli(&args, seed_as_reference);
    assert_eq!(result.code, 0, "{}", result.stderr);
    let text = fs::read_to_string(out.join("C01_02_negzero/equivalence.json")).unwrap();
    assert!(text.contains("\"tuple_role\": \"seed\""), "{text}");
}

#[test]
fn run_without_required_options_prints_usage() {
    let result = run_cli(
        &["run".to_owned(), "--corpus".to_owned(), "seed".to_owned()],
        process_env,
    );
    assert_eq!(result.code, EXIT_USAGE);
    assert!(
        result.stderr.contains("--reference é obrigatório"),
        "{}",
        result.stderr
    );
    assert_eq!(result.stdout, "");
}

#[test]
fn run_rejects_unknown_corpus_profile_and_reference() {
    let out = scratch("bad-out");
    for (flag, value, needle) in [
        ("--corpus", "outro", "desconhecido"),
        ("--profile", "solto", "perfil desconhecido"),
        ("--reference", "bun", "não suportado"),
    ] {
        let mut args = base_args("bad", &out);
        args.extend([flag.to_owned(), value.to_owned()]);
        let result = run_cli(&args, process_env);
        assert_eq!(result.code, EXIT_USAGE, "{flag}");
        assert!(result.stderr.contains(needle), "{flag}: {}", result.stderr);
    }
}

#[test]
fn run_with_a_promoted_cmd_records_the_third_leg() {
    let out = scratch("promoted-out");
    let mut args = base_args("promoted", &out);
    let promoted_dir = scratch("promoted-leg");
    let promoted = node_target(&promoted_dir);
    args.extend(["--promoted-cmd".to_owned(), promoted.display().to_string()]);
    let result = run_cli(&args, process_env);
    assert_eq!(result.code, 0, "{}", result.stderr);
    let text = fs::read_to_string(out.join("C01_02_negzero/equivalence.json")).unwrap();
    let record: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(record["promoted"]["verdict"]["kind"], "identical", "{text}");
    assert!(record["promoted_target"]["command"]
        .as_str()
        .unwrap()
        .ends_with("target.sh"));
    let schema_path = root().join("schemas/equivalence.schema.json");
    let schema: Value = serde_json::from_str(&fs::read_to_string(schema_path).unwrap()).unwrap();
    assert!(
        jsonschema::validator_for(&schema)
            .unwrap()
            .is_valid(&record),
        "{text}"
    );
}

#[test]
fn run_with_a_mutating_promoted_cmd_is_not_green() {
    let out = scratch("promoted-bad-out");
    let mut args = base_args("promoted-bad", &out);
    let dir = scratch("promoted-bad-leg");
    let node = std::env::var("TYRUS_NODE_REFERENCE").unwrap();
    let path = dir.join("mut.sh");
    fs::write(
        &path,
        format!("#!/bin/sh\n{node} --experimental-strip-types \"$1\" | sed '1s/./Z/'\n"),
    )
    .unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    args.extend(["--promoted-cmd".to_owned(), path.display().to_string()]);
    let result = run_cli(&args, process_env);
    let text = fs::read_to_string(out.join("C01_02_negzero/equivalence.json")).unwrap();
    let record: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(record["promoted"]["verdict"]["kind"], "different", "{text}");
    assert_eq!(record["result"]["verdict"]["kind"], "identical", "{text}");
    assert_eq!(result.code, 1, "{}", result.stderr);
    assert!(
        result
            .stdout
            .starts_with("run: 1 sondas, 0 verdes, 1 não verdes\n"),
        "{}",
        result.stdout
    );
}
