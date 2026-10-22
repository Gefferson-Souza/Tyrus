#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use tyrus_oracle::lock::{OracleLock, LOCK_SCHEMA_VERSION};
use tyrus_oracle::tuple::NodeRole;

fn repo_file(relative: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(relative)
}

fn read(relative: &str) -> String {
    std::fs::read_to_string(repo_file(relative)).unwrap()
}

fn lock_schema() -> jsonschema::Validator {
    let schema: Value = serde_json::from_str(&read("schemas/oracle.lock.schema.json")).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

fn valid_lock_json() -> Value {
    serde_json::from_str(&read("oracle/oracle.lock")).unwrap()
}

#[test]
fn committed_lock_pins_the_reference_and_seed_tuples() {
    let lock = OracleLock::load(&repo_file("oracle/oracle.lock")).unwrap();
    assert_eq!(lock.schema_version, LOCK_SCHEMA_VERSION);
    let reference = lock.tuple(NodeRole::Reference);
    assert_eq!(
        (
            &*reference.node,
            &*reference.v8,
            &*reference.icu,
            &*reference.unicode,
            &*reference.tz,
            &*reference.cldr
        ),
        (
            "26.11.1",
            "14.6.202.34-node.37",
            "78.3",
            "17.0",
            "2026e",
            "48.0"
        )
    );
    let seed = lock.tuple(NodeRole::Seed);
    assert_eq!(
        (
            &*seed.node,
            &*seed.v8,
            &*seed.icu,
            &*seed.unicode,
            &*seed.tz,
            &*seed.cldr
        ),
        (
            "22.21.1",
            "12.4.254.21-node.33",
            "77.1",
            "16.0",
            "2025b",
            "47.0"
        )
    );
    assert_eq!(lock.host_crates, std::collections::BTreeMap::new());
}

#[test]
fn committed_lock_is_canonical_json() {
    let text = read("oracle/oracle.lock");
    let lock = OracleLock::parse(&text).unwrap();
    assert_eq!(
        format!("{}\n", serde_json::to_string_pretty(&lock).unwrap()),
        text
    );
}

#[test]
fn lock_schema_accepts_the_committed_lock() {
    let schema = lock_schema();
    let errors: Vec<String> = schema
        .iter_errors(&valid_lock_json())
        .map(|e| e.to_string())
        .collect();
    assert!(errors.is_empty(), "{errors:?}");
}

fn lock_with(pointer: &str, value: Value) -> Value {
    let (parent, key) = pointer.rsplit_once('/').unwrap();
    let mut lock = valid_lock_json();
    let target = if parent.is_empty() {
        &mut lock
    } else {
        lock.pointer_mut(parent).unwrap()
    };
    target
        .as_object_mut()
        .unwrap()
        .insert(key.to_owned(), value);
    lock
}

#[test]
fn lock_schema_rejects_malformed_locks() {
    let schema = lock_schema();
    let mutations: [(&str, Value); 7] = [
        ("/schema_version", json!(2)),
        ("/reference/node", json!("v26.11.1")),
        ("/reference/v8", json!("14.6.202.34")),
        ("/seed/tz", json!("2025")),
        ("/seed/cldr", json!("47")),
        ("/host_crates/v8", json!("")),
        ("/extra", json!(true)),
    ];
    for (pointer, value) in mutations {
        let lock = lock_with(pointer, value);
        assert!(
            !schema.is_valid(&lock),
            "{pointer} deveria ser rejeitado pelo esquema"
        );
    }
}

fn without_cldr() -> Value {
    let mut lock = valid_lock_json();
    lock.pointer_mut("/reference")
        .unwrap()
        .as_object_mut()
        .unwrap()
        .remove("cldr");
    lock
}

#[test]
fn lock_schema_requires_cldr() {
    assert!(!lock_schema().is_valid(&without_cldr()));
}

#[test]
fn parse_rejects_what_the_typed_model_can_detect() {
    let mut missing = valid_lock_json();
    missing.as_object_mut().unwrap().remove("seed");
    let rejected = [
        missing,
        lock_with("/reference/tz", json!("  ")),
        lock_with("/schema_version", json!(2)),
        lock_with("/reference/extra", json!("x")),
        without_cldr(),
    ];
    for bad in rejected {
        let error = OracleLock::parse(&bad.to_string()).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E5005", "{bad}");
    }
    assert_eq!(
        OracleLock::parse("não é json").unwrap_err().stable_code(),
        "TYRUS-E5005"
    );
}

#[test]
fn load_reports_an_unreadable_path_as_lock_invalid() {
    let error = OracleLock::load(Path::new("/nao/existe/oracle.lock")).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5005");
    assert!(error.to_string().contains("/nao/existe/oracle.lock"));
}
