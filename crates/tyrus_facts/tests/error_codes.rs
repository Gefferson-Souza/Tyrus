#![allow(clippy::expect_used, clippy::unwrap_used)]

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tyrus_facts::{sha256_hex, FactsError};

fn one_of_each_variant() -> Vec<FactsError> {
    vec![
        FactsError::HashMismatch {
            path: "src/a.ts".into(),
            expected: sha256_hex(b"a"),
            actual: None,
        },
        FactsError::UnknownSchemaVersion {
            found: "9".into(),
            supported: 1,
        },
        FactsError::InvalidDocument {
            reason: "campo".into(),
        },
        FactsError::Io {
            path: PathBuf::from("/x"),
            reason: "falhou".into(),
        },
    ]
}

#[test]
fn codes_are_unique_and_in_the_facts_block() {
    let errors = one_of_each_variant();
    let codes: BTreeSet<&str> = errors.iter().map(FactsError::stable_code).collect();
    assert_eq!(codes.len(), errors.len(), "código repetido entre variantes");
    for code in &codes {
        let digits = code.strip_prefix("TYRUS-E71").expect("bloco E71xx");
        assert!(
            digits.len() == 2 && digits.bytes().all(|b| b.is_ascii_digit()),
            "{code}"
        );
    }
}

#[test]
fn every_variant_has_exactly_one_code_literal_in_the_source() {
    let source = include_str!("../src/error.rs");
    let literals = source.matches("\"TYRUS-E71").count();
    assert_eq!(literals, one_of_each_variant().len());
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
fn no_facts_code_is_reused_by_another_crate() {
    let workspace_crates = Path::new(env!("CARGO_MANIFEST_DIR")).join("..");
    let mut other_owners: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut facts_codes = BTreeSet::new();
    for entry in std::fs::read_dir(&workspace_crates).unwrap() {
        let member_dir = entry.unwrap().path();
        let name = member_dir
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        if name == "tyrus_facts" {
            facts_codes = codes_declared_by(&member_dir);
            continue;
        }
        for code in codes_declared_by(&member_dir) {
            other_owners.entry(code).or_default().insert(name.clone());
        }
    }
    assert!(
        facts_codes.contains("TYRUS-E7101"),
        "a varredura não enxergou o próprio crate"
    );
    let reused: Vec<_> = facts_codes
        .iter()
        .filter(|c| other_owners.contains_key(*c))
        .collect();
    assert_eq!(
        reused,
        Vec::<&String>::new(),
        "código dos fatos reaproveitado por outro crate (R14)"
    );
}
