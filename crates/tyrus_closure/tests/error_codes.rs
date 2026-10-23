#![allow(clippy::expect_used, clippy::unwrap_used)]
use std::collections::BTreeSet;
use std::path::PathBuf;

use tyrus_closure::ClosureError;
use tyrus_facts::FactsError;

#[test]
fn codes_are_unique_stable_and_in_the_closure_block() {
    let errors = [
        ClosureError::DevDependency { paths: vec![] },
        ClosureError::TraceMissing { paths: vec![] },
        ClosureError::InvalidTrace {
            line: 1,
            reason: String::new(),
        },
        ClosureError::Npm {
            status: String::new(),
            stderr: String::new(),
        },
        ClosureError::Io {
            path: PathBuf::from("/x"),
            reason: String::new(),
        },
    ];
    let codes: BTreeSet<&str> = errors.iter().map(ClosureError::stable_code).collect();
    assert_eq!(codes.len(), errors.len());
    for code in &codes {
        let digits = code.strip_prefix("TYRUS-E72").expect("bloco E72xx");
        assert!(
            digits.len() == 2 && digits.bytes().all(|b| b.is_ascii_digit()),
            "{code}"
        );
    }
}

#[test]
fn facts_errors_keep_their_own_code() {
    let inner = FactsError::InvalidDocument { reason: "x".into() };
    assert_eq!(ClosureError::from(inner).stable_code(), "TYRUS-E7103");
}
