#!/usr/bin/env python3
"""Fail when a relative import points at a module that does not exist.

Real breakage this would have caught (both only failed at startup/request time,
so nothing in CI noticed because the service was never imported):

  src/api/v1/endpoints/scoring.py:  from .engine import ...        (lives in services/scoring)
  src/api/v1/endpoints/voice.py:    from ...core.config import ... (one level short)

Run with: python3 scripts/check_imports.py
"""

import ast
import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent


def module_exists(base: pathlib.Path, parts: list[str]) -> bool:
    candidate = base.joinpath(*parts) if parts else base
    return candidate.with_suffix(".py").exists() or (candidate / "__init__.py").exists()


def check(path: pathlib.Path) -> list[str]:
    problems: list[str] = []
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    for node in ast.walk(tree):
        if not isinstance(node, ast.ImportFrom) or not node.level:
            continue
        base = path.parent
        for _ in range(node.level - 1):
            base = base.parent
        parts = node.module.split(".") if node.module else []
        if not module_exists(base, parts):
            dotted = "." * node.level + (node.module or "")
            names = ", ".join(alias.name for alias in node.names)
            problems.append(
                f"{path.relative_to(ROOT)}:{node.lineno}: from {dotted} import {names}"
            )
    return problems


def main() -> int:
    failures: list[str] = []
    for path in sorted(ROOT.rglob("*.py")):
        if ".venv" in path.parts:
            continue
        failures.extend(check(path))
    if failures:
        print("broken relative imports:", file=sys.stderr)
        for failure in failures:
            print(f"  {failure}", file=sys.stderr)
        return 1
    print("relative imports resolve")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
