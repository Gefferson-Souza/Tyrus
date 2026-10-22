#![allow(clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::codes::{BadProbeReason, RefusalCode};

#[test]
fn refusal_code_accepts_plain_and_detailed_forms() {
    let plain = RefusalCode::try_from("ADDON_NAPI_UNSUPPORTED".to_owned()).unwrap();
    assert_eq!(
        (plain.code(), plain.detail()),
        ("ADDON_NAPI_UNSUPPORTED", None)
    );
    let detailed = RefusalCode::try_from("UNBOUND_PLATFORM(fs.cpSync)".to_owned()).unwrap();
    assert_eq!(
        (detailed.code(), detailed.detail()),
        ("UNBOUND_PLATFORM", Some("fs.cpSync"))
    );
}

#[test]
fn refusal_code_round_trips_through_json_as_one_string() {
    let code = RefusalCode::new("UNBOUND_PLATFORM", Some("fs.cpSync")).unwrap();
    let json = serde_json::to_string(&code).unwrap();
    assert_eq!(json, "\"UNBOUND_PLATFORM(fs.cpSync)\"");
    assert_eq!(serde_json::from_str::<RefusalCode>(&json).unwrap(), code);
}

#[test]
fn refusal_code_rejects_malformed_values() {
    let malformed = [
        "",
        "ab",
        "AB",
        "lower_case",
        "1STARTS_WITH_DIGIT",
        "HAS SPACE",
        "CODE()",
        "CODE(unclosed",
        "CODE(a b)",
        "CODE(x)trailing",
        "CODE(x(y))",
    ];
    for value in malformed {
        let error = RefusalCode::try_from(value.to_owned()).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E5009", "{value:?}");
    }
    let too_long = "A".repeat(65);
    assert!(RefusalCode::try_from(too_long).is_err());
    assert!(RefusalCode::try_from("ABC".to_owned()).is_ok());
    assert!(RefusalCode::try_from("A".repeat(64)).is_ok());
}

#[test]
fn bad_probe_reason_is_one_to_two_hundred_printable_characters() {
    assert!(BadProbeReason::try_from(
        "strip-types imprime 7; o emit CJS dá SyntaxError".to_owned()
    )
    .is_ok());
    for invalid in [String::new(), "x".repeat(201), "linha\nquebrada".to_owned()] {
        let error = BadProbeReason::try_from(invalid).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E5009");
    }
    let at_limit = BadProbeReason::try_from("x".repeat(200)).unwrap();
    assert_eq!(at_limit.as_str().len(), 200);
    let reason = BadProbeReason::try_from("motivo".to_owned()).unwrap();
    assert_eq!(reason.as_str(), "motivo");
    assert_eq!(serde_json::to_string(&reason).unwrap(), "\"motivo\"");
}
