#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

use std::process::Command;

const ALLOWED_WORKSPACE_PREFIX: &str = "tyrus_archive";

#[test]
fn dependency_tree_has_no_other_workspace_crate() {
    let cargo = std::env::var("CARGO").unwrap_or_else(|_| "cargo".into());
    let output = Command::new(cargo)
        .args([
            "tree",
            "-p",
            "tyrus_archive",
            "--prefix",
            "none",
            "--edges",
            "normal,build",
            "--offline",
        ])
        .current_dir(env!("CARGO_MANIFEST_DIR"))
        .output()
        .unwrap();
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let tree = String::from_utf8(output.stdout).unwrap();
    assert!(tree.starts_with(ALLOWED_WORKSPACE_PREFIX), "{tree}");
    let offenders: Vec<&str> = tree
        .lines()
        .filter(|line| line.starts_with("tyrus_") && !line.starts_with(ALLOWED_WORKSPACE_PREFIX))
        .collect();
    assert_eq!(offenders, Vec::<&str>::new());
}
