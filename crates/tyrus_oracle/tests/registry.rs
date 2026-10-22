#![allow(clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::registry::{Assumption, Assumptions, Normalizer};

#[test]
fn normalizer_registry_is_closed() {
    let ids: Vec<&str> = Normalizer::ALL.iter().map(|n| n.id()).collect();
    assert_eq!(ids, ["N-STACK"]);
    assert_eq!(Normalizer::from_id("N-STACK").unwrap(), Normalizer::Stack);
    for unknown in ["", "n-stack", "N-STACK ", "N-OTHER", "STACK"] {
        let error = Normalizer::from_id(unknown).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E5006", "{unknown:?}");
    }
}

#[test]
fn every_normalizer_id_round_trips_through_json() {
    for normalizer in Normalizer::ALL {
        let json = serde_json::to_string(&normalizer).unwrap();
        assert_eq!(json, format!("\"{}\"", normalizer.id()));
        assert_eq!(
            serde_json::from_str::<Normalizer>(&json).unwrap(),
            normalizer
        );
    }
    assert!(serde_json::from_str::<Normalizer>("\"N-INVENTADO\"").is_err());
}

#[test]
fn assumption_registry_is_closed() {
    let ids: Vec<&str> = Assumption::ALL.iter().map(|a| a.id()).collect();
    assert_eq!(ids, ["A-NATIVE-TOSTRING"]);
    let error = Assumption::from_id("A-INVENTADO").unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5007");
}

#[test]
fn assumptions_must_be_non_empty_and_are_canonicalised() {
    let empty = Assumptions::try_from(Vec::new()).unwrap_err();
    assert_eq!(empty.stable_code(), "TYRUS-E5009");
    let doubled =
        Assumptions::try_from(vec![Assumption::NativeToString, Assumption::NativeToString])
            .unwrap();
    assert_eq!(doubled.as_slice(), [Assumption::NativeToString]);
    assert!(serde_json::from_str::<Assumptions>("[]").is_err());
    assert!(serde_json::from_str::<Assumptions>("[\"A-NATIVE-TOSTRING\"]").is_ok());
}
