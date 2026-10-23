"""Testes do rules_lint.py com árvores de fixture em diretório temporário."""

import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
REPO = HERE.parents[1]
sys.path.insert(0, str(HERE))

import rules_lint  # noqa: E402

CONSTITUTION = """# Constitution
| # | Rule |
|---|---|
| R1 | Translate |
| R2 | Names |
"""

BASE_DATA = {
    "gates": {"unsafe": {"state": "ACTIVE"}, "claims": {"state": "SKIPPED", "activates_in": "docs-claims-migration"}},
    "constitution_rules": {"R1": {"planned_in": "place-all-hosted"}, "R2": {"gates": ["unsafe"]}},
    "rules_lint": {
        "naming_paths": ["docs", "CLAUDE.md", ".claude/rules"],
        "naming_allow": ["Power of Ten v2"],
        "adr_dir": "docs/architecture/decisions",
        "adr_index": "docs/ARCHITECTURE.md",
        "constitution": "docs/standards/CONSTITUTION.md",
    },
    "loc_budget": {
        "extensions": [".rs", ".py"],
        "tooling_max_ratio": 0.25,
        "categories": {"tooling": ["scripts/"], "oracle": ["oracle/"]},
    },
}


def write(root, rel, text):
    path = Path(root) / rel
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(text, encoding="utf-8")


class TempRoot(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)

    def tearDown(self):
        self.tmp.cleanup()


class NamingTest(TempRoot):
    def lint(self, text):
        write(self.root, "docs/a.md", text)
        return rules_lint.check_naming(self.root, BASE_DATA)

    def test_compiler_iteration_name_fails(self):
        errors = self.lint("Isto é o v2 do compilador.\n")
        self.assertEqual(len(errors), 1)
        self.assertIn("docs/a.md:1", errors[0])

    def test_tool_versions_pass(self):
        self.assertEqual(self.lint("node v22 e `v8 =147.0.0` e actions/checkout@v4 e v2.1.0\n"), [])

    def test_action_tags_and_file_names_pass(self):
        self.assertEqual(self.lint("Swatinem/rust-cache@v2 e roadmap-v3.md e (v8)\n"), [])

    def test_allowlisted_context_passes(self):
        self.assertEqual(self.lint("Power of Ten v2 vale para o legado\n"), [])

    def test_portuguese_iteration_number_fails(self):
        self.assertEqual(len(self.lint("Na iteração 2 tudo mudou\n")), 1)

    def test_crate_name_with_iteration_token_fails(self):
        write(self.root, "crates/tyrus_v2/Cargo.toml", "[package]\nname = \"tyrus_v2\"\n")
        errors = rules_lint.check_naming(self.root, BASE_DATA)
        self.assertTrue(any("tyrus_v2" in e for e in errors))

    def test_claude_md_and_rules_are_scanned(self):
        write(self.root, "CLAUDE.md", "veja o v3\n")
        write(self.root, ".claude/rules/x.md", "o v1 antigo\n")
        self.assertEqual(len(rules_lint.check_naming(self.root, BASE_DATA)), 2)


class ConstitutionTest(TempRoot):
    def setUp(self):
        super().setUp()
        write(self.root, "docs/standards/CONSTITUTION.md", CONSTITUTION)

    def test_complete_mapping_passes_and_planned_prints_skipped(self):
        errors, notes = rules_lint.check_constitution(self.root, BASE_DATA)
        self.assertEqual(errors, [])
        self.assertIn("R1: SKIPPED(place-all-hosted)", notes)
        self.assertFalse(any("PASS" in n for n in notes))

    def test_rule_without_gate_fails(self):
        data = {**BASE_DATA, "constitution_rules": {"R1": {"planned_in": "x"}}}
        errors, _ = rules_lint.check_constitution(self.root, data)
        self.assertEqual(len(errors), 1)
        self.assertIn("R2", errors[0])

    def test_entry_for_unknown_rule_fails(self):
        rules = {**BASE_DATA["constitution_rules"], "R9": {"gates": ["unsafe"]}}
        errors, _ = rules_lint.check_constitution(self.root, {**BASE_DATA, "constitution_rules": rules})
        self.assertEqual(len(errors), 1)
        self.assertIn("R9", errors[0])

    def test_unknown_gate_fails(self):
        rules = {"R1": {"planned_in": "x"}, "R2": {"gates": ["ghost"]}}
        errors, _ = rules_lint.check_constitution(self.root, {**BASE_DATA, "constitution_rules": rules})
        self.assertEqual(len(errors), 1)
        self.assertIn("ghost", errors[0])

    def test_skipped_gate_reports_phase(self):
        rules = {"R1": {"gates": ["claims"]}, "R2": {"gates": ["unsafe"]}}
        _, notes = rules_lint.check_constitution(self.root, {**BASE_DATA, "constitution_rules": rules})
        self.assertIn("R1: SKIPPED(docs-claims-migration)", notes)


