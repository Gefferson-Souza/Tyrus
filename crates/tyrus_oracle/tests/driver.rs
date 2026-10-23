#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::unwrap_used)]

use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::Duration;

use tyrus_oracle::driver::profile::Profile;
use tyrus_oracle::driver::record::ShimCanary;
use tyrus_oracle::driver::run::{run_corpus, RunConfig, RunSummary};
use tyrus_oracle::driver::runner::CommandSpec;
use tyrus_oracle::lock::OracleLock;
use tyrus_oracle::tuple::NodeRole;
use tyrus_oracle::verdict::Verdict;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn node() -> PathBuf {
    PathBuf::from(std::env::var("TYRUS_NODE_REFERENCE").expect("TYRUS_NODE_REFERENCE"))
}

fn scratch(name: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("driver")
        .join(name);
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).unwrap();
    dir
}

fn write_script(dir: &Path, name: &str, body: &str) -> PathBuf {
    let path = dir.join(name);
    fs::write(&path, format!("#!/bin/sh\n{body}\n")).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o755)).unwrap();
    path
}

const FLAG_PICKER: &str = "flag=--experimental-strip-types";

fn node_target(dir: &Path, suffix: &str) -> CommandSpec {
    let body = format!(
        "{FLAG_PICKER}\nexec {} $flag \"$1\"{suffix}",
        node().display()
    );
    CommandSpec::parse(&write_script(dir, "target.sh", &body).display().to_string()).unwrap()
}

fn config(name: &str, probes: PathBuf, hosted: CommandSpec) -> RunConfig {
    let lock = OracleLock::load(&manifest().join("../../oracle/oracle.lock")).unwrap();
    RunConfig {
        corpus: "test".to_owned(),
        probes_dir: probes,
        out_dir: scratch(&format!("{name}-out")),
        profile: Profile::Exact,
        tuple_role: NodeRole::Reference,
        tuple: lock.reference,
        reference_node: node(),
        hosted,
        promoted: None,
        shim: manifest().join("../../oracle/shim/fix-env.js"),
        self_check: false,
        filter: None,
        reference_timeout: Duration::from_secs(20),
        hosted_timeout: Duration::from_secs(10),
    }
}

fn seed_probes() -> PathBuf {
    manifest().join("../../oracle/corpus/seed/probes")
}

fn probe_dir(name: &str, probes: &[(&str, &str)]) -> PathBuf {
    let dir = scratch(&format!("{name}-probes"));
    for (id, source) in probes {
        fs::write(dir.join(format!("{id}.ts")), source).unwrap();
    }
    dir
}

fn verdicts(summary: &tyrus_oracle::driver::run::RunSummary) -> Vec<(String, Verdict)> {
    summary
        .records
        .iter()
        .map(|record| (record.probe.clone(), record.result.verdict.clone()))
        .collect()
}

#[test]
fn node_against_itself_is_identical_for_every_probe() {
    let targets = scratch("self-target");
    let mut cfg = config("self", seed_probes(), node_target(&targets, ""));
    cfg.filter = Some("C01_0".to_owned());
    let summary = run_corpus(&cfg).unwrap();
    assert!(summary.records.len() >= 8, "{}", summary.records.len());
    for (id, verdict) in verdicts(&summary) {
        assert_eq!(verdict, Verdict::Identical, "{id}");
    }
    assert!(summary.records.iter().all(|r| r.profile == "exact"));
    assert!(cfg
        .out_dir
        .join("C01_01_int_float/equivalence.json")
        .is_file());
}

#[test]
fn a_target_that_changes_one_byte_is_different() {
    let targets = scratch("mutant-target");
    let target = node_target(&targets, " | sed '1s/./Z/'");
    let mut cfg = config("mutant", seed_probes(), target);
    cfg.filter = Some("C01_02".to_owned());
    let summary = run_corpus(&cfg).unwrap();
    let (_, verdict) = verdicts(&summary).remove(0);
    assert_eq!(verdict, Verdict::Different);
    assert!(!summary.records[0].result.verdict.is_green());
    assert_eq!(summary.records[0].detail.as_deref(), Some("stdout"));
}

