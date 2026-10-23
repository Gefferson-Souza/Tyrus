#![allow(clippy::indexing_slicing, clippy::expect_used, clippy::unwrap_used)]

mod common;

use common::{sample_files, write_archive};
use tyrus_archive::cli::{run, EXIT_IO, EXIT_OK, EXIT_USAGE};

fn args(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn text_output_lists_header_and_one_line_per_entry() {
    let dir = std::env::temp_dir().join(format!("tyrus-archive-text-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let file = dir.join("closure.tyar");
    std::fs::write(&file, write_archive(&sample_files())).unwrap();
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        &args(&["inspect", file.to_str().unwrap()]),
        &mut out,
        &mut err,
    )
    .unwrap();
    std::fs::remove_dir_all(&dir).unwrap();
    assert_eq!(code, EXIT_OK);
    assert_eq!(err, Vec::<u8>::new());
    let text = String::from_utf8(out).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    assert!(lines[0].starts_with("formato 1 | 4 entradas | conjunto "));
    assert_eq!(lines.len(), 5);
    assert!(lines[1].starts_with("755 ") && lines[1].ends_with("  bin/run.sh"));
}

#[test]
fn missing_file_exits_with_io_code_and_stable_error() {
    let (mut out, mut err) = (Vec::new(), Vec::new());
    let code = run(
        &args(&["inspect", "/nao/existe/closure.tyar"]),
        &mut out,
        &mut err,
    )
    .unwrap();
    assert_eq!(code, EXIT_IO);
    assert!(String::from_utf8(err).unwrap().starts_with("TYRUS-E8105"));
}

#[test]
fn extra_positional_argument_and_unknown_command_are_usage_errors() {
    for case in [
        args(&["inspect", "a", "b"]),
        args(&["inspect", "--json"]),
        args(&["verify", "a"]),
    ] {
        let (mut out, mut err) = (Vec::new(), Vec::new());
        assert_eq!(run(&case, &mut out, &mut err).unwrap(), EXIT_USAGE);
        assert!(String::from_utf8(err).unwrap().starts_with("uso:"));
    }
}
