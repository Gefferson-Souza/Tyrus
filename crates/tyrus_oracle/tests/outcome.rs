use tyrus_oracle::driver::outcome::{sha256_hex, Outcome};

fn with_stderr(text: &str) -> Outcome {
    Outcome {
        exit_code: Some(1),
        stdout: Vec::new(),
        stderr: text.as_bytes().to_vec(),
        timed_out: false,
    }
}

#[test]
fn sha256_matches_the_known_vectors() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

#[test]
fn first_error_line_is_the_first_name_message_pair() {
    let stderr = "file:///x.ts:3\n  throw new Error('x');\n  ^\n\nRangeError: Invalid count\n    at foo\nTypeError: depois\n";
    assert_eq!(
        with_stderr(stderr).error_first_line().as_deref(),
        Some("RangeError: Invalid count")
    );
    assert_eq!(
        with_stderr("AggregateException: varios\n").error_first_line().as_deref(),
        Some("AggregateException: varios")
    );
}

#[test]
fn lines_that_only_look_like_errors_are_ignored() {
    assert_eq!(with_stderr("aviso: algo\nErro sem dois pontos\n").error_first_line(), None);
    assert_eq!(with_stderr("Error sem separador\n").error_first_line(), None);
    assert_eq!(with_stderr("nome: Error\n").error_first_line(), None);
}
