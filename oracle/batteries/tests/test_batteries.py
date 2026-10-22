"""Testes das baterias de conformidade. Rodar com: python3 -I -m unittest discover -s oracle/batteries/tests -v"""
import contextlib
import hashlib
import importlib.util
import io
import json
import os
import re
import shutil
import tempfile
import unittest

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(HERE)
GOLDEN_DIR = os.path.join(ROOT, "goldens", "node-26.11.1")
SEED_PROBE_SHA256 = "5abf489ffac44c03c8da6ebd489929d8c103062f776bc7d00830375ac74309bf"
LITMUS_TRACES = {
    "L01_tick_vs_promise_main": "sync tick promise microtask",
    "L02_tick_promise_interleave": "sync tick1 tick2 tick-in-tick1 promise1 promise2 promise-in-tick1 promise-in-promise1 tick-in-promise1",
    "L03_queue_microtask_order": "sync qm1 p1 qm2 p3 p2 qm-in-qm2",
    "L06_nested_immediates": "a b c b-nested",
    "L07_tick_between_immediates": "i1 tick-in-i1 promise-in-i1 i2",
    "L08_tick_between_timers": "t1 tick-in-t1 promise-in-t1 t2",
    "L09_readfile_tick_vs_promise": "callback tick promise",
    "L11_async_await_basic": "f1 sync f2 p1 f3 p2 p3",
    "L13_return_await": "chain-1 resolved-awaited chain-2 resolved-plain chain-3 chain-4 chain-5",
    "L14_async_return_value_vs_promise": "val-1 chain-1 thenable-3 chain-2 prom-2 chain-3 chain-4 chain-5",
}
INSPECT_FIRST_LINES = {
    "I01_map_set": "Map(3) { 'a' => 1, { k: 1 } => [ 1, 2 ], NaN => Set(1) { 1 } }",
    "I04_circular": "<ref *1> { name: 'a', self: [Circular *1] }",
    "I05_strings_quotes": "[ \"it's\", 'say \"hi\"', `both ' and \"`, 'back`tick \\' \"', 'plain' ]",
}
FINGERPRINT_PROBE_COUNTS = {"core": 23, "intl": 15, "unicode": 9, "tz": 5, "runtime": 10}
CORE_CLASS_SIZES = {"sin": 20000, "pow": 20000, "sortCalls": 2000, "errorMessages": 18, "toFixed": 2000}


