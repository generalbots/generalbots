#!/usr/bin/env python3
"""The production-readiness gate registry (#2121).

`scripts/prod_readiness.py` is the runner; this module is the data. Keeping the
list separate is what makes the gate and the BotBook scoreboard impossible to
drift: the table, the JSON and the page are all rendered from `GATES` and
`CHILDREN`, and adding a check is one entry here.

Every gate carries the issue that owns it. A gate that does not exist yet is
declared with `evidence` (a file the owning issue lands) or with `steps` that
name a script: the runner reports SKIP with the expected path instead of hiding
the gap.

Field meanings
--------------
`steps`        commands run in order; the first non-zero exit fails the gate.
`requires`     paths that must exist before the steps run (missing -> SKIP).
`evidence`     `(glob, marker)` pairs proving a gate is installed: the glob must
               match `min_files` files and each file must contain its marker
               (an empty marker means "the file exists"). Unmatched -> SKIP.
`ok_detail`    the stable sentence the scoreboard shows when the tool is quiet.
`fixed_detail` ignore the tool's stdout (its counts would churn the page).
"""

from __future__ import annotations

from dataclasses import dataclass

EPIC = 2121

PASS, FAIL, SKIP = "PASS", "FAIL", "SKIP"

# Markers that prove the Security workflow still runs every scan. Used as the
# offline fallback when the gitleaks binary is not installed in this
# environment (CI installs it in `security.yml`, not here).
SECURITY_SCANS = ("cargo-deny", "osv-scanner", "gitleaks", "pip-audit", "syft", "actionlint")


@dataclass(frozen=True)
class Gate:
    """One readiness check."""

    id: str
    title: str
    owner: int
    proves: str
    steps: tuple[tuple[str, ...], ...] = ()
    requires: tuple[str, ...] = ()
    evidence: tuple[tuple[str, str], ...] = ()
    min_files: int = 1
    ok_detail: str = ""
    fixed_detail: bool = False


@dataclass(frozen=True)
class Result:
    gate: Gate
    status: str
    detail: str


