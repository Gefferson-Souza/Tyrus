#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]
mod support;

use std::fs;

use tyrus_facts::{
    ClosureFile, ClosureLock, Facts, FactsError, FileFact, FileMode, PartitionLock, PartitionUnit,
    Placement,
};

fn facts_for(tree: &support::TempTree) -> Facts {
    let source = tree.write("src/main.ts", b"export const a = 1;\n");
    let tsconfig = tree.write("tsconfig.json", b"{}\n");
    let output = tree.write("out/main.js", b"exports.a = 1;\n");
    let mut facts: Facts = serde_json::from_str(support::REPLICA_FACTS).unwrap();
    facts.tsconfig = tsconfig;
    facts.sources = vec![source];
    facts.outputs = vec![FileFact::new("main.js", output.sha256().clone()).unwrap()];
    facts
}

#[test]
fn arvore_intacta_valida() {
    let tree = support::TempTree::new("intacta");
    let facts = facts_for(&tree);
    facts
        .verify(tree.path(), &tree.path().join("out"))
        .expect("hashes batem");
}

#[test]
fn alterar_um_byte_de_uma_fonte_da_e7101() {
    let tree = support::TempTree::new("fonte");
    let facts = facts_for(&tree);
    fs::write(tree.path().join("src/main.ts"), b"export const a = 2;\n").unwrap();
    let error = facts
        .verify(tree.path(), &tree.path().join("out"))
        .unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7101");
    assert!(error.to_string().contains("src/main.ts"), "{error}");
}

#[test]
fn alterar_um_byte_de_uma_saida_da_e7101() {
    let tree = support::TempTree::new("saida");
    let facts = facts_for(&tree);
    fs::write(tree.path().join("out/main.js"), b"exports.a = 2;\n").unwrap();
    let error = facts
        .verify(tree.path(), &tree.path().join("out"))
        .unwrap_err();
    assert!(matches!(error, FactsError::HashMismatch { .. }), "{error}");
}

#[test]
fn alterar_o_tsconfig_da_e7101() {
    let tree = support::TempTree::new("tsconfig");
    let facts = facts_for(&tree);
    fs::write(tree.path().join("tsconfig.json"), b"{ }\n").unwrap();
    assert_eq!(
        facts
            .verify(tree.path(), &tree.path().join("out"))
            .unwrap_err()
            .stable_code(),
        "TYRUS-E7101"
    );
}

#[test]
fn arquivo_ausente_e_divergencia_de_hash() {
    let tree = support::TempTree::new("ausente");
    let facts = facts_for(&tree);
    fs::remove_file(tree.path().join("src/main.ts")).unwrap();
    let error = facts
        .verify(tree.path(), &tree.path().join("out"))
        .unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E7101");
    assert!(error.to_string().contains("ausente"), "{error}");
}

#[test]
fn closure_lock_detecta_byte_alterado() {
    let tree = support::TempTree::new("closure");
    let entry = tree.write("node_modules/a/index.js", b"module.exports = 1;\n");
    let lock = ClosureLock::new(vec![ClosureFile::new(
        entry.path(),
        FileMode::Regular,
        entry.sha256().clone(),
    )
    .unwrap()])
    .unwrap();
    lock.verify(tree.path()).expect("intacto");
    fs::write(
        tree.path().join("node_modules/a/index.js"),
        b"module.exports = 2;\n",
    )
    .unwrap();
    assert_eq!(
        lock.verify(tree.path()).unwrap_err().stable_code(),
        "TYRUS-E7101"
    );
}

#[test]
fn partition_lock_detecta_byte_alterado() {
    let tree = support::TempTree::new("partition");
    let entry = tree.write("engine/app.js", b"1;\n");
    let unit = PartitionUnit::new(
        entry.path(),
        entry.sha256().clone(),
        Placement::Hosted,
        "FLOOR",
    )
    .unwrap();
    let lock = PartitionLock::new(vec![unit]).unwrap();
    lock.verify(tree.path()).expect("intacto");
    fs::write(tree.path().join("engine/app.js"), b"2;\n").unwrap();
    assert_eq!(
        lock.verify(tree.path()).unwrap_err().stable_code(),
        "TYRUS-E7101"
    );
}