def load_runner(path=None):
    spec = importlib.util.spec_from_file_location("batteries_run", path or os.path.join(ROOT, "run.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


RUN = load_runner()


def require_env(name):
    value = os.environ.get(name)
    if not value:
        raise AssertionError(f"variavel {name} ausente: o teste nao pode passar sem o motor")
    return value


def delay_value(raw):
    text = raw.strip().strip("'\"")
    if text == "NaN":
        return 0
    try:
        return max(float(text), 0)
    except ValueError:
        return float("inf")


def timer_delays(source):
    promise_timers = "timers/promises" in source
    delays = []
    for match in re.finditer(r"(\.?)\b(?:setTimeout|setInterval)\(", source):
        depth, commas, index = 1, [], match.end()
        for index in range(match.end(), len(source)):
            char = source[index]
            if char in "([{":
                depth += 1
            elif char in ")]}":
                depth -= 1
                if depth == 0:
                    break
            elif char == "," and depth == 1:
                commas.append(index)
        bounds = [match.end() - 1] + commas + [index]
        args = [source[bounds[i] + 1:bounds[i + 1]] for i in range(len(bounds) - 1)]
        position = 0 if match.group(1) and promise_timers else 1
        delays.append(delay_value(args[position]) if position < len(args) else 0)
    return delays


def run_cli(*argv):
    out, err = io.StringIO(), io.StringIO()
    with contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
        code = RUN.main(list(argv))
    return code, out.getvalue(), err.getvalue()


def load_index():
    with open(os.path.join(ROOT, "index.json"), encoding="utf-8") as handle:
        return json.load(handle)


LITMUS_FILE = re.compile(r"L\d{2}_\w+\.(js|mjs)")


def litmus_names():
    folder = os.path.join(ROOT, "litmus")
    return sorted(os.path.splitext(f)[0] for f in os.listdir(folder) if LITMUS_FILE.fullmatch(f))


def litmus_path(name):
    for extension in (".js", ".mjs"):
        candidate = os.path.join(ROOT, "litmus", name + extension)
        if os.path.isfile(candidate):
            return candidate
    raise AssertionError(f"litmus ausente: {name}")


class CompareLogicTest(unittest.TestCase):
    FP = {"kind": "fingerprint", "probe": "core", "n": 3, "sha256": "aa"}
    LIT = {"kind": "litmus", "exit": 0, "sha256": "bb"}

    def test_same_and_diff_for_fingerprint(self):
        self.assertEqual(RUN.compare_class(self.FP, dict(self.FP), {}), "same")
        self.assertEqual(RUN.compare_class(self.FP, {**self.FP, "sha256": "zz"}, {}), "diff")
        self.assertEqual(RUN.compare_class(self.FP, {**self.FP, "n": 4}, {}), "diff")

    def test_exit_code_participates_in_litmus_comparison(self):
        self.assertEqual(RUN.compare_class(self.LIT, {**self.LIT, "exit": 1}, {}), "diff")

    def test_unstable_missing_and_failed_probe_are_never_same(self):
        self.assertEqual(RUN.compare_class(self.LIT, {"kind": "litmus", "unstable": True}, {}), "unstable")
        self.assertEqual(RUN.compare_class(self.LIT, None, {}), "missing")
        self.assertEqual(RUN.compare_class(self.FP, None, {"core": "exit 1"}), "error")

    def test_class_globs(self):
        self.assertTrue(RUN.matches("fingerprint.sin", ["fingerprint.*"]))
        self.assertTrue(RUN.matches("litmus.L01_x", None))
        self.assertFalse(RUN.matches("inspect.I01", ["litmus.*", "fingerprint.sin"]))

    def test_report_lists_every_non_same_class_sorted(self):
        golden = {"tuple": {"v": 1}, "classes": {"litmus.B": self.LIT, "litmus.A": self.LIT, "litmus.C": self.LIT}}
        actual = {"litmus.A": dict(self.LIT), "litmus.B": {**self.LIT, "sha256": "x"}}
        report = RUN.build_report("cmd", golden, RUN.Measurement({"v": 1}, actual, {}), None)
        self.assertEqual(report["diff_classes"], ["litmus.B", "litmus.C"])
        self.assertEqual(report["counts"], {"total": 3, "same": 1, "not_same": 2})
        self.assertTrue(report["tuple_matches"])

    def test_canonical_sha256_is_independent_of_engine_json_formatting(self):
        self.assertEqual(RUN.canonical_sha256(["a", "\ud83d"]), RUN.canonical_sha256(json.loads('[ "a" , "\\ud83d" ]')))

    def test_canonical_sha256_covers_every_element(self):
        base = [str(i) for i in range(100)]
        changed = base[:-1] + ["x"]
        self.assertNotEqual(RUN.canonical_sha256(base), RUN.canonical_sha256(changed))


class ExecutionTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reference = require_env("TYRUS_NODE_REFERENCE")

    def test_engine_env_is_hermetic(self):
        os.environ["NODE_OPTIONS"] = "--no-warnings"
        try:
            env = RUN.engine_env("America/Sao_Paulo", {"EXTRA": "1"}, "/home-x")
        finally:
            del os.environ["NODE_OPTIONS"]
        self.assertEqual(sorted(env), ["EXTRA", "HOME", "LANG", "LC_ALL", "NO_COLOR", "PATH", "TZ"])
        self.assertEqual((env["TZ"], env["HOME"], env["LC_ALL"]), ("America/Sao_Paulo", "/home-x", "en_US.UTF-8"))

    def test_run_script_captures_stdout_and_exit_code(self):
        with tempfile.TemporaryDirectory() as temp:
            script = os.path.join(temp, "s.js")
            with open(script, "w", encoding="utf-8") as handle:
                handle.write("process.stdout.write('hello'); process.exitCode = 3;")
            result = RUN.run_script(self.reference, script, RUN.engine_env("UTC", {}, temp))
        self.assertEqual(result, {"exit": 3, "stdout": b"hello"})

    def test_run_script_reports_timeout_instead_of_hanging(self):
        with tempfile.TemporaryDirectory() as temp:
            script = os.path.join(temp, "s.js")
            with open(script, "w", encoding="utf-8") as handle:
                handle.write("setInterval(() => {}, 1000);")
            result = RUN.run_script(self.reference, script, RUN.engine_env("UTC", {}, temp), timeout=1)
        self.assertEqual(result["exit"], "timeout")

    def test_missing_engine_raises_harness_error(self):
        with self.assertRaises(RUN.HarnessError):
            RUN.run_script("/nonexistent/engine", "/tmp/x.js", RUN.engine_env("UTC", {}, "/tmp"))

    def test_missing_script_raises_harness_error(self):
        with self.assertRaises(RUN.HarnessError):
            RUN.run_script(self.reference, "/nonexistent/script.js", RUN.engine_env("UTC", {}, "/tmp"))

    def test_unstable_script_is_detected(self):
        env = RUN.engine_env("UTC", {}, tempfile.gettempdir())
        classes, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "perturbation"), "litmus", env, RUN.Selection(5, None))
        self.assertTrue(classes["litmus.P01_hrtime_noise"]["unstable"])


