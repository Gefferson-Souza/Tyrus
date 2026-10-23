import json
import os
import shutil
import subprocess
import tempfile
import unittest
from pathlib import Path

HOOK = Path(__file__).resolve().parents[1] / "closure_trace_hook.cjs"
NODE = os.environ.get("TYRUS_NODE_REFERENCE") or shutil.which("node")


@unittest.skipIf(NODE is None, "node ausente (o job closure-e2e exige)")
class ClosureTraceHookTest(unittest.TestCase):
    def setUp(self):
        self.tmp = Path(tempfile.mkdtemp(prefix="tyrus-closure-hook-"))
        self.addCleanup(shutil.rmtree, self.tmp, ignore_errors=True)
        self.root = self.tmp / "tree"
        (self.root / "app").mkdir(parents=True)
        (self.root / "node_modules" / "dep").mkdir(parents=True)
        (self.root / "app" / "main.js").write_text(
            "require('dep'); require('./data.json');"
            "require('fs').readFileSync(__dirname + '/note.txt');"
            "require('fs').readFileSync(process.env.OUTSIDE_FILE);"
            "require('node:path');\n"
        )
        (self.root / "app" / "data.json").write_text("{}")
        (self.root / "app" / "note.txt").write_text("x")
        (self.root / "app" / "extra.txt").write_text("y")
        (self.root / "node_modules" / "dep" / "index.js").write_text("module.exports = 1;\n")
        self.outside = self.tmp / "outside.txt"
        self.outside.write_text("fora")

    def run_node(self, extra_env=None, hook=True):
        trace = self.tmp / "trace.jsonl"
        env = {"PATH": os.environ.get("PATH", ""), "OUTSIDE_FILE": str(self.outside),
               "TYRUS_CLOSURE_ROOT": str(self.root), "TYRUS_CLOSURE_TRACE": str(trace)}
        env.update(extra_env or {})
        args = [NODE] + (["--require", str(HOOK)] if hook else []) + [str(self.root / "app" / "main.js")]
        result = subprocess.run(args, env=env, capture_output=True, text=True, check=False)
        return result, trace

    def entries(self, trace):
        return [json.loads(line) for line in trace.read_text().splitlines()]

    def test_records_loaded_modules_relative_to_the_root(self):
        result, trace = self.run_node()
        self.assertEqual(result.returncode, 0, result.stderr)
        loads = {e["path"] for e in self.entries(trace) if e["kind"] == "load"}
        self.assertEqual(loads, {"app/main.js", "app/data.json", "node_modules/dep/index.js"})

    def test_records_reads_inside_the_root(self):
        _, trace = self.run_node()
        reads = {e["path"] for e in self.entries(trace) if e["kind"] == "read"}
        self.assertIn("app/note.txt", reads)

    def test_files_outside_the_root_are_external_with_absolute_path(self):
        _, trace = self.run_node()
        external = [e["path"] for e in self.entries(trace) if e["kind"] == "external"]
        self.assertIn(str(self.outside), external)

    def test_each_file_is_recorded_once_per_kind(self):
        _, trace = self.run_node()
        pairs = [(e["kind"], e["path"]) for e in self.entries(trace)]
        self.assertEqual(len(pairs), len(set(pairs)))

    def test_never_records_the_trace_file_itself(self):
        _, trace = self.run_node()
        self.assertFalse([e for e in self.entries(trace) if e["path"].endswith("trace.jsonl")])

    def run_script(self, source):
        (self.root / "app" / "main.js").write_text(source)
        return self.run_node()

    def test_missing_file_probe_is_not_recorded_as_read(self):
        result, trace = self.run_script(
            "try { require('fs').readFileSync(__dirname + '/nao-existe.json'); } catch (e) {}\n"
            "require('fs').readFileSync(__dirname + '/note.txt');\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        reads = {e["path"] for e in self.entries(trace) if e["kind"] == "read"}
        self.assertEqual(reads - {"app/main.js"}, {"app/note.txt"})

    def test_async_reads_and_streams_are_recorded(self):
        result, trace = self.run_script(
            "const fs = require('fs');\n"
            "fs.readFile(__dirname + '/note.txt', () => {\n"
            "  fs.promises.readFile(__dirname + '/data.json').then(() => {\n"
            "    fs.createReadStream(__dirname + '/extra.txt').on('data', () => {});\n"
            "  });\n"
            "});\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        reads = {e["path"] for e in self.entries(trace) if e["kind"] == "read"}
        self.assertEqual(reads - {"app/main.js"}, {"app/note.txt", "app/data.json", "app/extra.txt"})

    def test_package_json_consulted_by_resolution_is_recorded(self):
        pkg = self.root / "node_modules" / "dep"
        (pkg / "package.json").write_text('{"name":"dep","main":"lib/entry.js"}')
        (pkg / "lib").mkdir()
        (pkg / "lib" / "entry.js").write_text("module.exports = 1;\n")
        result, trace = self.run_script("require('dep');\n")
        self.assertEqual(result.returncode, 0, result.stderr)
        reads = {e["path"] for e in self.entries(trace) if e["kind"] == "read"}
        self.assertIn("node_modules/dep/package.json", reads)

    def test_refuses_to_start_without_configuration(self):
        result, _ = self.run_node({"TYRUS_CLOSURE_TRACE": ""})
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("TYRUS_CLOSURE_TRACE", result.stderr)

    def test_without_the_hook_nothing_is_written(self):
        _, trace = self.run_node(hook=False)
        self.assertFalse(trace.exists())


if __name__ == "__main__":
    unittest.main()
