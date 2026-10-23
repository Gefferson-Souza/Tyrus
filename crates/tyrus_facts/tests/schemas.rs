#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use std::path::PathBuf;

use serde_json::Value;
use tyrus_facts::{parse_closure_lock, parse_facts, parse_partition_lock};

fn schema(name: &str) -> jsonschema::Validator {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../schemas")
        .join(name);
    let text = std::fs::read_to_string(path).expect("esquema existe");
    jsonschema::validator_for(&serde_json::from_str::<Value>(&text).unwrap())
        .expect("esquema compila")
}

#[test]
fn golden_de_facts_valida_no_esquema_unico() {
    let value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    assert!(schema("facts.schema.json").is_valid(&value));
}

#[test]
fn saida_canonica_dos_tipos_valida_no_esquema() {
    let facts = parse_facts(support::REPLICA_FACTS.as_bytes()).unwrap();
    let value = serde_json::to_value(&facts).unwrap();
    assert!(schema("facts.schema.json").is_valid(&value));
    let closure = parse_closure_lock(include_bytes!("golden/closure.lock")).unwrap();
    assert!(schema("closure.lock.schema.json").is_valid(&serde_json::to_value(&closure).unwrap()));
    let partition = parse_partition_lock(include_bytes!("golden/partition.lock")).unwrap();
    assert!(
        schema("partition.lock.schema.json").is_valid(&serde_json::to_value(&partition).unwrap())
    );
}

#[test]
fn esquema_e_serde_concordam_na_rejeicao() {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    value["extra"] = true.into();
    assert!(!schema("facts.schema.json").is_valid(&value));
    assert!(parse_facts(value.to_string().as_bytes()).is_err());
    let mut closure: Value = serde_json::from_str(include_str!("golden/closure.lock")).unwrap();
    closure["files"][0]["mode"] = "0600".into();
    assert!(!schema("closure.lock.schema.json").is_valid(&closure));
    assert!(parse_closure_lock(closure.to_string().as_bytes()).is_err());
    let mut partition: Value = serde_json::from_str(include_str!("golden/partition.lock")).unwrap();
    partition["units"][0]["placement"] = "other".into();
    assert!(!schema("partition.lock.schema.json").is_valid(&partition));
    assert!(parse_partition_lock(partition.to_string().as_bytes()).is_err());
}
