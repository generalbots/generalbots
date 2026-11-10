#!/usr/bin/env python3
"""IMP-09 security-header gate.

AGENTS.md requires six headers on *every* response of *every* server:

    Content-Security-Policy, Strict-Transport-Security, X-Frame-Options,
    X-Content-Type-Options: nosniff, Referrer-Policy, Permissions-Policy

The layer that emits them lives in `botsecurity_core::headers`
(`with_security_headers`). It is a plain tower layer, which means a new
`axum::serve` call or a router factory that forgets it compiles cleanly and
ships a bare server. Review does not catch that reliably, so this gate does:

R1 serve-coverage   every `axum::serve(` site must serve a hardened router:
                    the file calls `with_security_headers`, the served
                    expression is a `configure_*_router()` factory (hardened
                    inside), or the file is listed in the allowlist with a
                    reason. Test files are skipped.
R2 api-stack        `apply_middleware` (the botserver layer stack) must end in
                    `with_security_headers`, i.e. the header layer is the
                    outermost one and therefore covers 401/403/429/500.
R3 header-set       the shared applier must still emit all six header names.
R4 profile-intact   no `SecurityHeadersConfig` profile may set a required
                    header to `None` (a profile may widen a policy, never drop
                    the header).

Usage:
    python3 scripts/check_security_headers.py --check
    python3 scripts/check_security_headers.py --self-test

Exit codes: 0 = clean, 1 = violations, 2 = usage error.
"""

from __future__ import annotations

import argparse
import re
import sys
import tempfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
ALLOWLIST = Path("scripts/security_headers_allowlist.txt")
SCAN_DIRS = ("botserver/src", "botui/src")
HEADERS_DIR = Path("botserver/crates/botsecurity-core/src/headers")
LAYER_CALL = "with_security_headers("
HARDENED_FACTORY = re.compile(r"configure_[a-z_]*router\s*\(")
TEST_MARKERS = ("/tests.rs", "_tests.rs", "/tests/", "/test/")

REQUIRED_HEADERS = (
    "content-security-policy",
    "strict-transport-security",
    "x-frame-options",
    "x-content-type-options",
    "referrer-policy",
    "permissions-policy",
)

PROFILE_NONE = re.compile(
    r"\b(content_security_policy|strict_transport_security|x_frame_options"
    r"|x_content_type_options|referrer_policy|permissions_policy)\s*:\s*None"
)


class Violation:
    def __init__(self, path: Path, line: int, rule: str, message: str) -> None:
        self.path = path
        self.line = line
        self.rule = rule
        self.message = message

    def render(self, root: Path) -> str:
        try:
            shown = self.path.relative_to(root)
        except ValueError:
            shown = self.path
        return f"{shown}:{self.line}: {self.rule}: {self.message}"


def load_allowlist(root: Path) -> dict[str, str]:
    """`<path> # reason` lines. A reason is mandatory."""
    allow: dict[str, str] = {}
    path = root / ALLOWLIST
    if not path.is_file():
        return allow
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.strip()
        if not line or line.startswith("#"):
            continue
        if "#" not in line:
            raise SystemExit(f"{ALLOWLIST}:{raw!r} has no reason comment")
        entry, reason = line.split("#", 1)
        allow[entry.strip()] = reason.strip()
    return allow


def is_test_file(path: Path) -> bool:
    text = path.as_posix()
    return any(marker in text for marker in TEST_MARKERS)


def rust_sources(root: Path) -> list[Path]:
    files: list[Path] = []
    for scan in SCAN_DIRS:
        base = root / scan
        if base.is_dir():
            files.extend(sorted(base.rglob("*.rs")))
    return files


def function_body(lines: list[str], start: int) -> str:
    """Text of the function whose `fn` keyword is on line `start`."""
    depth = 0
    seen_brace = False
    body: list[str] = []
    for line in lines[start:]:
        body.append(line)
        for char in line:
            if char == "{":
                depth += 1
                seen_brace = True
            elif char == "}":
                depth -= 1
        if seen_brace and depth <= 0:
            break
    return "\n".join(body)


