#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

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
        OracleError::CommandInvalid {
            reason: "vazio".into(),
        },
        OracleError::CorpusUnavailable {
            path: PathBuf::from("/x"),
            reason: "ausente".into(),
        },
        OracleError::SpawnFailed {
            program: "x".into(),
            reason: "ausente".into(),
        },
        OracleError::OutputWriteFailed {
            path: PathBuf::from("/x"),
            reason: "negado".into(),
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

fn collect_rust_files(dir: &Path, files: &mut Vec<PathBuf>) {
    for entry in std::fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, files);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            files.push(path);
        }
    }
}

fn error_codes_in(text: &str) -> BTreeSet<String> {
    let marker = "TYRUS-E";
    text.match_indices(marker)
        .filter_map(|(start, _)| {
            let digits = text.get(start + marker.len()..start + marker.len() + 4)?;
            digits
                .bytes()
                .all(|b| b.is_ascii_digit())
                .then(|| format!("{marker}{digits}"))
        })
        .collect()
}

fn codes_declared_by(member_dir: &Path) -> BTreeSet<String> {
    let mut files = Vec::new();
    if member_dir.join("src").is_dir() {
        collect_rust_files(&member_dir.join("src"), &mut files);
    }
    files
        .iter()
        .flat_map(|file| error_codes_in(&std::fs::read_to_string(file).unwrap()))
        .collect()
}

#[test]
fn no_oracle_code_is_reused_by_another_crate() {
    let workspace_crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut other_owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut oracle_codes = BTreeSet::new();
    for entry in std::fs::read_dir(&workspace_crates).unwrap() {
        let member_dir = entry.unwrap().path();
        let name = member_dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if name == "tyrus_oracle" {
            oracle_codes = codes_declared_by(&member_dir);
            continue;
        }
        for code in codes_declared_by(&member_dir) {
            other_owners.entry(code).or_default().insert(name.clone());
        }
    }
    assert!(
        oracle_codes.contains("TYRUS-E5001"),
        "a varredura não enxergou o próprio crate"
    );
    let reused: Vec<_> = oracle_codes
        .iter()
        .filter(|c| other_owners.contains_key(*c))
        .collect();
    assert!(
        reused.is_empty(),
        "código do oráculo reaproveitado por outro crate (R14): {reused:?}"
    );
}
