#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use serde_json::Value;
use tyrus_facts::{canonical_bytes, parse_facts, Facts, FactsError};

#[test]
fn golden_da_replica_faz_ida_e_volta_sem_perda() {
    let facts = parse_facts(support::REPLICA_FACTS.as_bytes()).expect("golden válido");
    assert_eq!(facts.typescript.version, "5.1.6");
    assert_eq!(facts.sources.len(), 27);
    let original: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    let reserialized = serde_json::to_value(&facts).unwrap();
    assert_eq!(reserialized, original);
}

#[test]
fn golden_canonico_e_ponto_fixo() {
    let facts = parse_facts(support::REPLICA_FACTS.as_bytes()).unwrap();
    let canonical = canonical_bytes(&facts).unwrap();
    let again = parse_facts(&canonical).unwrap();
    assert_eq!(canonical_bytes(&again).unwrap(), canonical);
    assert_eq!(canonical.last(), Some(&b'\n'));
}

#[test]
fn golden_canonico_versionado_bate_com_o_arquivo() {
    let facts = parse_facts(support::REPLICA_FACTS.as_bytes()).unwrap();
    let golden = include_str!("golden/replica.facts.canonical.json");
    assert_eq!(
        String::from_utf8(canonical_bytes(&facts).unwrap()).unwrap(),
        golden
    );
}

#[test]
fn campo_desconhecido_e_documento_invalido() {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    value["extra"] = Value::Bool(true);
    let error = parse_facts(value.to_string().as_bytes()).unwrap_err();
    assert!(
        matches!(error, FactsError::InvalidDocument { .. }),
        "{error}"
    );
    assert_eq!(error.stable_code(), "TYRUS-E7103");
}

#[test]
fn sha256_malformado_e_recusado() {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    value["tsconfig"]["sha256"] = Value::String("ABC".into());
    assert!(parse_facts(value.to_string().as_bytes()).is_err());
}

#[test]
fn caminho_que_escapa_da_raiz_e_recusado() {
    for bad in ["../x.ts", "/abs.ts", "a/../b.ts", "", "a\\b.ts"] {
        let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
        value["sources"][0]["path"] = Value::String(bad.into());
        assert!(
            parse_facts(value.to_string().as_bytes()).is_err(),
            "{bad:?}"
        );
    }
}

#[test]
fn caminho_duplicado_e_recusado() {
    let mut value: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    let first = value["sources"][0].clone();
    value["sources"].as_array_mut().unwrap().push(first);
    assert!(parse_facts(value.to_string().as_bytes()).is_err());
}

#[test]
fn tipo_exportado_e_o_do_esquema() {
    let _: Option<Facts> = None;
}
