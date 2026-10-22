//! Álgebra de vereditos fechada (D21). Só `Identical` e `Normalized` (com id do
//! registro fechado) são verdes; todo veredito carrega as frações de CPU.
use serde::{Deserialize, Serialize};

use crate::codes::{BadProbeReason, RefusalCode};
use crate::error::OracleError;
use crate::registry::{Assumptions, Normalizer};
use crate::shares::Shares;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", try_from = "VerdictWire")]
pub enum Verdict {
    Identical,
    Normalized { normalizer: Normalizer },
    Refused { code: RefusalCode },
    IdenticalUnderAssumption { assumptions: Assumptions },
    NondeterministicReference,
    BadProbe { reason: BadProbeReason },
    EmitSkew,
    Different,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum VerdictKind {
    Identical,
    Normalized,
    Refused,
    IdenticalUnderAssumption,
    NondeterministicReference,
    BadProbe,
    EmitSkew,
    Different,
}

/// Forma de fio: o serde ignora campos extras de variante unitária em enum com tag
/// interna, e o oráculo precisa rejeitá-los.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct VerdictWire {
    kind: VerdictKind,
    #[serde(default, deserialize_with = "present")]
    normalizer: Option<Normalizer>,
    #[serde(default, deserialize_with = "present")]
    code: Option<RefusalCode>,
    #[serde(default, deserialize_with = "present")]
    assumptions: Option<Assumptions>,
    #[serde(default, deserialize_with = "present")]
    reason: Option<BadProbeReason>,
}

/// `Option<T>` do serde aceita `null`; o esquema JSON não. Campo presente tem de ter valor.
fn present<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

impl TryFrom<VerdictWire> for Verdict {
    type Error = OracleError;

    fn try_from(wire: VerdictWire) -> Result<Self, Self::Error> {
        let verdict = match (
            wire.kind,
            wire.normalizer,
            wire.code,
            wire.assumptions,
            wire.reason,
        ) {
            (VerdictKind::Identical, None, None, None, None) => Self::Identical,
            (VerdictKind::Normalized, Some(normalizer), None, None, None) => {
                Self::Normalized { normalizer }
            }
            (VerdictKind::Refused, None, Some(code), None, None) => Self::Refused { code },
            (VerdictKind::IdenticalUnderAssumption, None, None, Some(assumptions), None) => {
                Self::IdenticalUnderAssumption { assumptions }
            }
            (VerdictKind::NondeterministicReference, None, None, None, None) => {
                Self::NondeterministicReference
            }
            (VerdictKind::BadProbe, None, None, None, Some(reason)) => Self::BadProbe { reason },
            (VerdictKind::EmitSkew, None, None, None, None) => Self::EmitSkew,
            (VerdictKind::Different, None, None, None, None) => Self::Different,
            _ => {
                return Err(OracleError::InvalidVerdictField {
                    field: "kind",
                    reason: "campos do veredito incompatíveis com o `kind` informado".to_owned(),
                })
            }
        };
        Ok(verdict)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VerdictClass {
    Green,
    /// Honesto, mas nunca verde em marco; tem catraca percentual.
    Refused,
    /// Relatado à parte; nunca conta como equivalência.
    ReportedSeparately,
    Failure,
}

impl Verdict {
    #[must_use]
    pub const fn class(&self) -> VerdictClass {
        match self {
            Self::Identical | Self::Normalized { .. } => VerdictClass::Green,
            Self::Refused { .. } => VerdictClass::Refused,
            Self::IdenticalUnderAssumption { .. }
            | Self::NondeterministicReference
            | Self::BadProbe { .. }
            | Self::EmitSkew => VerdictClass::ReportedSeparately,
            Self::Different => VerdictClass::Failure,
        }
    }

    #[must_use]
    pub fn is_green(&self) -> bool {
        self.class() == VerdictClass::Green
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClaimBasis {
    /// `native_share` zero: a frase impressa é "baseline hospedado".
    HostedBaseline,
    Promoted,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct EquivalenceClaim {
    pub basis: ClaimBasis,
    pub shares: Shares,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct VerdictRecord {
    pub verdict: Verdict,
    pub shares: Shares,
}

impl VerdictRecord {
    /// Só registros verdes sustentam uma alegação de equivalência, sempre com as frações.
    #[must_use]
    pub fn equivalence_claim(&self) -> Option<EquivalenceClaim> {
        if !self.verdict.is_green() {
            return None;
        }
        let basis = if self.shares.is_hosted_baseline() {
            ClaimBasis::HostedBaseline
        } else {
            ClaimBasis::Promoted
        };
        Some(EquivalenceClaim {
            basis,
            shares: self.shares,
        })
    }
}
