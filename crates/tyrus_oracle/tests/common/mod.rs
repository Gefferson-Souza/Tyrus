#![allow(dead_code, clippy::expect_used, clippy::unwrap_used)]

use tyrus_oracle::codes::{BadProbeReason, RefusalCode};
use tyrus_oracle::registry::{Assumption, Assumptions, Normalizer};
use tyrus_oracle::shares::Shares;
use tyrus_oracle::verdict::Verdict;

pub(crate) const VARIANT_COUNT: usize = 8;

pub(crate) fn variant_index(verdict: &Verdict) -> usize {
    match verdict {
        Verdict::Identical => 0,
        Verdict::Normalized { .. } => 1,
        Verdict::Refused { .. } => 2,
        Verdict::IdenticalUnderAssumption { .. } => 3,
        Verdict::NondeterministicReference => 4,
        Verdict::BadProbe { .. } => 5,
        Verdict::EmitSkew => 6,
        Verdict::Different => 7,
    }
}

pub(crate) fn every_variant() -> Vec<Verdict> {
    vec![
        Verdict::Identical,
        Verdict::Normalized {
            normalizer: Normalizer::Stack,
        },
        Verdict::Refused {
            code: RefusalCode::new("UNBOUND_PLATFORM", Some("fs.cpSync")).unwrap(),
        },
        Verdict::IdenticalUnderAssumption {
            assumptions: Assumptions::try_from(vec![Assumption::NativeToString]).unwrap(),
        },
        Verdict::NondeterministicReference,
        Verdict::BadProbe {
            reason: BadProbeReason::try_from("C12_04".to_owned()).unwrap(),
        },
        Verdict::EmitSkew,
        Verdict::Different,
    ]
}

pub(crate) fn hosted_shares() -> Shares {
    Shares::new(0.0, 0.93, 0.07, 0.0).unwrap()
}
