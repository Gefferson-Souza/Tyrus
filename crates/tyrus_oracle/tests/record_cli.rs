#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

mod record_support;

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

use serde_json::Value;

use record_support::*;

#[test]
fn missing_corpus_flag_is_a_usage_error() {
    let run = record(&["--check"], process_env);
    assert_eq!(run.code, 64);
    assert!(
        run.stderr.contains("--corpus é obrigatório"),
        "{}",
        run.stderr
    );
}

#[test]
fn wrong_reference_tuple_aborts_with_exit_2() {
    let (corpus, out) = fixture("wrong-tuple");
    let fake = corpus.parent().unwrap().join("fake-node");
    write(
        &fake,
        "#!/bin/sh\necho '{\"node\":\"24.21.0\",\"v8\":\"13.6.233.17-node.53\",\"icu\":\"78.3\",\"unicode\":\"17.0\",\"tz\":\"2026c\",\"cldr\":\"48.0\"}'\n",
    );
    fs::set_permissions(&fake, fs::Permissions::from_mode(0o755)).unwrap();
    let fake_path = fake.into_os_string();
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    let run = record(&base_args(&c, &o, &l), |name| {
        (name == "TYRUS_NODE_REFERENCE").then(|| fake_path.clone())
    });
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("TYRUS-E5001"), "{}", run.stderr);
    assert!(!corpus.join("golden").exists(), "nada pode ser gravado");
}

#[test]
fn record_then_check_round_trip() {
    let (corpus, out) = fixture("round-trip");
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert_eq!(
        run.stdout,
        "record: total=4 same=1 changed=1 bad_probe=2 nondeterministic=1\n"
    );
    assert!(corpus.join("golden/probes/A01_stable.json").is_file());
    assert!(!corpus.join("golden/probes/A02_flaky.json").exists());
    assert!(!corpus.join("golden/probes/A03_throws.json").exists());
    let diff = json(&corpus.join("rerecord-diff.json"));
    let reasons: Vec<&str> = diff["probes"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|p| p["reason"].as_str())
        .collect();
    assert!(
        reasons.contains(&"reference_exit_nonzero(1): RangeError: boom"),
        "{reasons:?}"
    );
    let mut check_args = base_args(&c, &o, &l);
    check_args.push("--check");
    let check = record(&check_args, process_env);
    assert_eq!(check.code, 0, "{}", check.stderr);
}

#[test]
fn check_fails_when_a_golden_no_longer_matches_the_reference() {
    let (corpus, out) = fixture("stale-golden");
    let (c, o, l) = (
        corpus.display().to_string(),
        out.display().to_string(),
        lock_arg(),
    );
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    write(&out.join("emit/probes/A01_stable.js"), "console.log(9);\n");
    let mut check_args = base_args(&c, &o, &l);
    check_args.push("--check");
    let check = record(&check_args, process_env);
    assert_eq!(check.code, 1);
    assert!(
        check.stderr.contains("probes/A01_stable: golden diverge"),
        "{}",
        check.stderr
    );
}

#[test]
fn relative_paths_work_like_absolute_ones() {
    let (corpus, out) = fixture("relative");
    let cwd = std::env::current_dir().unwrap();
    let rel = |p: &Path| pathdiff(p, &cwd);
    let (c, o, l) = (rel(&corpus), rel(&out), lock_arg());
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("same=1"), "{}", run.stdout);
}

#[test]
fn reference_killed_by_signal_is_a_bad_probe_not_a_golden() {
    let (corpus, out) = fixture_with(
        "signal",
        &[(
            "probes",
            "P1_sigkill",
            "console.log(1);\n",
            "console.log('partial'); process.kill(process.pid, 'SIGKILL');\n",
        )],
    );
    let (c, o, l) = strings(&corpus, &out);
    let run = record(&base_args(&c, &o, &l), process_env);
    assert_eq!(run.code, 0, "{}", run.stderr);
    assert!(run.stdout.contains("bad_probe=1"), "{}", run.stdout);
    assert!(!corpus.join("golden/probes/P1_sigkill.json").exists());
    assert_eq!(
        diff_entry(&corpus, "P1_sigkill")["reason"],
        "reference_killed_by_signal"
    );
}