#[test]
fn a_target_that_times_out_is_recorded_and_never_identical() {
    let targets = scratch("sleep-target");
    let target = CommandSpec::parse(
        &write_script(&targets, "sleep.sh", "exec sleep 30")
            .display()
            .to_string(),
    )
    .unwrap();
    let mut cfg = config("timeout", seed_probes(), target);
    cfg.filter = Some("C01_02".to_owned());
    cfg.hosted_timeout = Duration::from_millis(300);
    let summary = run_corpus(&cfg).unwrap();
    let record = &summary.records[0];
    assert_eq!(record.result.verdict, Verdict::Different);
    assert_eq!(record.detail.as_deref(), Some("TIMEOUT"));
    assert_eq!(record.hosted.as_ref().map(|h| h.timed_out), Some(true));
}

#[test]
fn a_timer_order_that_depends_on_real_waiting_is_a_nondeterministic_reference() {
    let probes = probe_dir(
        "litmus",
        &[
            (
                "T01_timers",
                "setTimeout(() => console.log('a'), 5);\nsetTimeout(() => console.log('b'), 1);\n",
            ),
            ("T02_stable", "console.log('estavel');\n"),
        ],
    );
    let targets = scratch("litmus-target");
    let summary = run_corpus(&config("litmus", probes, node_target(&targets, ""))).unwrap();
    let by_id = verdicts(&summary);
    assert_eq!(
        by_id[0],
        ("T01_timers".to_owned(), Verdict::NondeterministicReference)
    );
    assert_eq!(by_id[1], ("T02_stable".to_owned(), Verdict::Identical));
}

#[test]
fn legacy_trim_profile_is_written_in_the_record_header() {
    let probes = probe_dir(
        "legacy",
        &[("L01_trail", "process.stdout.write('x \\n');\n")],
    );
    let targets = scratch("legacy-target");
    let target = node_target(&targets, " | sed 's/ $//'");
    let mut cfg = config("legacy", probes.clone(), target.clone());
    let exact = run_corpus(&cfg).unwrap();
    assert_eq!(exact.records[0].result.verdict, Verdict::Different);
    cfg.profile = Profile::LegacyTrim;
    cfg.out_dir = scratch("legacy-out2");
    let legacy = run_corpus(&cfg).unwrap();
    assert_eq!(legacy.records[0].result.verdict, Verdict::Identical);
    let text = fs::read_to_string(cfg.out_dir.join("L01_trail/equivalence.json")).unwrap();
    assert!(text.contains("\"profile\": \"legacy_trim\""), "{text}");
}

#[test]
fn a_missing_hosted_program_is_an_error_not_a_verdict() {
    let probes = probe_dir("spawn", &[("S01", "console.log(1);\n")]);
    let target = CommandSpec::parse("/nao/existe/alvo").unwrap();
    let error = run_corpus(&config("spawn", probes, target)).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5012");
}

#[test]
fn an_empty_probes_dir_is_an_error() {
    let probes = scratch("empty-probes");
    let targets = scratch("empty-target");
    let error = run_corpus(&config("empty", probes, node_target(&targets, ""))).unwrap_err();
    assert_eq!(error.stable_code(), "TYRUS-E5011");
}

#[test]
fn a_probe_the_reference_cannot_run_is_a_bad_probe_never_identical() {
    let targets = scratch("c13-target");
    let mut cfg = config("c13", seed_probes(), node_target(&targets, ""));
    cfg.filter = Some("C13_01".to_owned());
    let summary = run_corpus(&cfg).unwrap();
    let record = &summary.records[0];
    assert!(
        matches!(record.result.verdict, Verdict::BadProbe { .. }),
        "{:?}",
        record.result.verdict
    );
    assert!(!record.result.verdict.is_green());
}

