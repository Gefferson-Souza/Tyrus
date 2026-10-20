#![allow(clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::shares::{Share, Shares};

#[test]
fn share_accepts_the_closed_unit_interval() {
    for value in [0.0, 0.25, 1.0] {
        assert!((Share::new(value).unwrap().get() - value).abs() < f64::EPSILON);
    }
}

#[test]
fn share_rejects_out_of_range_and_non_finite_values() {
    for value in [
        -0.000_001,
        1.000_001,
        f64::NAN,
        f64::INFINITY,
        f64::NEG_INFINITY,
    ] {
        let error = Share::new(value).unwrap_err();
        assert_eq!(error.stable_code(), "TYRUS-E5008", "{value}");
    }
}

#[test]
fn hosted_baseline_means_native_share_is_zero() {
    let hosted = Shares::new(0.0, 0.93, 0.07, 0.0).unwrap();
    let promoted = Shares::new(0.01, 0.92, 0.07, 0.0).unwrap();
    assert!(hosted.is_hosted_baseline());
    assert!(!promoted.is_hosted_baseline());
}

#[test]
fn compiles_everything_claim_needs_host_share_zero() {
    let hosted = Shares::new(0.0, 0.93, 0.07, 0.0).unwrap();
    let all_native = Shares::new(0.9, 0.0, 0.1, 0.0).unwrap();
    assert!(!hosted.permits_compiles_everything_claim());
    assert!(all_native.permits_compiles_everything_claim());
}

#[test]
fn shares_json_requires_all_four_fields_and_rejects_extras() {
    let complete = r#"{"native_share":0,"host_share":1,"platform_share":0,"refused_share":0}"#;
    assert!(serde_json::from_str::<Shares>(complete).is_ok());
    let missing_native = r#"{"host_share":1,"platform_share":0,"refused_share":0}"#;
    assert!(serde_json::from_str::<Shares>(missing_native).is_err());
    let extra = r#"{"native_share":0,"host_share":1,"platform_share":0,"refused_share":0,"x":1}"#;
    assert!(serde_json::from_str::<Shares>(extra).is_err());
    let out_of_range = r#"{"native_share":2,"host_share":1,"platform_share":0,"refused_share":0}"#;
    assert!(serde_json::from_str::<Shares>(out_of_range).is_err());
}
