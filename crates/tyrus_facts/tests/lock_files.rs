#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use tyrus_facts::{
    canonical_bytes, parse_closure_lock, parse_partition_lock, ClosureFile, ClosureLock, FileMode,
    PartitionLock, PartitionUnit, Placement,
};

fn closure() -> ClosureLock {
    let files = [
        ("engine/app.js", FileMode::Regular, "a"),
        ("node_modules/.bin/x", FileMode::Executable, "b"),
        ("package.json", FileMode::Regular, "c"),
    ];
    ClosureLock::new(
        files
            .iter()
            .map(|(path, mode, seed)| {
                ClosureFile::new(path, *mode, support::digest_of(seed)).unwrap()
            })
            .collect(),
    )
    .unwrap()
}

#[test]
fn closure_lock_golden_faz_ida_e_volta() {
    let golden = include_str!("golden/closure.lock");
    let parsed = parse_closure_lock(golden.as_bytes()).unwrap();
    assert_eq!(parsed, closure());
    assert_eq!(
        String::from_utf8(canonical_bytes(&parsed).unwrap()).unwrap(),
        golden
    );
}

#[test]
fn partition_lock_golden_faz_ida_e_volta() {
    let golden = include_str!("golden/partition.lock");
    let parsed = parse_partition_lock(golden.as_bytes()).unwrap();
    assert_eq!(parsed.units().len(), 3);
    assert_eq!(parsed.units()[0].reason(), "FLOOR");
    assert_eq!(
        String::from_utf8(canonical_bytes(&parsed).unwrap()).unwrap(),
        golden
    );
}

#[test]
fn closure_lock_exige_ordem_estrita_por_bytes() {
    let b = ClosureFile::new("b.js", FileMode::Regular, support::digest_of("b")).unwrap();
    let a = ClosureFile::new("a.js", FileMode::Regular, support::digest_of("a")).unwrap();
    assert!(ClosureLock::new(vec![b.clone(), a.clone()]).is_err());
    assert!(ClosureLock::new(vec![a.clone(), a]).is_err());
    assert!(ClosureLock::new(vec![b]).is_ok());
}

#[test]
fn closure_lock_recusa_modo_fora_do_normalizado() {
    let mut value: serde_json::Value =
        serde_json::from_str(include_str!("golden/closure.lock")).unwrap();
    value["files"][0]["mode"] = "0600".into();
    assert_eq!(
        parse_closure_lock(value.to_string().as_bytes())
            .unwrap_err()
            .stable_code(),
        "TYRUS-E7103"
    );
}

#[test]
fn partition_lock_recusa_motivo_fora_do_formato() {
    for bad in ["floor", "", "COM ESPACO", "1ABC"] {
        assert!(
            PartitionUnit::new("a.js", support::digest_of("a"), Placement::Hosted, bad).is_err(),
            "{bad:?}"
        );
    }
}

#[test]
fn partition_lock_exige_ordem_estrita() {
    let unit = |path: &str| {
        PartitionUnit::new(path, support::digest_of(path), Placement::Hosted, "FLOOR").unwrap()
    };
    assert!(PartitionLock::new(vec![unit("b.js"), unit("a.js")]).is_err());
    assert!(PartitionLock::new(vec![unit("a.js"), unit("b.js")]).is_ok());
}

#[test]
fn construtores_nao_aceitam_hash_de_outro_tipo() {
    assert!(tyrus_facts::Sha256Hex::parse("xyz").is_err());
    assert!(tyrus_facts::Sha256Hex::parse(&"A".repeat(64)).is_err());
    assert!(tyrus_facts::Sha256Hex::parse(&"a".repeat(64)).is_ok());
}
