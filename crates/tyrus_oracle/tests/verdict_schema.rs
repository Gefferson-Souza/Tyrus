#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use std::collections::BTreeSet;
use std::path::Path;

use common::{every_variant, hosted_shares};
use serde_json::{json, Value};
use tyrus_oracle::registry::{Assumption, Normalizer};
use tyrus_oracle::verdict::VerdictRecord;

fn schema_json() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../schemas/verdict.schema.json");
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn validator() -> jsonschema::Validator {
    jsonschema::validator_for(&schema_json()).unwrap()
}

fn record_json(verdict: &Value) -> Value {
    json!({
        "verdict": verdict,
        "shares": {
            "native_share": 0, "host_share": 0.93, "platform_share": 0.07, "refused_share": 0
        }
    })
}

#[test]
fn schema_accepts_the_serialization_of_every_variant() {
    let validator = validator();
    for verdict in every_variant() {
        let record = VerdictRecord {
            verdict,
            shares: hosted_shares(),
        };
        let json = serde_json::to_value(&record).unwrap();
        let errors: Vec<String> = validator
            .iter_errors(&json)
            .map(|e| e.to_string())
            .collect();
        assert!(errors.is_empty(), "{json}: {errors:?}");
    }
}

#[test]
fn schema_rejects_what_the_rust_model_rejects() {
    let validator = validator();
    let bad_verdicts = [
        json!({"kind": "green"}),
        json!({"kind": "identical", "extra": 1}),
        json!({"kind": "identical", "normalizer": null}),
        json!({"kind": "normalized"}),
        json!({"kind": "normalized", "normalizer": "N-INVENTADO"}),
        json!({"kind": "refused", "code": "minuscula"}),
        json!({"kind": "identical_under_assumption", "assumptions": []}),
        json!({"kind": "identical_under_assumption", "assumptions": ["A-INVENTADA"]}),
        json!({"kind": "bad_probe", "reason": ""}),
        json!({"kind": "bad_probe", "reason": "duas\nlinhas"}),
    ];
    for verdict in bad_verdicts {
        let record = record_json(&verdict);
        assert!(!validator.is_valid(&record), "o esquema aceitou {verdict}");
        assert!(
            serde_json::from_value::<VerdictRecord>(record).is_err(),
            "o Rust aceitou {verdict}"
        );
    }
}

#[test]
fn schema_requires_all_four_shares_in_range() {
    let validator = validator();
    let good = record_json(&json!({"kind": "identical"}));
    assert!(validator.is_valid(&good));
    for field in [
        "native_share",
        "host_share",
        "platform_share",
        "refused_share",
    ] {
        let mut missing = good.clone();
        missing
            .pointer_mut("/shares")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .remove(field);
        assert!(!validator.is_valid(&missing), "{field} ausente");
        let mut high = good.clone();
        high.pointer_mut("/shares")
            .unwrap()
            .as_object_mut()
            .unwrap()
            .insert(field.into(), json!(1.5));
        assert!(!validator.is_valid(&high), "{field} = 1.5");
    }
}

fn enum_at(pointer: &str) -> BTreeSet<String> {
    let schema = schema_json();
    let values = schema.pointer(pointer).unwrap().as_array().unwrap();
    values
        .iter()
        .map(|v| v.as_str().unwrap().to_owned())
        .collect()
}

#[test]
fn schema_registries_match_the_rust_registries() {
    let normalizers: BTreeSet<String> = Normalizer::ALL.iter().map(|n| n.id().to_owned()).collect();
    assert_eq!(enum_at("/$defs/normalizerId/enum"), normalizers);
    let assumptions: BTreeSet<String> = Assumption::ALL.iter().map(|a| a.id().to_owned()).collect();
    assert_eq!(enum_at("/$defs/assumptionId/enum"), assumptions);
}

#[test]
fn schema_kinds_match_the_serde_kinds() {
    let schema = schema_json();
    let mut schema_kinds = enum_at("/$defs/verdict/oneOf/0/properties/kind/enum");
    for index in 1..5 {
        let pointer = format!("/$defs/verdict/oneOf/{index}/properties/kind/const");
        schema_kinds.insert(
            schema
                .pointer(&pointer)
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned(),
        );
    }
    let serde_kinds: BTreeSet<String> = every_variant()
        .iter()
        .map(|v| {
            serde_json::to_value(v)
                .unwrap()
                .get("kind")
                .unwrap()
                .as_str()
                .unwrap()
                .to_owned()
        })
        .collect();
    assert_eq!(schema_kinds, serde_kinds);
}
