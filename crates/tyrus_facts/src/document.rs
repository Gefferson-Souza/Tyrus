//! Leitura de documentos versionados: a versão do esquema é checada ANTES do parse
//! tipado, para um documento de versão futura nunca virar "campo desconhecido".
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;

use crate::closure::ClosureLock;
use crate::error::FactsError;
use crate::facts::Facts;
use crate::partition::PartitionLock;

pub const SUPPORTED_SCHEMA_VERSION: u64 = 1;

/// Versão de esquema suportada; só o literal `1` desserializa.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct SchemaVersion;

impl SchemaVersion {
    #[must_use]
    pub const fn number(self) -> u64 {
        SUPPORTED_SCHEMA_VERSION
    }
}

impl Serialize for SchemaVersion {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u64(SUPPORTED_SCHEMA_VERSION)
    }
}

impl<'de> Deserialize<'de> for SchemaVersion {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = Value::deserialize(deserializer)?;
        if value.as_u64() == Some(SUPPORTED_SCHEMA_VERSION) {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "schema_version {value} não suportada"
            )))
        }
    }
}

#[derive(Deserialize)]
struct VersionProbe {
    schema_version: Option<Value>,
}

fn parse_versioned<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, FactsError> {
    let probe: VersionProbe =
        serde_json::from_slice(bytes).map_err(|error| FactsError::invalid(error.to_string()))?;
    let found = probe.schema_version.unwrap_or(Value::Null);
    if found.as_u64() != Some(SUPPORTED_SCHEMA_VERSION) {
        return Err(FactsError::UnknownSchemaVersion {
            found: found.to_string(),
            supported: SUPPORTED_SCHEMA_VERSION,
        });
    }
    serde_json::from_slice(bytes).map_err(|error| FactsError::invalid(error.to_string()))
}

/// # Errors
/// `E7102` para versão desconhecida; `E7103` para documento fora do esquema.
pub fn parse_facts(bytes: &[u8]) -> Result<Facts, FactsError> {
    let facts: Facts = parse_versioned(bytes)?;
    facts.validate()?;
    Ok(facts)
}

/// # Errors
/// `E7102` para versão desconhecida; `E7103` para documento fora do esquema.
pub fn parse_closure_lock(bytes: &[u8]) -> Result<ClosureLock, FactsError> {
    parse_versioned(bytes)
}

/// # Errors
/// `E7102` para versão desconhecida; `E7103` para documento fora do esquema.
pub fn parse_partition_lock(bytes: &[u8]) -> Result<PartitionLock, FactsError> {
    parse_versioned(bytes)
}
