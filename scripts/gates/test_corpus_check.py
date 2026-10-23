import hashlib
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import corpus_check  # noqa: E402


class CorpusCheckTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self.tmp.name)
        self.seed = self.repo / "oracle" / "corpus" / "seed"
        (self.seed / "probes").mkdir(parents=True)
        (self.seed / "probes" / "a.ts").write_bytes(b"console.log(1);\n")
        (self.seed / "probes" / "b.ts").write_bytes(b"console.log(2);\n")
        corpus_check.write_lock(self.repo)

    def tearDown(self):
        self.tmp.cleanup()

    def test_valid_lock_passes(self):
        self.assertEqual(corpus_check.check(self.repo), [])

    def test_lock_is_sha256sum_compatible_and_sorted(self):
        lines = (self.seed / "corpus.lock").read_text().splitlines()
        digest = hashlib.sha256(b"console.log(1);\n").hexdigest()
        self.assertEqual(lines[0], f"{digest}  oracle/corpus/seed/probes/a.ts")
        self.assertEqual(lines, sorted(lines, key=lambda l: l.split("  ", 1)[1]))

    def test_changed_byte_fails(self):
        (self.seed / "probes" / "a.ts").write_bytes(b"console.log(9);\n")
        errors = corpus_check.check(self.repo)
        self.assertEqual(len(errors), 1)
        self.assertIn("a.ts", errors[0])

    def test_extra_file_fails(self):
        (self.seed / "probes" / "c.ts").write_bytes(b"x\n")
        self.assertIn("c.ts", " ".join(corpus_check.check(self.repo)))

    def test_missing_file_fails(self):
        (self.seed / "probes" / "b.ts").unlink()
        self.assertIn("b.ts", " ".join(corpus_check.check(self.repo)))

    def test_symlink_fails(self):
        (self.seed / "probes" / "l.ts").symlink_to("a.ts")
        self.assertIn("l.ts", " ".join(corpus_check.check(self.repo)))

    def test_duplicate_lock_entry_fails(self):
        lock = self.seed / "corpus.lock"
        first = lock.read_text().splitlines()[0]
        lock.write_text(lock.read_text() + first + "\n")
        self.assertNotEqual(corpus_check.check(self.repo), [])

    def test_write_is_deterministic(self):
        before = (self.seed / "corpus.lock").read_bytes()
        corpus_check.write_lock(self.repo)
        self.assertEqual((self.seed / "corpus.lock").read_bytes(), before)

    def test_cli_exit_codes(self):
        cmd = [sys.executable, "-B", "-I", str(HERE / "corpus_check.py"), "--root", str(self.repo)]
        self.assertEqual(subprocess.run(cmd, capture_output=True).returncode, 0)
        (self.seed / "probes" / "a.ts").write_bytes(b"changed\n")
        self.assertNotEqual(subprocess.run(cmd, capture_output=True).returncode, 0)


if __name__ == "__main__":
    unittest.main()