#[test]
fn bad_probe_records_what_the_seed_baseline_prints() {
    let (corpus, out) = fixture_with(
        "bad-baseline",
        &[(
            "probes",
            "T01_top_level_await",
            "console.log(await Promise.resolve(7));\n",
            "throw new SyntaxError('await is only valid in async functions');\n",
        )],
    );
    write(&corpus.join("package.json"), "{\"type\": \"commonjs\"}\n");
    let (c, o, l) = strings(&corpus, &out);
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    let entry = diff_entry(&corpus, "T01_top_level_await");
    assert_eq!(entry["status"], "BAD_PROBE");
    assert_eq!(entry["baseline_stdout"], "7\n");
    assert_eq!(entry["baseline_exit_code"], 0);
}

#[test]
fn baseline_flag_follows_the_c13_prefix() {
    let enum_ts = "enum E { A = 1 }\nconsole.log(E.A);\n";
    let (corpus, out) = fixture_with(
        "transform-prefix",
        &[
            ("probes", "C13_enum", enum_ts, "console.log(1);\n"),
            ("probes", "C01_enum", enum_ts, "console.log(1);\n"),
        ],
    );
    let (c, o, l) = strings(&corpus, &out);
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    assert_eq!(diff_entry(&corpus, "C13_enum")["status"], "SAME");
    let plain = diff_entry(&corpus, "C01_enum");
    assert_eq!(plain["status"], "CHANGED");
    assert_eq!(plain["change"], "baseline_exit_nonzero");
}

#[test]
fn seed_results_directory_feeds_the_seed_verdict() {
    let (corpus, out) = fixture("seed-results");
    let results = corpus.parent().unwrap().join("results");
    write(
        &results.join("A01_stable.json"),
        "{\"verdict\": \"RUST_DOES_NOT_COMPILE\"}\n",
    );
    let (c, o, l) = strings(&corpus, &out);
    let r = results.display().to_string();
    let mut args = base_args(&c, &o, &l);
    args.extend(["--seed-results", &r]);
    assert_eq!(record(&args, process_env).code, 0);
    assert_eq!(
        diff_entry(&corpus, "A01_stable")["seed_verdict"],
        "RUST_DOES_NOT_COMPILE"
    );
}

#[test]
fn wrong_seed_tuple_aborts_with_exit_2_and_writes_nothing() {
    let (corpus, out) = fixture("wrong-seed");
    let fake = node_script(
        corpus.parent().unwrap(),
        "fake-seed",
        "echo '{\"node\":\"22.0.0\",\"v8\":\"1.0.0.0-node.1\",\"icu\":\"1\",\"unicode\":\"1\",\"tz\":\"1\",\"cldr\":\"1\"}'",
    );
    let (c, o, l) = strings(&corpus, &out);
    let run = record(&base_args(&c, &o, &l), |name| {
        if name == "TYRUS_NODE_SEED" {
            Some(fake.clone())
        } else {
            process_env(name)
        }
    });
    assert_eq!(run.code, 2, "{}", run.stderr);
    assert!(run.stderr.contains("TYRUS-E5001"), "{}", run.stderr);
    assert!(!corpus.join("golden").exists());
}

#[test]
fn check_fails_when_the_stored_stderr_was_tampered_with() {
    let (corpus, out) = fixture("tamper-stderr");
    let (c, o, l) = strings(&corpus, &out);
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    let golden = corpus.join("golden/probes/A01_stable.json");
    let mut value = json(&golden);
    value["stderr"] = Value::from("ruído\n");
    write(&golden, &serde_json::to_string(&value).unwrap());
    let mut args = base_args(&c, &o, &l);
    args.push("--check");
    let check = record(&args, process_env);
    assert_eq!(check.code, 1, "{}", check.stderr);
    assert!(check.stderr.contains("golden diverge"), "{}", check.stderr);
}

#[test]
fn check_fails_when_a_bad_probe_changes_kind() {
    let (corpus, out) = fixture("bad-kind");
    let (c, o, l) = strings(&corpus, &out);
    assert_eq!(record(&base_args(&c, &o, &l), process_env).code, 0);
    let diff_file = corpus.join("rerecord-diff.json");
    let text = fs::read_to_string(&diff_file).unwrap();
    let swapped = text.replace("reference_exit_nonzero(1)", "reference_timeout");
    assert_ne!(text, swapped);
    write(&diff_file, &swapped);
    let mut args = base_args(&c, &o, &l);
    args.push("--check");
    let check = record(&args, process_env);
    assert_eq!(check.code, 1, "{}", check.stderr);
    assert!(
        check.stderr.contains("A03_throws: agora é BAD_PROBE"),
        "{}",
        check.stderr
    );
}