class LitmusTest(unittest.TestCase):
    """Traces derivadas da semantica documentada do Node, independentes dos goldens."""

    @classmethod
    def setUpClass(cls):
        cls.reference = require_env("TYRUS_NODE_REFERENCE")
        cls.env = RUN.engine_env("UTC", {}, tempfile.gettempdir())
        cls.litmus = litmus_names()

    def test_traces_match_documented_node_semantics(self):
        for name, expected in LITMUS_TRACES.items():
            with self.subTest(litmus=name):
                result = RUN.run_script(self.reference, litmus_path(name), self.env)
                self.assertEqual(result["exit"], 0)
                self.assertEqual(result["stdout"].decode("utf-8").split(), expected.split())

    def test_no_litmus_uses_nondeterministic_sources(self):
        banned = re.compile(r"Math\.random|Date\.now|hrtime|performance\.now|randomBytes|randomUUID")
        for name in self.litmus:
            with open(litmus_path(name), encoding="utf-8") as handle:
                self.assertIsNone(banned.search(handle.read()), name)

    def test_no_litmus_depends_on_a_long_wait(self):
        for name in self.litmus:
            with open(litmus_path(name), encoding="utf-8") as handle:
                delays = timer_delays(handle.read())
            self.assertTrue(all(d < 10 for d in delays), f"{name}: {delays}")

    def test_timer_delay_scan_sees_waits_the_old_regex_missed(self):
        self.assertEqual(timer_delays("setTimeout(() => { L('a'); L('b'); }, 50);"), [50])
        self.assertEqual(timer_delays("await new Promise(r => setTimeout(r, 30));"), [30])
        self.assertEqual(timer_delays("const tp = require('timers/promises'); await tp.setTimeout(20);"), [20])
        self.assertEqual(timer_delays("setInterval(f, 15)"), [15])
        self.assertEqual(timer_delays("setTimeout(f, 1); setTimeout(g)"), [1, 0])
        self.assertEqual(timer_delays("setTimeout(f, delay)"), [float("inf")])

    def test_no_litmus_flutters_in_five_runs(self):
        classes, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "litmus"), "litmus", self.env, RUN.Selection(5, None))
        self.assertEqual([c for c, v in classes.items() if v.get("unstable")], [])
        self.assertEqual(len(classes), len(self.litmus))


