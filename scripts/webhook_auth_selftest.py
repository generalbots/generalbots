#!/usr/bin/env python3
"""Self-test for scripts/check_webhook_auth.py (#2142/#2546).

Kept in a helper so the guard itself stays under the 450-line project limit.
The caller injects `Webhook` and `scan` to avoid a circular import.
"""
from __future__ import annotations

import sys
import tempfile
from pathlib import Path


def self_test(Webhook, scan) -> int:
    """Prove the verification rules fire and retired 410 routes need no entry."""
    failures: list[str] = []

    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        crate = root / "botserver/crates/botdemo/src"
        crate.mkdir(parents=True)

        (crate / "webhook.rs").write_text(
            'pub fn configure() {\n'
            '    Router::new()\n'
            '        .route("/webhook/demo", post(handle_demo))\n'
            '        .route("/webhook/open", post(handle_open))\n'
            '        .route("/webhook/new", post(handle_new))\n'
            '}\n'
            'async fn handle_demo() { authorize_delivery("s", None); }\n'
            'async fn handle_open() { /* no gate */ }\n'
            'async fn handle_new() { authorize_delivery("s", None); }\n',
            encoding="utf-8",
        )
        (crate / "bad.rs").write_text(
            'pub fn authorize_delivery(secret: &str) -> bool {\n'
            '    if secret.is_empty() {\n'
            '        return true;\n'
            '    }\n'
            '    false\n'
            '}\n',
            encoding="utf-8",
        )

        allowlist = root / "allowlist.txt"
        allowlist.write_text("/webhook/open # demo: authenticated elsewhere\n", encoding="utf-8")

        # The demo crate has no INVENTORY entry, so every route is unclassified
        # too; the assertions below only look at the rule under test.
        inventory = (
            Webhook("demo", "botdemo", ("/webhook/demo", "/webhook/open", "/webhook/new")),
        )
        violations, _ = scan(root, Path("allowlist.txt"), inventory)

        rules = {violation.rule for violation in violations}
        if "empty-accept" not in rules:
            failures.append("an empty-secret accept branch was not detected")
        if "unverified" in rules:
            failures.append("the allowlisted handler was reported")
        if "unclassified" in rules:
            failures.append("an inventory route was reported as unclassified")

        allowlist.unlink()
        violations, _ = scan(root, Path("allowlist.txt"), inventory)
        if "unverified" not in {violation.rule for violation in violations}:
            failures.append("an unverified handler was not detected")

        allowlist.write_text("/webhook/gone # stale\n", encoding="utf-8")
        violations, _ = scan(root, Path("allowlist.txt"), inventory)
        if "stale-allowlist" not in {violation.rule for violation in violations}:
            failures.append("a stale allowlist entry was not detected")

        # #2540 — a `gone_handler!`-generated route fails closed with 410 and
        # must be recognised rather than flagged as an unreadable handler.
        (crate / "gone.rs").write_text(
            'macro_rules! gone_handler { ($name:ident, $to:literal) => {\n'
            '    async fn $name() {}\n'
            '} }\n'
            'gone_handler!(old_delivery, "/api/new");\n'
            'pub fn configure() { Router::new()\n'
            '    .route("/webhook/gone", post(old_delivery))\n'
            '}\n',
            encoding="utf-8",
        )
        gone_inventory = (
            Webhook("demo", "botdemo", ("/webhook/demo", "/webhook/open", "/webhook/new",
                                        "/webhook/gone")),
        )
        allowlist.write_text("/webhook/open # demo: authenticated elsewhere\n", encoding="utf-8")
        gone_rules = {v.rule for v in scan(root, Path("allowlist.txt"), gone_inventory)[0]}
        if "unknown-handler" in gone_rules:
            failures.append("a gone_handler! route was reported as an unknown handler")
        if "unverified" in gone_rules:
            failures.append("a gone_handler! route was reported as unverified")
        # #2546 — a retired 410 route needs no INVENTORY entry at all.
        retired_rules = {v.rule for v in scan(root, Path("allowlist.txt"), inventory)[0]}
        if "unclassified" in retired_rules or "unknown-handler" in retired_rules:
            failures.append("a retired gone_handler! route was reported without an entry")
        if "stale-inventory" in retired_rules:
            failures.append("a retired route without an entry was reported as stale inventory")

        partial = (Webhook("demo", "botdemo", ("/webhook/demo", "/webhook/open")),)
        violations, _ = scan(root, Path("allowlist.txt"), partial)
        if "unclassified" not in {violation.rule for violation in violations}:
            failures.append("a route missing from INVENTORY was not detected")
        if "stale-inventory" in {violation.rule for violation in violations}:
            failures.append("a registered route was reported as stale inventory")

    if failures:
        for failure in failures:
            print(f"self-test FAILED: {failure}", file=sys.stderr)
        return 1

    print("self-test: ok")
    return 0
