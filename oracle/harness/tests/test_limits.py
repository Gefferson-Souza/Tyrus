import ast
import unittest
from pathlib import Path

HARNESS = Path(__file__).resolve().parents[1]
MAX_FILE_LINES = 400
MAX_FUNCTION_LINES = 50
MAX_PARAMETERS = 5


def sources() -> list[Path]:
    return sorted(p for p in HARNESS.rglob("*.py") if "__pycache__" not in p.parts)


class LimitsTest(unittest.TestCase):
    def test_files_stay_within_four_hundred_lines(self):
        for path in sources():
            lines = len(path.read_text(encoding="utf-8").splitlines())
            self.assertLessEqual(lines, MAX_FILE_LINES, f"{path.name}: {lines} linhas")

    def test_functions_stay_within_fifty_lines_and_five_parameters(self):
        for path in sources():
            for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
                if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
                    length = node.end_lineno - node.lineno + 1
                    params = len(node.args.args) + len(node.args.kwonlyargs) - (1 if node.args.args and node.args.args[0].arg in ("self", "cls") else 0)
                    self.assertLessEqual(length, MAX_FUNCTION_LINES, f"{path.name}:{node.name}: {length} linhas")
                    self.assertLessEqual(params, MAX_PARAMETERS, f"{path.name}:{node.name}: {params} parâmetros")


if __name__ == "__main__":
    unittest.main()
