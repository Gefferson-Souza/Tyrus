#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::BTreeSet;
use std::path::PathBuf;

use tyrus_oracle::error::OracleError;
use tyrus_oracle::tuple::{FieldDiff, NodeRole};

fn one_of_each_variant() -> Vec<OracleError> {
    vec![
        OracleError::TupleMismatch {
            role: NodeRole::Reference,
            diffs: vec![FieldDiff {
                field: "tz",
                expected: "2026e".into(),
                actual: "2025b".into(),
            }],
        },
        OracleError::NodeBinaryNotConfigured {
            env_var: "TYRUS_NODE_REFERENCE",
        },
        OracleError::NodeProbeFailed {
            path: PathBuf::from("/x/node"),
            reason: "falhou".into(),
        },
        OracleError::NodeProbeOutputInvalid {
            reason: "json".into(),
        },
        OracleError::LockInvalid {
            reason: "campo".into(),
        },
        OracleError::UnknownNormalizer { id: "N-X".into() },
        OracleError::UnknownAssumption { id: "A-X".into() },
        OracleError::InvalidShare {
            value: "1.5".into(),
        },
        OracleError::InvalidVerdictField {
            field: "code",
            reason: "vazio".into(),
        },
    ]
}

#[test]
fn codes_are_unique_and_in_the_oracle_block() {
    let errors = one_of_each_variant();
    let codes: BTreeSet<&str> = errors.iter().map(OracleError::stable_code).collect();
    assert_eq!(codes.len(), errors.len(), "código repetido entre variantes");
    for code in &codes {
        let digits = code.strip_prefix("TYRUS-E5").expect("bloco E5xxx");
        assert!(
            digits.len() == 3 && digits.bytes().all(|b| b.is_ascii_digit()),
            "{code}"
        );
    }
}

#[test]
fn every_variant_has_exactly_one_code_literal_in_the_source() {
    let source = include_str!("../src/error.rs");
    let literals = source.matches("\"TYRUS-E5").count();
    assert_eq!(literals, one_of_each_variant().len());
}

#[test]
fn tuple_mismatch_is_e5001_and_names_the_field() {
    let error = one_of_each_variant().remove(0);
    assert_eq!(error.stable_code(), "TYRUS-E5001");
    let message = error.to_string();
    assert!(message.contains("reference"), "{message}");
    assert!(
        message.contains("tz: esperado `2026e`, obtido `2025b`"),
        "{message}"
    );
}