class AdrTest(TempRoot):
    def adr(self, number, status):
        write(self.root, f"docs/architecture/decisions/{number}-x.md", f"# {number}\n\nDate: 2026\nStatus: {status}\n")

    def index(self, *numbers):
        links = "\n".join(f"- [ADR {n}](architecture/decisions/{n}-x.md)" for n in numbers)
        write(self.root, "docs/ARCHITECTURE.md", links + "\n")

    def test_consistent_adrs_pass(self):
        self.adr("0001", "Accepted")
        self.adr("0002", "Proposto — aguardando")
        self.index("0001", "0002")
        self.assertEqual(rules_lint.check_adrs(self.root, BASE_DATA), [])

    def test_superseded_but_accepted_fails(self):
        self.adr("0001", "Accepted. Superseded by ADR 0002")
        self.index("0001")
        errors = rules_lint.check_adrs(self.root, BASE_DATA)
        self.assertEqual(len(errors), 1)
        self.assertIn("0001", errors[0])

    def test_superseded_status_alone_passes(self):
        self.adr("0001", "Superseded by ADR 0002")
        self.adr("0002", "Accepted")
        self.index("0001", "0002")
        self.assertEqual(rules_lint.check_adrs(self.root, BASE_DATA), [])

    def test_indexed_adr_missing_fails(self):
        self.adr("0001", "Accepted")
        self.index("0001", "0005")
        errors = rules_lint.check_adrs(self.root, BASE_DATA)
        self.assertEqual(len(errors), 1)
        self.assertIn("0005", errors[0])

    def test_unindexed_adr_fails(self):
        self.adr("0001", "Accepted")
        self.adr("0002", "Accepted")
        self.index("0001")
        errors = rules_lint.check_adrs(self.root, BASE_DATA)
        self.assertEqual(len(errors), 1)
        self.assertIn("0002", errors[0])

    def test_unknown_status_fails(self):
        self.adr("0001", "Maybe")
        self.index("0001")
        self.assertEqual(len(rules_lint.check_adrs(self.root, BASE_DATA)), 1)


class LocBudgetTest(TempRoot):
    def lines(self, rel, count):
        write(self.root, rel, "x = 1\n" * count)

    def test_ratio_is_computed_from_the_tree(self):
        self.lines("scripts/a.py", 10)
        self.lines("crates/p/src/lib.rs", 30)
        self.lines("oracle/o.py", 60)
        self.lines("docs/ignored.md", 500)
        stats = rules_lint.loc_stats(self.root, BASE_DATA)
        self.assertEqual((stats["tooling"], stats["total"]), (10, 100))
        self.assertAlmostEqual(stats["ratio"], 0.10)
        self.assertAlmostEqual(stats["wide_ratio"], 0.70)

    def test_over_budget_fails(self):
        self.lines("scripts/a.py", 30)
        self.lines("crates/p/src/lib.rs", 70)
        errors, _ = rules_lint.check_loc(self.root, BASE_DATA)
        self.assertEqual(len(errors), 1)
        self.assertIn("30.0%", errors[0])

    def test_at_the_limit_passes(self):
        self.lines("scripts/a.py", 25)
        self.lines("crates/p/src/lib.rs", 75)
        errors, notes = rules_lint.check_loc(self.root, BASE_DATA)
        self.assertEqual(errors, [])
        self.assertTrue(any("25.0%" in n for n in notes))


def git(root, *args):
    subprocess.run(["git", "-C", str(root), "-c", "user.name=t", "-c", "user.email=t@t", *args],
                   check=True, capture_output=True)


class AdrTriggerTest(TempRoot):
    def setUp(self):
        super().setUp()
        git(self.root, "init", "-q", "-b", "main")
        for name in ("a", "b", "c", "d"):
            write(self.root, f"crates/{name}/src/lib.rs", "pub fn f() {}\n")
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", "base")
        git(self.root, "checkout", "-q", "-b", "pr")

    def commit(self, message="change"):
        git(self.root, "add", "-A")
        git(self.root, "commit", "-q", "-m", message)

    def trigger(self, body=""):
        return rules_lint.check_adr_trigger(self.root, "main", "HEAD", body)

    def touch_crates(self, names):
        for name in names:
            write(self.root, f"crates/{name}/src/x.rs", "pub fn g() {}\n")

    def test_three_crates_without_adr_fail(self):
        self.touch_crates("abc")
        self.commit()
        errors = self.trigger()
        self.assertEqual(len(errors), 1)
        self.assertIn("3 crates", errors[0])

    def test_two_crates_pass(self):
        self.touch_crates("ab")
        self.commit()
        self.assertEqual(self.trigger(), [])

    def test_adr_in_diff_satisfies_the_trigger(self):
        self.touch_crates("abc")
        write(self.root, "docs/architecture/decisions/0020-x.md", "# 20\n")
        self.commit()
        self.assertEqual(self.trigger(), [])

    def test_adr_reference_in_pr_body_satisfies_the_trigger(self):
        self.touch_crates("abc")
        self.commit()
        self.assertEqual(self.trigger("Segue o ADR 0016"), [])

    def test_adr_reference_in_commit_message_satisfies_the_trigger(self):
        self.touch_crates("abc")
        self.commit("feat: x (ADR 0016)")
        self.assertEqual(self.trigger(), [])

    def test_new_public_trait_fails(self):
        write(self.root, "crates/a/src/t.rs", "pub trait Handler {\n    fn run(&self);\n}\n")
        self.commit()
        errors = self.trigger()
        self.assertEqual(len(errors), 1)
        self.assertIn("pub trait", errors[0])

    def test_private_trait_passes(self):
        write(self.root, "crates/a/src/t.rs", "trait Handler {}\n")
        self.commit()
        self.assertEqual(self.trigger(), [])


class RealTreeTest(unittest.TestCase):
    def test_static_checks_pass_on_the_real_tree(self):
        result = subprocess.run([sys.executable, "-B", "-I", str(HERE / "rules_lint.py"), "static"],
                                capture_output=True, text=True, cwd=REPO)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)

    def test_real_ratio_is_reported_not_hardcoded(self):
        data = rules_lint.load_config(REPO / "gates.toml")
        stats = rules_lint.loc_stats(REPO, data)
        print(f"\nrazao real de ferramental: {stats['ratio']:.1%} ({stats['tooling']}/{stats['total']} LOC)")
        self.assertGreater(stats["total"], 0)
        self.assertLessEqual(stats["ratio"], data["loc_budget"]["tooling_max_ratio"])


if __name__ == "__main__":
    unittest.main()
