//! Frações de CPU e de recusa carregadas por todo veredito (D21). `native_share`
//! zero imprime "baseline hospedado"; "compila tudo" exige `host_share` zero.
use serde::{Deserialize, Serialize};

use crate::error::OracleError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "f64", into = "f64")]
pub struct Share(f64);

impl Share {
    pub fn new(value: f64) -> Result<Self, OracleError> {
        if value.is_finite() && (0.0..=1.0).contains(&value) {
            Ok(Self(value))
        } else {
            Err(OracleError::InvalidShare {
                value: value.to_string(),
            })
        }
    }

    #[must_use]
    pub const fn get(self) -> f64 {
        self.0
    }

    #[must_use]
    pub fn is_zero(self) -> bool {
        self.0 <= 0.0
    }
}

impl TryFrom<f64> for Share {
    type Error = OracleError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<Share> for f64 {
    fn from(share: Share) -> Self {
        share.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Shares {
    pub native_share: Share,
    pub host_share: Share,
    pub platform_share: Share,
    pub refused_share: Share,
}

impl Shares {
    pub fn new(native: f64, host: f64, platform: f64, refused: f64) -> Result<Self, OracleError> {
        Ok(Self {
            native_share: Share::new(native)?,
            host_share: Share::new(host)?,
            platform_share: Share::new(platform)?,
            refused_share: Share::new(refused)?,
        })
    }

    #[must_use]
    pub fn is_hosted_baseline(&self) -> bool {
        self.native_share.is_zero()
    }

    #[must_use]
    pub fn permits_compiles_everything_claim(&self) -> bool {
        self.host_share.is_zero()
    }
}