fn promoted_run(name: &str, hosted_suffix: &str, promoted_suffix: &str) -> RunSummary {
    let probes = probe_dir(name, &[("P01", "console.log('p');\n")]);
    let hosted_dir = scratch(&format!("{name}-hosted"));
    let promoted_dir = scratch(&format!("{name}-promoted"));
    let mut cfg = config(name, probes, node_target(&hosted_dir, hosted_suffix));
    cfg.promoted = Some(node_target(&promoted_dir, promoted_suffix));
    run_corpus(&cfg).unwrap()
}

#[test]
fn the_promoted_leg_is_judged_against_the_reference_not_the_hosted_leg() {
    let summary = promoted_run("promoted-ok", " | sed '1s/./Z/'", "");
    let record = &summary.records[0];
    assert_eq!(record.result.verdict, Verdict::Different);
    let promoted = record.promoted.as_ref().unwrap();
    assert_eq!(promoted.verdict, Verdict::Identical);
    assert_eq!(promoted.detail, None);
    assert!(record.promoted_target.is_some());
}

#[test]
fn a_promoted_leg_that_changes_one_byte_is_different_while_hosted_stays_identical() {
    let summary = promoted_run("promoted-bad", "", " | sed '1s/./Z/'");
    let record = &summary.records[0];
    assert_eq!(record.result.verdict, Verdict::Identical);
    let promoted = record.promoted.as_ref().unwrap();
    assert_eq!(promoted.verdict, Verdict::Different);
    assert_eq!(promoted.detail.as_deref(), Some("stdout"));
}

#[test]
fn the_record_identifies_each_target_and_flags_node_against_node() {
    let targets = scratch("ident-target");
    let probes = probe_dir("ident", &[("I01", "console.log('i');\n")]);
    let mut cfg = config("ident", probes.clone(), node_target(&targets, ""));
    let wrapper = run_corpus(&cfg).unwrap();
    let record = &wrapper.records[0];
    assert!(record.hosted_target.command.ends_with("target.sh"));
    assert_eq!(
        record
            .hosted_target
            .program_sha256
            .as_ref()
            .map(String::len),
        Some(64)
    );
    assert!(!record.self_check);
    cfg.out_dir = scratch("ident-out2");
    cfg.hosted = CommandSpec {
        program: node(),
        args: vec!["--experimental-strip-types".to_owned()],
    };
    let direct = run_corpus(&cfg).unwrap();
    assert!(direct.records[0].self_check);
    cfg.hosted = node_target(&targets, "");
    cfg.self_check = true;
    cfg.out_dir = scratch("ident-out3");
    assert!(run_corpus(&cfg).unwrap().records[0].self_check);
}

#[test]
fn the_canary_proves_whether_the_target_applied_the_shim() {
    let probes = probe_dir("canary", &[("K01", "console.log('k');\n")]);
    let targets = scratch("canary-target");
    let applied = run_corpus(&config("canary", probes.clone(), node_target(&targets, ""))).unwrap();
    assert_eq!(applied.records[0].shim_canary, ShimCanary::Applied);
    let ignoring = scratch("canary-ignoring");
    let body = format!(
        "exec env -u NODE_OPTIONS {} --experimental-strip-types \"$1\"",
        node().display()
    );
    let target =
        CommandSpec::parse(&write_script(&ignoring, "t.sh", &body).display().to_string()).unwrap();
    let missed = run_corpus(&config("canary2", probes.clone(), target)).unwrap();
    assert_eq!(missed.records[0].shim_canary, ShimCanary::NotApplied);
    let mut legacy = config("canary3", probes, node_target(&targets, ""));
    legacy.profile = Profile::LegacyTrim;
    let unchecked = run_corpus(&legacy).unwrap();
    assert_eq!(unchecked.records[0].shim_canary, ShimCanary::NotChecked);
}
