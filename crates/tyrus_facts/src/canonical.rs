//! JSON canônico: chaves ordenadas por bytes em todos os níveis, forma compacta e
//! `\n` final. A ordenação é feita aqui, sem depender de `preserve_order` do `serde_json`
//! (que outro crate do workspace pode ligar por unificação de features).
use std::collections::BTreeMap;

use serde::Serialize;
use serde_json::Value;

use crate::error::FactsError;

/// # Errors
/// `InvalidDocument` quando o valor não é representável como JSON.
pub fn canonical_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>, FactsError> {
    let tree =
        serde_json::to_value(value).map_err(|error| FactsError::invalid(error.to_string()))?;
    Ok(render(&tree))
}

/// # Errors
/// `InvalidDocument` quando os bytes não são JSON.
pub fn canonicalize_json(bytes: &[u8]) -> Result<Vec<u8>, FactsError> {
    let tree: Value =
        serde_json::from_slice(bytes).map_err(|error| FactsError::invalid(error.to_string()))?;
    Ok(render(&tree))
}

fn render(tree: &Value) -> Vec<u8> {
    let mut out = String::new();
    write_value(tree, &mut out);
    out.push('\n');
    out.into_bytes()
}

fn write_value(value: &Value, out: &mut String) {
    match value {
        Value::Object(map) => {
            let sorted: BTreeMap<&String, &Value> = map.iter().collect();
            out.push('{');
            for (index, (key, child)) in sorted.into_iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(key.clone()).to_string());
                out.push(':');
                write_value(child, out);
            }
            out.push('}');
        }
        Value::Array(items) => {
            out.push('[');
            for (index, child) in items.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                }
                write_value(child, out);
            }
            out.push(']');
        }
        scalar => out.push_str(&scalar.to_string()),
    }
}