def hardened_variables(text: str) -> set[str]:
    """`let app = configure_router();` — routers already hardened by a factory."""
    pattern = re.compile(
        r"let\s+([A-Za-z_]\w*)\s*(?::[^=]+)?=\s*[^;]*?configure_[a-z_]*router\s*\("
    )
    return set(pattern.findall(text))


def check_serve_coverage(root: Path, allow: dict[str, str]) -> list[Violation]:
    violations: list[Violation] = []
    for path in rust_sources(root):
        if is_test_file(path):
            continue
        rel = path.relative_to(root).as_posix()
        text = path.read_text(encoding="utf-8", errors="replace")
        if "axum::serve(" not in text:
            continue
        if rel in allow:
            continue
        lines = text.splitlines()
        hardened_file = LAYER_CALL in text
        factory_vars = hardened_variables(text)
        for number, line in enumerate(lines, start=1):
            if "axum::serve(" not in line or line.lstrip().startswith("//"):
                continue
            # The served expression may sit on this line or on the next few.
            window = "\n".join(lines[number - 1 : number + 4])
            if hardened_file or HARDENED_FACTORY.search(window):
                continue
            if any(re.search(rf"\b{re.escape(name)}\b", window) for name in factory_vars):
                continue
            violations.append(
                Violation(
                    path,
                    number,
                    "serve-coverage",
                    "axum::serve without the IMP-09 header layer; wrap the router "
                    f"in {LAYER_CALL}...) or add a reasoned allowlist entry",
                )
            )
    return violations


def check_api_stack(root: Path) -> list[Violation]:
    violations: list[Violation] = []
    for path in rust_sources(root):
        if is_test_file(path):
            continue
        lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
        for index, line in enumerate(lines):
            if re.search(r"\bfn\s+apply_middleware\b", line) is None:
                continue
            body = function_body(lines, index)
            if LAYER_CALL not in body:
                violations.append(
                    Violation(
                        path,
                        index + 1,
                        "api-stack",
                        "apply_middleware must end in with_security_headers(...) so the "
                        "layer is outermost and covers 401/403/429/500",
                    )
                )
    return violations


def check_header_set(root: Path) -> list[Violation]:
    base = root / HEADERS_DIR
    if not base.is_dir():
        return [Violation(base, 1, "header-set", "shared header layer is missing")]
    blob = "\n".join(
        path.read_text(encoding="utf-8", errors="replace").lower()
        for path in sorted(base.rglob("*.rs"))
    )
    violations: list[Violation] = []
    for header in REQUIRED_HEADERS:
        if header not in blob:
            violations.append(
                Violation(
                    base,
                    1,
                    "header-set",
                    f"the shared layer no longer mentions {header}",
                )
            )
    return violations


def check_profiles(root: Path) -> list[Violation]:
    base = root / HEADERS_DIR
    violations: list[Violation] = []
    if not base.is_dir():
        return violations
    for path in sorted(base.rglob("*.rs")):
        if path.name in {"tests.rs"} or path.name.endswith("_tests.rs"):
            continue
        for number, line in enumerate(
            path.read_text(encoding="utf-8", errors="replace").splitlines(), start=1
        ):
            stripped = line.strip()
            if stripped.startswith("//"):
                continue
            match = PROFILE_NONE.search(line)
            if match:
                violations.append(
                    Violation(
                        path,
                        number,
                        "profile-intact",
                        f"{match.group(1)} is None: a profile may widen the policy but "
                        "never drop a required header",
                    )
                )
    return violations


def run_checks(root: Path) -> list[Violation]:
    allow = load_allowlist(root)
    violations: list[Violation] = []
    violations.extend(check_serve_coverage(root, allow))
    violations.extend(check_api_stack(root))
    violations.extend(check_header_set(root))
    violations.extend(check_profiles(root))
    return violations