class GoldenPortabilityTest(unittest.TestCase):
    def test_no_golden_output_leaks_host_paths_or_node_internals(self):
        forbidden = re.compile(r"/home/|/tmp/|/Users/|node:internal|tyrus-wt")
        outputs = os.path.join(GOLDEN_DIR, "outputs")
        checked = 0
        for folder, _, names in os.walk(outputs):
            for name in names:
                with open(os.path.join(folder, name), encoding="utf-8") as handle:
                    self.assertIsNone(forbidden.search(handle.read()), os.path.join(folder, name))
                checked += 1
        self.assertGreater(checked, 40)

    def test_compare_passes_from_a_copy_in_another_directory(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        with tempfile.TemporaryDirectory() as temp:
            clone = os.path.join(temp, "batteries")
            shutil.copytree(ROOT, clone, ignore=shutil.ignore_patterns("__pycache__"))
            lock = os.path.normpath(os.path.join(ROOT, load_index()["reference_lock"]))
            shutil.copy(lock, os.path.join(temp, "oracle.lock"))
            runner = load_runner(os.path.join(clone, "run.py"))
            code = runner.main(["--engine-cmd", reference, "--compare", os.path.join(clone, "goldens", "node-26.11.1"), "--class", "inspect.*"])
        self.assertEqual(code, 0)


class LitmusIndexTest(unittest.TestCase):
    def setUp(self):
        self.index = load_index()
        self.litmus = litmus_names()

    def test_has_at_least_25_litmus(self):
        self.assertGreaterEqual(len(self.litmus), 25)

    def test_has_exactly_11_sections(self):
        self.assertEqual(sorted(self.index["sections"]), [f"S{n:02d}" for n in range(1, 12)])

    def test_every_litmus_belongs_to_exactly_one_section(self):
        members = [m for s in self.index["sections"].values() for m in s["members"]]
        self.assertEqual(sorted(members), self.litmus)


class ReferenceLockTest(unittest.TestCase):
    def test_reference_tuple_comes_from_the_single_lock(self):
        index = load_index()
        self.assertNotIn("reference_tuple", index)
        tuple_info = RUN.reference_tuple(index)
        lock = RUN.load_json(os.path.normpath(os.path.join(ROOT, index["reference_lock"])))["reference"]
        self.assertEqual(tuple_info["runtime"], "node")
        self.assertEqual(tuple_info["version"], "v" + lock["node"])
        for key in ("v8", "icu", "unicode", "tz", "cldr"):
            self.assertEqual(tuple_info[key], lock[key], key)


ESM_LITMUS_TRACES = {
    "L31_esm_entry_order": "sync promise microtask tick immediate",
    "L32_esm_top_level_await": "before-await promise-1 after-await-null promise-2 tick tick-2 after-await-immediate timeout-0 after-await-timeout",
    "L33_esm_dynamic_import_order": "sync promise eval-esm-tla-start eval-esm-plain eval-esm-tla-end awaited-plain then-dep-tla tick immediate",
    "L34_require_esm_sync": "before-require eval-esm-plain after-require-plain-[object Module] same-instance-true sync-end tick promise",
    "L35_require_esm_tla_error": "before-require caught-ERR_REQUIRE_ASYNC_MODULE is-error-true after-require promise",
    "L36_esm_import_cjs": "eval-cjs-dep default-n-e named-n require-same-true meta-true",
}


class EsmLitmusTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reference = require_env("TYRUS_NODE_REFERENCE")
        cls.env = RUN.engine_env("UTC", {}, tempfile.gettempdir())

    def test_traces_match_documented_esm_semantics(self):
        for name, expected in ESM_LITMUS_TRACES.items():
            with self.subTest(litmus=name):
                result = RUN.run_script(self.reference, litmus_path(name), self.env)
                self.assertEqual(result["exit"], 0)
                self.assertEqual(result["stdout"].decode("utf-8").split(), expected.split())

    def test_runner_discovers_mjs_litmus_and_skips_fixtures(self):
        wanted = {f"litmus.{n}" for n in ESM_LITMUS_TRACES}
        classes, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "litmus"), "litmus", self.env, RUN.Selection(1, wanted))
        self.assertEqual(sorted(classes), sorted(wanted))
        self.assertEqual({v["exit"] for v in classes.values()}, {0})
        every, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "litmus"), "litmus", self.env, RUN.Selection(1, None))
        self.assertFalse([c for c in every if "esm_dep" in c or "cjs_dep" in c])

    def test_esm_litmus_are_the_last_six_and_total_is_36(self):
        self.assertEqual([n for n in litmus_names() if n >= "L31"], sorted(ESM_LITMUS_TRACES))
        self.assertEqual(len(litmus_names()), 36)

    def test_esm_litmus_do_not_flutter_in_five_runs(self):
        wanted = {f"litmus.{n}" for n in ESM_LITMUS_TRACES}
        classes, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "litmus"), "litmus", self.env, RUN.Selection(5, wanted))
        self.assertEqual([c for c, v in classes.items() if v.get("unstable")], [])
        self.assertEqual(len(classes), 6)


