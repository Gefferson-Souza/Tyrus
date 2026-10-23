#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use std::path::Path;

use common::{replica_tree, sample_files, write_archive};
use serde_json::Value;
use tyrus_archive::cli::run;

fn validator() -> jsonschema::Validator {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/closure-archive.schema.json");
    let schema: Value = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

fn inspect(bytes: &[u8], name: &str) -> (u8, String, String) {
    let dir = std::env::temp_dir().join(format!("tyrus-archive-{name}-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("closure.tyar");
    std::fs::write(&file, bytes).unwrap();
    let args = vec![
        "inspect".to_owned(),
        "--json".to_owned(),
        file.to_string_lossy().into_owned(),
    ];
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(&args, &mut out, &mut err).unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    (
        code,
        String::from_utf8(out).unwrap(),
        String::from_utf8(err).unwrap(),
    )
}

#[test]
fn inspect_json_validates_against_the_schema() {
    let validator = validator();
    for (name, files) in [("sample", sample_files()), ("replica", replica_tree())] {
        let (code, stdout, stderr) = inspect(&write_archive(&files), name);
        assert_eq!((code, stderr.as_str()), (0, ""));
        let value: Value = serde_json::from_str(&stdout).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(&value)
            .map(|e| e.to_string())
            .collect();
        assert_eq!(errors, Vec::<String>::new());
        assert_eq!(value["entry_count"].as_u64().unwrap(), files.len() as u64);
    }
}

#[test]
fn schema_rejects_a_report_with_unknown_fields_or_bad_hash() {
    let validator = validator();
    let (_, stdout, _) = inspect(&write_archive(&sample_files()), "negative");
    let mut value: Value = serde_json::from_str(&stdout).unwrap();
    value["set_sha256"] = Value::String("xyz".into());
    assert!(!validator.is_valid(&value));
    let mut extra: Value = serde_json::from_str(&stdout).unwrap();
    extra["surprise"] = Value::Bool(true);
    assert!(!validator.is_valid(&extra));
}

#[test]
fn inspect_refuses_a_corrupted_archive_with_the_stable_code() {
    let mut bytes = write_archive(&sample_files());
    bytes[20] ^= 0x01;
    let (code, stdout, stderr) = inspect(&bytes, "corrupt");
    assert_eq!(code, 2);
    assert_eq!(stdout, "");
    assert!(stderr.contains("TYRUS-E81"), "{stderr}");
}

#[test]
fn usage_errors_exit_64() {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    assert_eq!(run(&[], &mut out, &mut err).unwrap(), 64);
    assert_eq!(
        run(&["inspect".to_owned()], &mut out, &mut err).unwrap(),
        64
    );
}
