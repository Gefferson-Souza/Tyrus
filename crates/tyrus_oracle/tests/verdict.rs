#![allow(clippy::expect_used, clippy::unwrap_used)]

mod common;

use std::collections::BTreeSet;

use common::{every_variant, hosted_shares, variant_index, VARIANT_COUNT};
use tyrus_oracle::codes::RefusalCode;
use tyrus_oracle::shares::Shares;
use tyrus_oracle::verdict::{ClaimBasis, Verdict, VerdictClass, VerdictRecord};

#[test]
fn samples_cover_every_variant_exactly_once() {
    let indexes: BTreeSet<usize> = every_variant().iter().map(variant_index).collect();
    assert_eq!(indexes, (0..VARIANT_COUNT).collect());
    assert_eq!(every_variant().len(), VARIANT_COUNT);
}

#[test]
fn verdict_green_set_is_exactly_identical_and_normalized() {
    let green: Vec<usize> = every_variant()
        .iter()
        .filter(|verdict| verdict.is_green())
        .map(variant_index)
        .collect();
    assert_eq!(green, [0, 1]);
}

#[test]
fn refused_is_never_green_in_milestone() {
    let refused = Verdict::Refused {
        code: RefusalCode::new("UNBOUND_PLATFORM", Some("fs.cpSync")).unwrap(),
    };
    assert!(!refused.is_green());
    assert_eq!(refused.class(), VerdictClass::Refused);
}

#[test]
fn separately_reported_verdicts_never_count_as_equivalence() {
    let classes: Vec<VerdictClass> = every_variant().iter().map(Verdict::class).collect();
    assert_eq!(
        classes,
        [
            VerdictClass::Green,
            VerdictClass::Green,
            VerdictClass::Refused,
            VerdictClass::ReportedSeparately,
            VerdictClass::ReportedSeparately,
            VerdictClass::ReportedSeparately,
            VerdictClass::ReportedSeparately,
            VerdictClass::Failure,
        ]
    );
}

#[test]
fn equivalence_claim_requires_native_share() {
    let json = r#"{"verdict":{"kind":"identical"},"shares":{"host_share":1,"platform_share":0,"refused_share":0}}"#;
    assert!(serde_json::from_str::<VerdictRecord>(json).is_err());

    let green_hosted = VerdictRecord {
        verdict: Verdict::Identical,
        shares: hosted_shares(),
    };
    let claim = green_hosted.equivalence_claim().unwrap();
    assert_eq!(claim.basis, ClaimBasis::HostedBaseline);
    assert_eq!(claim.shares, hosted_shares());

    let promoted_shares = Shares::new(0.4, 0.5, 0.1, 0.0).unwrap();
    let promoted = VerdictRecord {
        verdict: Verdict::Identical,
        shares: promoted_shares,
    };
    assert_eq!(
        promoted.equivalence_claim().unwrap().basis,
        ClaimBasis::Promoted
    );
}

#[test]
fn only_green_records_yield_an_equivalence_claim() {
    for verdict in every_variant()
        .into_iter()
        .filter(|verdict| !verdict.is_green())
    {
        let record = VerdictRecord {
            verdict,
            shares: hosted_shares(),
        };
        assert!(record.equivalence_claim().is_none(), "{record:?}");
    }
}

#[test]
fn every_variant_round_trips_through_json() {
    for verdict in every_variant() {
        let record = VerdictRecord {
            verdict,
            shares: hosted_shares(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert_eq!(
            serde_json::from_str::<VerdictRecord>(&json).unwrap(),
            record,
            "{json}"
        );
    }
}

#[test]
fn unknown_kinds_normalizers_and_fields_are_rejected() {
    let shares =
        r#""shares":{"native_share":0,"host_share":1,"platform_share":0,"refused_share":0}"#;
    let bad = [
        r#"{"kind":"green"}"#,
        r#"{"kind":"normalized","normalizer":"N-INVENTADO"}"#,
        r#"{"kind":"normalized"}"#,
        r#"{"kind":"identical","extra":1}"#,
        r#"{"kind":"identical","normalizer":null}"#,
        r#"{"kind":"refused","code":"ABC","reason":null}"#,
        r#"{"kind":"identical_under_assumption","assumptions":[]}"#,
    ];
    for verdict in bad {
        let json = format!("{{\"verdict\":{verdict},{shares}}}");
        assert!(
            serde_json::from_str::<VerdictRecord>(&json).is_err(),
            "{json}"
        );
    }
}