def self_test() -> int:
    """Proves each rule fires and stays quiet on a clean tree."""
    failures: list[str] = []
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        headers = root / HEADERS_DIR
        headers.mkdir(parents=True)
        (root / "scripts").mkdir(parents=True)
        (root / ALLOWLIST).write_text("# empty\n", encoding="utf-8")
        (headers / "strict.rs").write_text(
            "\n".join(f'"{name}"' for name in REQUIRED_HEADERS)
            + '\npub fn security_headers_middleware() {}\n',
            encoding="utf-8",
        )

        server = root / "botui/src"
        server.mkdir(parents=True)
        clean = server / "clean.rs"
        clean.write_text(
            "pub fn configure_router() -> Router {\n"
            "    with_security_headers(Router::new(), config)\n"
            "}\n"
            "pub async fn main() { axum::serve(listener, configure_router()).await; }\n",
            encoding="utf-8",
        )
        if run_checks(root):
            failures.append("clean tree reported violations")

        bad = server / "bad.rs"
        bad.write_text(
            "pub async fn main() {\n    axum::serve(listener, router).await;\n}\n",
            encoding="utf-8",
        )
        rules = {v.rule for v in run_checks(root)}
        if "serve-coverage" not in rules:
            failures.append("serve-coverage did not fire on a bare axum::serve")

        clean.unlink()
        bad.write_text(
            "pub async fn main() { axum::serve(listener, configure_router()).await; }\n",
            encoding="utf-8",
        )
        if run_checks(root):
            failures.append("hardened factory reported as a violation")

        bad.write_text(
            "fn apply_middleware(app: Router) -> Router {\n    app\n}\n",
            encoding="utf-8",
        )
        rules = {v.rule for v in run_checks(root)}
        if "api-stack" not in rules:
            failures.append("api-stack did not fire on a stack without the layer")
        bad.unlink()

        (headers / "strict.rs").write_text(
            'pub fn f() { "x-content-type-options"; }\n', encoding="utf-8"
        )
        rules = {v.rule for v in run_checks(root)}
        if "header-set" not in rules:
            failures.append("header-set did not fire on a missing header")
        (headers / "strict.rs").write_text(
            "\n".join(f'"{name}"' for name in REQUIRED_HEADERS), encoding="utf-8"
        )

        (headers / "strict.rs").write_text(
            "\n".join(f'"{name}"' for name in REQUIRED_HEADERS)
            + "\n            permissions_policy: None,\n",
            encoding="utf-8",
        )
        rules = {v.rule for v in run_checks(root)}
        if "profile-intact" not in rules:
            failures.append("profile-intact did not fire on a dropped header")

    for failure in failures:
        print(f"self-test FAILED: {failure}", file=sys.stderr)
    if failures:
        return 1
    print("self-test OK: serve-coverage, api-stack, header-set, profile-intact")
    return 0


def main() -> int:
    parser = argparse.ArgumentParser(description="IMP-09 security header gate")
    parser.add_argument("--check", action="store_true", help="scan the repository")
    parser.add_argument("--self-test", action="store_true", help="prove the rules fire")
    parser.add_argument("--root", default=str(ROOT), help="repository root")
    args = parser.parse_args()

    if not args.check and not args.self_test:
        parser.error("pass --check and/or --self-test")

    status = 0
    if args.self_test:
        status |= self_test()
    if args.check:
        root = Path(args.root).resolve()
        violations = run_checks(root)
        for violation in violations:
            print(violation.render(root))
        if violations:
            print(f"\n{len(violations)} IMP-09 violation(s)", file=sys.stderr)
            status = 1
        else:
            print(
                "security headers OK: every axum::serve is layered, the API stack "
                "is layered outermost, all six headers are emitted"
            )
    return status


if __name__ == "__main__":
    sys.exit(main())