class InspectTest(unittest.TestCase):
    def test_first_lines_match_documented_node_formatting(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        env = RUN.engine_env("UTC", {}, tempfile.gettempdir())
        for name, expected in INSPECT_FIRST_LINES.items():
            with self.subTest(inspect=name):
                result = RUN.run_script(reference, os.path.join(ROOT, "inspect", name + ".js"), env)
                self.assertEqual(result["exit"], 0)
                self.assertEqual(result["stdout"].decode("utf-8").splitlines()[0], expected)

    def test_has_at_least_16_inspect_scripts(self):
        self.assertGreaterEqual(len([f for f in os.listdir(os.path.join(ROOT, "inspect")) if f.endswith(".js")]), 16)

    def test_no_inspect_script_is_unstable_in_five_runs(self):
        env = RUN.engine_env("UTC", {}, tempfile.gettempdir())
        classes, _ = RUN.script_classes(require_env("TYRUS_NODE_REFERENCE"), os.path.join(ROOT, "inspect"), "inspect", env, RUN.Selection(5, None))
        self.assertEqual([c for c, v in classes.items() if v.get("unstable")], [])


class FingerprintTest(unittest.TestCase):
    def test_seed_probe_is_preserved_verbatim(self):
        with open(os.path.join(ROOT, "fingerprint", "core.js"), "rb") as handle:
            self.assertEqual(hashlib.sha256(handle.read()).hexdigest(), SEED_PROBE_SHA256)

    def test_tuple_probe_reports_the_reference_tuple(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        env = RUN.engine_env("UTC", {}, tempfile.gettempdir())
        self.assertEqual(RUN.probe_tuple(reference, env), RUN.reference_tuple(load_index()))

    def test_probes_emit_the_expected_classes(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        index = load_index()
        classes, failed = RUN.fingerprint_classes(reference, index, lambda zone: RUN.engine_env(zone, {}, tempfile.gettempdir()), None)
        self.assertEqual(failed, {})
        per_probe = {}
        for data in classes.values():
            per_probe[data["probe"]] = per_probe.get(data["probe"], 0) + 1
        for probe, count in FINGERPRINT_PROBE_COUNTS.items():
            self.assertEqual(per_probe[probe], count, probe)
        for zone in index["fingerprint"]["local_time_zones"]:
            self.assertEqual(per_probe[f"tzlocal@{zone}"], 4, zone)
            self.assertIn(f"fingerprint.tzLocalName@{zone}", classes)
        for name, size in CORE_CLASS_SIZES.items():
            self.assertEqual(classes[f"fingerprint.{name}"]["n"], size, name)

    def test_probes_are_reproducible_on_the_reference(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        index = load_index()
        env_for = lambda zone: RUN.engine_env(zone, {}, tempfile.gettempdir())  # noqa: E731
        first, _ = RUN.fingerprint_classes(reference, index, env_for, {"intl", "unicode", "tz", "runtime"})
        second, _ = RUN.fingerprint_classes(reference, index, env_for, {"intl", "unicode", "tz", "runtime"})
        self.assertEqual(first, second)


class ReferenceEngineTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.reference = require_env("TYRUS_NODE_REFERENCE")

    def test_golden_meta_matches_reference_tuple(self):
        with open(os.path.join(GOLDEN_DIR, "meta.json"), encoding="utf-8") as handle:
            self.assertEqual(json.load(handle)["tuple"], RUN.reference_tuple(load_index()))

    def test_compare_against_goldens_is_green_and_reproducible(self):
        first = run_cli("--engine-cmd", self.reference, "--compare", GOLDEN_DIR)
        second = run_cli("--engine-cmd", self.reference, "--compare", GOLDEN_DIR)
        self.assertEqual(first[0], 0, first[2])
        self.assertEqual(first, second)
        report = json.loads(first[1])
        self.assertTrue(report["tuple_matches"])
        self.assertEqual(report["diff_classes"], [])
        self.assertGreaterEqual(report["counts"]["total"], 130)
        self.assertEqual({s["status"] for s in report["sections"].values()}, {"same"})

    def test_class_filter_only_runs_what_was_asked(self):
        code, out, _ = run_cli("--engine-cmd", self.reference, "--compare", GOLDEN_DIR, "--class", "fingerprint.sin,litmus.L01*")
        self.assertEqual(code, 0)
        self.assertEqual(sorted(json.loads(out)["statuses"]), ["fingerprint.sin", "litmus.L01_tick_vs_promise_main"])

    def test_class_filter_matching_nothing_is_a_harness_error(self):
        code, out, err = run_cli("--engine-cmd", self.reference, "--compare", GOLDEN_DIR, "--class", "litmus.L99_typo")
        self.assertEqual(code, 2)
        self.assertEqual(out, "")
        self.assertIn("L99_typo", err)

    def test_class_filter_with_one_dead_pattern_is_a_harness_error(self):
        code, _, err = run_cli("--engine-cmd", self.reference, "--compare", GOLDEN_DIR, "--class", "fingerprint.sin,litmus.L99*")
        self.assertEqual(code, 2)
        self.assertIn("L99*", err)

    def test_record_refuses_unstable_scripts(self):
        with tempfile.TemporaryDirectory() as temp:
            clone = os.path.join(temp, "batteries")
            shutil.copytree(ROOT, clone, ignore=shutil.ignore_patterns("goldens", "__pycache__"))
            shutil.copy(os.path.join(ROOT, "perturbation", "P01_hrtime_noise.js"), os.path.join(clone, "litmus"))
            lock = os.path.normpath(os.path.join(ROOT, load_index()["reference_lock"]))
            shutil.copy(lock, os.path.join(temp, "oracle.lock"))
            runner = load_runner(os.path.join(clone, "run.py"))
            err = io.StringIO()
            with contextlib.redirect_stderr(err):
                code = runner.main(["--engine-cmd", self.reference, "--record", os.path.join(temp, "g"), "--stability", "3"])
            self.assertEqual(code, 2)
            self.assertIn("instaveis", err.getvalue())
            self.assertFalse(os.path.exists(os.path.join(temp, "g", "golden.json")))

    def test_missing_engine_is_a_harness_error(self):
        code, _, err = run_cli("--engine-cmd", "/nonexistent/engine", "--compare", GOLDEN_DIR)
        self.assertEqual(code, 2)
        self.assertIn("ERRO DO HARNESS", err)

    def test_missing_golden_dir_is_a_harness_error(self):
        code, _, _ = run_cli("--engine-cmd", self.reference, "--compare", "/nonexistent/goldens")
        self.assertEqual(code, 2)

    def test_record_requires_the_reference_tuple(self):
        with tempfile.TemporaryDirectory() as temp:
            code, _, err = run_cli("--engine-cmd", require_env("TYRUS_NODE_SEED"), "--record", os.path.join(temp, "g"), "--stability", "1")
            self.assertEqual(code, 2)
            self.assertIn("difere da referencia", err)

    def test_class_without_golden_is_a_harness_error(self):
        with tempfile.TemporaryDirectory() as temp:
            clone = os.path.join(temp, "g")
            shutil.copytree(GOLDEN_DIR, clone)
            path = os.path.join(clone, "golden.json")
            with open(path, encoding="utf-8") as handle:
                golden = json.load(handle)
            del golden["classes"]["litmus.L01_tick_vs_promise_main"]
            with open(path, "w", encoding="utf-8") as handle:
                json.dump(golden, handle)
            code, _, err = run_cli("--engine-cmd", self.reference, "--compare", clone)
            self.assertEqual(code, 2)
            self.assertIn("desatualizados", err)


class NegativeControlTest(unittest.TestCase):
    """Motores conhecidamente diferentes tem de reprovar nas classes certas."""

    def test_foreign_engines_fail_exactly_where_measured(self):
        with open(os.path.join(HERE, "negative_controls.json"), encoding="utf-8") as handle:
            controls = json.load(handle)
        for name, spec in controls.items():
            with self.subTest(engine=name):
                code, out, err = run_cli("--engine-cmd", require_env(spec["env"]), "--compare", GOLDEN_DIR)
                self.assertEqual(code, 1, err)
                report = json.loads(out)
                self.assertFalse(report["tuple_matches"])
                diff = set(report["diff_classes"])
                self.assertEqual(sorted(set(spec["must_diff"]) - diff), [], "deveria diferir")
                self.assertEqual(sorted(set(spec["must_not_diff"]) & diff), [], "nao deveria diferir")


class MutantEngineTest(unittest.TestCase):
    """Cada mutante e um motor defeituoso conhecido; as baterias tem de mata-lo."""

    def test_every_mutant_is_killed_by_its_declared_classes(self):
        reference = require_env("TYRUS_NODE_REFERENCE")
        with open(os.path.join(ROOT, "mutants", "mutants.json"), encoding="utf-8") as handle:
            mutants = json.load(handle)
        self.assertGreaterEqual(len(mutants), 12)
        for name, spec in mutants.items():
            with self.subTest(mutant=name):
                preload = os.path.join(ROOT, "mutants", spec["preload"])
                code, out, _ = run_cli("--engine-cmd", f"{reference} --require {preload}", "--compare", GOLDEN_DIR, "--class", ",".join(spec["must_diff"]))
                self.assertEqual(code, 1, name)
                self.assertEqual(sorted(set(spec["must_diff"]) - set(json.loads(out)["diff_classes"])), [])

    def test_mutants_json_lists_every_preload_file(self):
        with open(os.path.join(ROOT, "mutants", "mutants.json"), encoding="utf-8") as handle:
            declared = {spec["preload"] for spec in json.load(handle).values()}
        on_disk = {f for f in os.listdir(os.path.join(ROOT, "mutants")) if f.endswith(".js")}
        self.assertEqual(declared, on_disk)


if __name__ == "__main__":
    unittest.main()
