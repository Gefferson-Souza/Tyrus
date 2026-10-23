#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

mod record_support;

use std::fs;
use std::path::{Path, PathBuf};

use record_support::*;

#[test]
fn missing_typescript_install_is_an_io_error_with_exit_74() {
    let (corpus, out) = fixture("no-typescript");
    fs::remove_dir_all(corpus.join("node_modules")).unwrap();
    let (c, o, l) = strings(&corpus, &out);
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 74, "{}", run.stderr);
    assert!(run.stderr.contains("TYRUS-E5052"), "{}", run.stderr);
}

fn emit_script(dir: &Path, exit_code: i32) -> PathBuf {
    let path = dir.join(format!("fake-front-{exit_code}.js"));
    write(
        &path,
        &format!(
            "const fs = require('fs');\nconst out = process.argv[process.argv.indexOf('--out') + 1];\n\
             fs.mkdirSync(out + '/emit/probes', {{ recursive: true }});\n\
             fs.writeFileSync(out + '/emit/probes/A01_stable.js', 'console.log(1);\\n');\n\
             process.exit({exit_code});\n"
        ),
    );
    path
}

fn run_with_sidecar_exit(name: &str, exit_code: i32) -> (Run, PathBuf) {
    let (corpus, out) = fixture_with(
        name,
        &[(
            "probes",
            "A01_stable",
            "console.log(1);\n",
            "console.log(1);\n",
        )],
    );
    let front = emit_script(corpus.parent().unwrap(), exit_code);
    let (c, o, l) = strings(&corpus, &out);
    let f = front.display().to_string();
    let args = [
        "--corpus",
        &c,
        "--emit-dir",
        &o,
        "--front",
        &f,
        "--lock",
        &l,
    ];
    (record(&args, process_env), corpus)
}

#[test]
fn sidecar_exit_codes_0_and_1_are_accepted() {
    for code in [0, 1] {
        let (run, corpus) = run_with_sidecar_exit(&format!("sidecar-{code}"), code);
        assert_eq!(run.code, 0, "código {code}: {}", run.stderr);
        assert!(corpus.join("golden/probes/A01_stable.json").is_file());
    }
}

#[test]
fn sidecar_exit_code_3_refuses_with_the_emit_error() {
    let (run, corpus) = run_with_sidecar_exit("sidecar-3", 3);
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("TYRUS-E5050"), "{}", run.stderr);
    assert!(!corpus.join("golden").exists());
}
