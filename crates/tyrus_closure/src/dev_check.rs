//! Dependências de desenvolvimento nunca entram no fecho.
use std::collections::{BTreeMap, BTreeSet};

use serde_json::Value;
use tyrus_facts::{ClosureFile, FactsError};

use crate::error::ClosureError;

const NODE_MODULES: &str = "node_modules/";
const TYPESCRIPT: &str = "typescript";
const TYPES_SCOPE: &str = "@types";

/// Pacotes do `package-lock.json` por caminho; `true` quando só existem para desenvolvimento.
type LockedPackages = BTreeMap<String, bool>;

/// `typescript` é sempre recusado. Um pacote de `@types` só é aceito quando o lockfile o
/// lista como dependência de produção (ex.: `@types/validator` do `class-validator`);
/// sem lockfile, todo `@types` é recusado. Entradas `dev: true` do lockfile também.
///
/// # Errors
/// `E7201` listando cada pacote de desenvolvimento encontrado; `E7103` se o lockfile for
/// JSON inválido.
pub fn ensure_no_dev(files: &[ClosureFile], lockfile: Option<&[u8]>) -> Result<(), ClosureError> {
    let locked = lockfile.map(locked_packages).transpose()?;
    let mut found = BTreeSet::new();
    for file in files {
        if let Some(package) = forbidden_package(file.path(), locked.as_ref()) {
            found.insert(package);
        }
    }
    for (package, dev_only) in locked.iter().flatten() {
        let prefix = format!("{package}/");
        if *dev_only && files.iter().any(|file| file.path().starts_with(&prefix)) {
            found.insert(package.clone());
        }
    }
    if found.is_empty() {
        Ok(())
    } else {
        Err(ClosureError::DevDependency {
            paths: found.into_iter().collect(),
        })
    }
}

/// Raiz do pacote proibido (`typescript` ou escopo `@types`) em qualquer profundidade.
fn forbidden_package(path: &str, locked: Option<&LockedPackages>) -> Option<String> {
    let segments: Vec<&str> = path.split('/').collect();
    segments.iter().enumerate().find_map(|(index, segment)| {
        if *segment != "node_modules" {
            return None;
        }
        let name = *segments.get(index + 1)?;
        let root_len = if name == TYPES_SCOPE {
            index + 3
        } else {
            index + 2
        };
        let root = segments.get(..root_len)?.join("/");
        let forbidden = match name {
            TYPESCRIPT => true,
            TYPES_SCOPE => locked.is_none_or(|map| map.get(&root) != Some(&false)),
            _ => false,
        };
        forbidden.then_some(root)
    })
}

fn locked_packages(lockfile: &[u8]) -> Result<LockedPackages, ClosureError> {
    let document: Value = serde_json::from_slice(lockfile).map_err(|error| {
        ClosureError::Facts(FactsError::InvalidDocument {
            reason: format!("package-lock.json inválido: {error}"),
        })
    })?;
    let packages = document.get("packages").and_then(Value::as_object);
    Ok(packages.map_or_else(BTreeMap::new, |map| {
        map.iter()
            .filter(|(key, _)| key.starts_with(NODE_MODULES))
            .map(|(key, entry)| {
                let dev_only = entry.get("dev").and_then(Value::as_bool) == Some(true);
                (key.clone(), dev_only)
            })
            .collect()
    }))
}
