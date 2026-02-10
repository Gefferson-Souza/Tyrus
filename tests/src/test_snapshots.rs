#[cfg(test)]
use insta::assert_snapshot;
use std::path::PathBuf;
use tyrus_common::fs::FilePath;

#[test]
fn test_snapshot_interface_simple() {
    let path = PathBuf::from("fixtures/pass_simple_dto/input.ts");
    let result = tyrus_orchestrator::build(FilePath::from(path)).unwrap();
    assert_snapshot!(result);
}

#[test]
fn test_snapshot_class_simple() {
    let path = PathBuf::from("fixtures/test_class/input.ts");
    let result = tyrus_orchestrator::build(FilePath::from(path)).unwrap();
    assert_snapshot!(result);
}

#[test]
fn test_snapshot_e2e_full_stack() {
    let path = PathBuf::from("fixtures/e2e_full_stack/input.ts");
    let result = tyrus_orchestrator::build(FilePath::from(path)).unwrap();
    assert_snapshot!(result);
}

#[test]
fn test_snapshot_smoke_valid() {
    let path = PathBuf::from("fixtures/smoke_valid/input.ts");
    let result = tyrus_orchestrator::build(FilePath::from(path)).unwrap();
    assert_snapshot!(result);
}
