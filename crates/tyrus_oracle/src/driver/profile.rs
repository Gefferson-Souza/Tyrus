//! Perfis de comparação gravados no cabeçalho de `equivalence.json`.
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Profile {
    /// Padrão: bytes de stdout e stderr mais código de saída.
    Exact,
    /// Só para o controle negativo da tupla semente: `stdout.strip()` e `rc != 0` = sonda ruim.
    LegacyTrim,
}

impl Profile {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::LegacyTrim => "legacy_trim",
        }
    }

    /// O perfil legado reproduz o `run_probes.py`, que nunca usou o shim.
    #[must_use]
    pub const fn uses_shim(self) -> bool {
        matches!(self, Self::Exact)
    }
}

impl FromStr for Profile {
    type Err = String;

    fn from_str(name: &str) -> Result<Self, Self::Err> {
        match name {
            "exact" => Ok(Self::Exact),
            "legacy_trim" => Ok(Self::LegacyTrim),
            other => Err(format!(
                "perfil desconhecido `{other}` (use `exact` ou `legacy_trim`)"
            )),
        }
    }
}
