#![allow(clippy::expect_used, clippy::unwrap_used, clippy::indexing_slicing)]

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};
use tyrus_oracle::lock::OracleLock;
use tyrus_oracle::record::model::{DiffReport, GoldenMeta, Status};

fn repo() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn live() -> PathBuf {
    repo().join("oracle/corpus/live")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap()
}

fn schema_validator() -> jsonschema::Validator {
    let schema: Value =
        serde_json::from_str(&read(&repo().join("schemas/rerecord-diff.schema.json"))).unwrap();
    jsonschema::validator_for(&schema).unwrap()
}

fn diff_value() -> Value {
    serde_json::from_str(&read(&live().join("rerecord-diff.json"))).unwrap()
}

fn report() -> DiffReport {
    serde_json::from_str(&read(&live().join("rerecord-diff.json"))).unwrap()
}

#[test]
fn committed_diff_validates_against_the_schema() {
    let errors: Vec<String> = schema_validator()
        .iter_errors(&diff_value())
        .map(|e| e.to_string())
        .collect();
    assert_eq!(errors, Vec::<String>::new());
}

#[test]
fn schema_rejects_a_bad_probe_without_reason_and_a_changed_without_change() {
    let validator = schema_validator();
    let mut bad = diff_value();
    bad["probes"][0] = json!({"id": "X", "set": "probes", "status": "BAD_PROBE"});
    assert!(!validator.is_valid(&bad));
    let mut changed = diff_value();
    changed["probes"][0] = json!({"id": "X", "set": "probes", "status": "CHANGED"});
    assert!(!validator.is_valid(&changed));
    let mut green_with_reason = diff_value();
    green_with_reason["probes"][0] =
        json!({"id": "X", "set": "probes", "status": "SAME", "reason": "x"});
    assert!(!validator.is_valid(&green_with_reason));
}

#[test]
fn c12_04_is_a_known_bad_probe() {
    let report = report();
    let entry = report
        .probes
        .iter()
        .find(|p| p.id == "C12_04_top_level_await")
        .unwrap();
    assert_eq!(entry.status, Status::BadProbe);
    assert!(entry
        .reason
        .as_ref()
        .unwrap()
        .as_str()
        .contains("SyntaxError"));
    assert_eq!(entry.baseline_stdout.as_deref(), Some("7\n"));
    assert_eq!(entry.baseline_exit_code, Some(0));
}

#[test]
fn counts_add_up_and_bad_probes_are_not_green() {
    let report = report();
    let c = &report.counts;
    assert_eq!(c.same + c.changed + c.bad_probe, c.total);
    assert_eq!(c.total, report.probes.len());
    assert_eq!(
        c.same,
        report
            .probes
            .iter()
            .filter(|p| p.status == Status::Same)
            .count()
    );
}

#[test]
fn every_probe_has_exactly_one_of_golden_or_bad_probe() {
    let report = report();
    let mut seen = BTreeSet::new();
    for entry in &report.probes {
        let golden = live().join(format!("golden/{}/{}.json", entry.set, entry.id));
        assert_eq!(
            golden.is_file(),
            entry.status != Status::BadProbe,
            "{}/{}",
            entry.set,
            entry.id
        );
        assert!(
            seen.insert((entry.set.clone(), entry.id.clone())),
            "duplicada"
        );
        let source = live().join(format!("{}/{}.ts", entry.set, entry.id));
        assert!(source.is_file(), "sonda sem fonte: {}", source.display());
    }
    let goldens = ["probes", "calib"]
        .iter()
        .map(|set| {
            std::fs::read_dir(live().join("golden").join(set))
                .unwrap()
                .count()
        })
        .sum::<usize>();
    assert_eq!(goldens, report.probes.len() - report.counts.bad_probe);
}

#[test]
fn golden_meta_tuple_equals_oracle_lock() {
    let lock = OracleLock::load(&repo().join("oracle/oracle.lock")).unwrap();
    let meta: GoldenMeta = serde_json::from_str(&read(&live().join("golden/meta.json"))).unwrap();
    assert_eq!(meta.reference, lock.reference);
    assert_eq!(report().reference, lock.reference);
    assert_eq!(report().seed, lock.seed);
}

#[test]
fn live_sources_are_byte_identical_to_the_seed() {
    for set in ["probes", "calib"] {
        let seed_dir = repo().join("oracle/corpus/seed").join(set);
        let mut names = 0;
        for entry in std::fs::read_dir(&seed_dir).unwrap() {
            let path = entry.unwrap().path();
            let name = path.file_name().unwrap().to_owned();
            assert_eq!(
                std::fs::read(&path).unwrap(),
                std::fs::read(live().join(set).join(name)).unwrap()
            );
            names += 1;
        }
        assert_eq!(std::fs::read_dir(live().join(set)).unwrap().count(), names);
    }
}
