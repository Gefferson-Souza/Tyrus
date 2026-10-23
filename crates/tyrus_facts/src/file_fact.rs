//! Caminho relativo portável e o par (caminho, sha256) usado por `facts.json`.
use serde::{Deserialize, Serialize};

use crate::error::FactsError;
use crate::sha256::Sha256Hex;

/// Caminho com `/`, relativo à raiz do inventário; `.` é a própria raiz.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct RelativePath(String);

impl RelativePath {
    /// # Errors
    /// `InvalidDocument` para vazio, absoluto, com `..`, `\` ou segmento vazio.
    pub fn parse(text: &str) -> Result<Self, FactsError> {
        let escapes = text.is_empty()
            || text.starts_with('/')
            || text.contains('\\')
            || text.contains('\0')
            || text != "."
                && text
                    .split('/')
                    .any(|part| part.is_empty() || part == ".." || part == ".");
        if escapes {
            Err(FactsError::invalid(format!(
                "caminho fora do formato relativo portável: `{text}`"
            )))
        } else {
            Ok(Self(text.to_owned()))
        }
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for RelativePath {
    type Error = FactsError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::parse(&value)
    }
}

impl From<RelativePath> for String {
    fn from(value: RelativePath) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileFact {
    path: RelativePath,
    sha256: Sha256Hex,
}

impl FileFact {
    /// # Errors
    /// `InvalidDocument` quando o caminho não é relativo portável.
    pub fn new(path: &str, sha256: Sha256Hex) -> Result<Self, FactsError> {
        Ok(Self {
            path: RelativePath::parse(path)?,
            sha256,
        })
    }

    #[must_use]
    pub fn path(&self) -> &str {
        self.path.as_str()
    }

    #[must_use]
    pub fn sha256(&self) -> &Sha256Hex {
        &self.sha256
    }
}

/// Rejeita caminhos repetidos (a ordem fica com quem produz a lista).
pub(crate) fn ensure_unique<'a>(
    paths: impl Iterator<Item = &'a str>,
    list: &str,
) -> Result<(), FactsError> {
    let mut seen = std::collections::HashSet::new();
    for path in paths {
        if !seen.insert(path) {
            return Err(FactsError::invalid(format!(
                "caminho repetido em {list}: `{path}`"
            )));
        }
    }
    Ok(())
}

/// Exige ordem estritamente crescente por bytes: ordenação e unicidade num passo só.
pub(crate) fn ensure_strictly_sorted<'a>(
    paths: impl Iterator<Item = &'a str>,
    list: &str,
) -> Result<(), FactsError> {
    let mut previous: Option<&str> = None;
    for path in paths {
        if previous.is_some_and(|before| before >= path) {
            return Err(FactsError::invalid(format!(
                "{list} fora de ordem estrita ou com repetição em `{path}`"
            )));
        }
        previous = Some(path);
    }
    Ok(())
}
