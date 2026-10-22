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
                delays = [int(d) for d in re.findall(r"set(?:Timeout|Interval)\([^;]*?,\s*(\d+)\)", handle.read())]
            self.assertTrue(all(d < 10 for d in delays), f"{name}: {delays}")

    def test_no_litmus_flutters_in_five_runs(self):
        classes, _ = RUN.script_classes(self.reference, os.path.join(ROOT, "litmus"), "litmus", self.env, RUN.Selection(5, None))
        self.assertEqual([c for c, v in classes.items() if v.get("unstable")], [])
        self.assertEqual(len(classes), len(self.litmus))


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


if __name__ == "__main__":
    unittest.main()
