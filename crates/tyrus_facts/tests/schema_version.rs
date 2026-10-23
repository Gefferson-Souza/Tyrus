#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use serde_json::{json, Value};
use tyrus_facts::{parse_closure_lock, parse_facts, parse_partition_lock, FactsError};

fn facts_with_version(version: Value) -> Vec<u8> {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    value["schema_version"] = version;
    value.to_string().into_bytes()
}

#[test]
fn versao_desconhecida_e_recusada_com_e7102() {
    for version in [
        json!(2),
        json!(0),
        json!(999),
        json!("1"),
        json!(1.5),
        json!(null),
    ] {
        let error = parse_facts(&facts_with_version(version.clone())).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E7102", "{version}: {error}");
    }
}

#[test]
fn versao_ausente_e_recusada_com_e7102() {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    value.as_object_mut().unwrap().remove("schema_version");
    let error = parse_facts(value.to_string().as_bytes()).unwrap_err();
    assert!(
        matches!(error, FactsError::UnknownSchemaVersion { .. }),
        "{error}"
    );
}

#[test]
fn versao_futura_vence_campos_desconhecidos() {
    let bytes = br#"{"schema_version":2,"campo_do_futuro":[1,2,3]}"#;
    for result in [
        parse_facts(bytes).map(drop),
        parse_closure_lock(bytes).map(drop),
        parse_partition_lock(bytes).map(drop),
    ] {
        assert_eq!(result.unwrap_err().stable_code(), "TYRUS-E7102");
    }
}

#[test]
fn json_malformado_e_e7103() {
    let error = parse_facts(b"{ nao e json").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7103");
}

#[test]
fn raiz_que_nao_e_objeto_e_e7103() {
    assert_eq!(
        parse_facts(b"[1]").unwrap_err().stable_code(),
        "TYRUS-E7103"
    );
}