GATES: tuple[Gate, ...] = (
    Gate(
        id="security-auth",
        title="JWT claims are signature-verified before they pick a tenant",
        owner=2112,
        proves="A forged/`alg=none` token cannot select another tenant's branch",
        evidence=(("botserver/crates/botsecurity-core/src/scope.rs", "fn a_forged_signature_cannot_select_a_branch"),),
        ok_detail="regression test present (executed by the unit-tests job)",
    ),
    Gate(
        id="security-webhooks",
        title="WhatsApp/Teams inbound requests prove their origin",
        owner=2113,
        proves="An unauthenticated POST cannot create a bot turn",
        evidence=(
            ("botserver/crates/botwhatsapp/src/utils_tests.rs", "fn a_published_vector_verifies"),
            ("botserver/crates/botmsteams/src/auth/tests.rs", "async fn a_tampered_body_is_rejected"),
        ),
        ok_detail="regression tests present (executed by the unit-tests job)",
    ),
    Gate(
        id="security-mfa",
        title="TOTP enrollment and verification share one secret encoding",
        owner=2114,
        proves="A code from the QR secret is accepted by the server",
        evidence=(("botserver/crates/botsecurity-auth/src/mfa/totp_tests.rs", "fn a_code_from_the_qr_secret_verifies_on_the_server"),),
        ok_detail="regression test present (executed by the unit-tests job)",
    ),
    Gate(
        id="slo-check",
        title="SLOs are defined, instrumented and gated",
        owner=2115,
        proves="A latency/error/cost regression fails instead of shipping silently",
        steps=(
            ("python3", "scripts/check_slo.py", "--self-test"),
            ("python3", "scripts/check_slo.py", "--snapshot", "perf/baselines/local-mock.json"),
        ),
        requires=("ops/slo.yaml", "perf/baselines/local-mock.json"),
    ),
    Gate(
        id="ops-runbooks",
        title="Operational runbooks exist for the on-call surface",
        owner=2116,
        proves="Analytics console, error budgets and runbooks are operable, not prose",
        evidence=(("botbook/src/10-configuration-deployment/operations/runbook-*.md", ""),),
        min_files=6,
        ok_detail="six runbooks published",
    ),
    Gate(
        id="data-restore",
        title="Restore drill proves migrations, tenancy and recovery together",
        owner=2117,
        proves="A production-like restore boots and stays tenant-isolated",
        evidence=(
            ("bottest/tests/integration/data_integrity/restore.rs", ""),
            ("bottest/tests/integration/data_integrity/isolation.rs", ""),
        ),
        ok_detail="restore + tenant-isolation drill present (data-integrity job)",
    ),
    Gate(
        id="release-version",
        title="One version everywhere, and the tag is applied to the build",
        owner=2118,
        proves="A tag and `/api/product.version` cannot disagree",
        steps=(("python3", "scripts/release_version.py", "--check"), ("python3", "scripts/release_version.py", "selftest")),
    ),
    Gate(
        id="test-guards",
        title="Unit-suite policy: registered modules, reasons for skips, offline",
        owner=2119,
        proves="The offline suite stays hermetic and reviewable",
        steps=(("python3", "scripts/test_guards.py", "--check"), ("python3", "scripts/test_guards.py", "--self-test")),
    ),
    Gate(
        id="app-contracts",
        title="Every registry app declares its classification, smoke and API surface",
        owner=2119,
        proves="A shipped app is documented and an invented route fails the build",
        steps=(("python3", "scripts/smoke_app_surfaces.py", "--contracts"),),
        ok_detail="every registry app classified, smoked and documented",
        fixed_detail=True,
    ),
    Gate(
        id="flake-policy",
        title="Flaky tests are registered, owned and inside the quarantine SLA",
        owner=2119,
        proves="A flake is time-boxed instead of retried until it passes",
        steps=(
            ("python3", "scripts/flake_report.py", "--self-test"),
            ("python3", "scripts/flake_report.py", "--quarantine-only"),
        ),
        ok_detail="no flake outside its SLA",
        fixed_detail=True,
    ),
    Gate(
        id="coverage-ratchet",
        title="Coverage ratchet against a committed baseline",
        owner=2119,
        proves="A PR cannot lower the covered lines of the crate it touches",
        steps=(("python3", "scripts/check_coverage.py", "--self-test"),),
        requires=("issues/coverage/baseline.json",),
        ok_detail="baseline committed; the ratchet self-test fails on a decrease",
        fixed_detail=True,
    ),
    Gate(
        id="product-profile",
        title="Every app is classified always/preview/hidden and its feature compiles",
        owner=2120,
        proves="The supported surface is bounded and owned",
        steps=(
            ("python3", "scripts/product_profile_check.py"),
            ("python3", "scripts/app_complexity_budget.py"),
        ),
        ok_detail="every app classified, owned and within the complexity budget",
        fixed_detail=True,
    ),
    Gate(
        id="security-scan",
        title="Supply-chain and secret scans are wired in CI",
        owner=EPIC,
        proves="cargo-deny, OSV, gitleaks, pip-audit, SBOM and actionlint still run",
        requires=(".gitleaks.toml", "deny.toml", "scripts/osv_allowlist.txt"),
    ),
    Gate(
        id="adequacy-guard",
        title="AGENTS.md structural rules stay at or below the baseline",
        owner=1735,
        proves="No new `unwrap`, `table!` outside botschema, 450+ line file or CDN asset",
        steps=(("python3", "scripts/adequacy_guard.py", "--baseline", "issues/adequacy/baseline.json"),),
    ),
    Gate(
        id="i18n-lint",
        title="Fluent catalogs and the keys the UI asks for are consistent",
        owner=2090,
        proves="A new user-facing string cannot ship untranslated or unbalanced",
        steps=(("python3", "scripts/i18n_lint.py", "--check"),),
        ok_detail="no new catalog violation",
        fixed_detail=True,
    ),
    Gate(
        id="workflow-pinning",
        title="Every `uses:` is a commit SHA and no workflow trusts a fork",
        owner=1726,
        proves="A retargeted action tag cannot execute with the repository token",
        steps=(("python3", "scripts/check_workflow_pinning.py"),),
        ok_detail="every `uses:` is a commit SHA; no workflow trusts a fork",
        fixed_detail=True,
    ),
)

# Child issue -> the gate(s) that prove it. The scoreboard's second table is
# rendered from this map, so an issue cannot be listed without a gate.
CHILDREN: tuple[tuple[int, str, tuple[str, ...]], ...] = (
    (2112, "security(auth): JWT signature", ("security-auth",)),
    (2113, "security(webhooks): WhatsApp/Teams auth", ("security-webhooks",)),
    (2114, "security(mfa): TOTP encoding", ("security-mfa",)),
    (2115, "perf(slo): SLOs and budgets", ("slo-check",)),
    (2116, "ops(analytics): console, budgets, runbooks", ("ops-runbooks",)),
    (2117, "data(integrity): migrations, restore, tenancy", ("data-restore",)),
    (2118, "release(auto-bump): one version", ("release-version",)),
    (2119, "test(quality): ratchet, contracts, fuzzing", ("test-guards", "app-contracts", "coverage-ratchet", "flake-policy")),
    (2120, "product(focus): supported matrix", ("product-profile",)),
)
