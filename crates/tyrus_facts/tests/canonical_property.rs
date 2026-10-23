#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use serde_json::{Map, Value};
use tyrus_facts::{canonical_bytes, canonicalize_json, parse_facts};

/// PRNG determinístico (xorshift64*): o teste de propriedade precisa ser reproduzível.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 >> 12;
        self.0 ^= self.0 << 25;
        self.0 ^= self.0 >> 27;
        self.0.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    fn below(&mut self, bound: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(bound).unwrap()).unwrap()
    }
}

/// Reconstrói o valor com as chaves de todo objeto em ordem embaralhada.
fn shuffle_keys(value: &Value, rng: &mut Rng) -> Value {
    match value {
        Value::Object(map) => {
            let mut entries: Vec<(String, Value)> = map
                .iter()
                .map(|(k, v)| (k.clone(), shuffle_keys(v, rng)))
                .collect();
            for index in (1..entries.len()).rev() {
                entries.swap(index, rng.below(index + 1));
            }
            Value::Object(entries.into_iter().collect::<Map<_, _>>())
        }
        Value::Array(items) => Value::Array(items.iter().map(|v| shuffle_keys(v, rng)).collect()),
        other => other.clone(),
    }
}

/// Serializa preservando a ordem de inserção das chaves, sem passar pelo canonicalizador.
fn render_in_order(value: &Value, rng: &mut Rng) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            for index in (1..keys.len()).rev() {
                keys.swap(index, rng.below(index + 1));
            }
            let body: Vec<String> = keys
                .iter()
                .map(|key| {
                    format!(
                        "{}:{}",
                        serde_json::to_string(key).unwrap(),
                        render_in_order(&map[*key], rng)
                    )
                })
                .collect();
            format!("{{{}}}", body.join(","))
        }
        Value::Array(items) => format!(
            "[{}]",
            items
                .iter()
                .map(|v| render_in_order(v, rng))
                .collect::<Vec<_>>()
                .join(",")
        ),
        other => other.to_string(),
    }
}

#[test]
fn canonico_e_estavel_sob_reordenacao_de_chaves() {
    let facts = parse_facts(support::REPLICA_FACTS.as_bytes()).unwrap();
    let baseline = canonical_bytes(&facts).unwrap();
    let source: Value = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    let mut rng = Rng(0x9E37_79B9_7F4A_7C15);
    for round in 0..200 {
        let text = render_in_order(&shuffle_keys(&source, &mut rng), &mut rng);
        assert_eq!(
            canonicalize_json(text.as_bytes()).unwrap(),
            baseline,
            "rodada {round}"
        );
        let reparsed = parse_facts(text.as_bytes()).unwrap();
        assert_eq!(
            canonical_bytes(&reparsed).unwrap(),
            baseline,
            "rodada {round}"
        );
    }
}

#[test]
fn canonico_ordena_chaves_por_bytes_em_todos_os_niveis() {
    let out = canonicalize_json(br#"{"b":{"z":1,"a":[{"y":1,"x":2}]},"a":null}"#).unwrap();
    assert_eq!(
        String::from_utf8(out).unwrap(),
        "{\"a\":null,\"b\":{\"a\":[{\"x\":2,\"y\":1}],\"z\":1}}\n"
    );
}

#[test]
fn ordem_de_arrays_e_significativa() {
    let a = canonicalize_json(b"[1,2]").unwrap();
    let b = canonicalize_json(b"[2,1]").unwrap();
    assert_ne!(a, b);
}

#[test]
fn canonico_e_idempotente() {
    let once = canonicalize_json(support::REPLICA_FACTS.as_bytes()).unwrap();
    assert_eq!(canonicalize_json(&once).unwrap(), once);
}
