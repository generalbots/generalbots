#!/usr/bin/env python3
"""Assertions behind the restore drill (#2117).

`scripts/restore_drill.sh` restores a `pg_dump -Fc` into a throwaway database
and asks this module whether the copy can be trusted. It answers three
questions, all from the database itself, never from prose:

* **Did the schema survive?** every core table exists and
  `__diesel_schema_migrations` is non-empty, so a restore that silently lost
  the migration history cannot pass.
* **Did the rows survive?** per-table row counts of the restored copy are
  compared with a manifest taken from the source database before the dump.
* **Is the copy still tenant-isolated?** for every table that carries
  `org_id`/`branch_id`/`bot_id`/`tenant_id`, generated probes count rows that
  point at another tenant's organization, branch or bot. Every probe must
  return zero rows; one non-zero probe fails the drill.

`--self-test` feeds hand-written observations (clean, short table, cross-tenant
row, missing table, empty migration history) through the very same comparison
code, so the assertions are proven offline, without a PostgreSQL server.

Exit codes: 0 pass, 1 mismatch, 2 environment or usage error.
"""

from __future__ import annotations

import argparse
import json
import re
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

CORE_TABLES = ("organizations", "branches", "bots", "users")
SCOPE_COLUMNS = ("org_id", "branch_id", "bot_id", "tenant_id")
# `Uuid::nil()` is the documented global SaaS scope (`branch_id = nil`), not a
# tenant: rows that carry it are excluded from the orphan/cross-tenant probes so
# a legitimate global catalog row is not reported as an isolation failure.
GLOBAL_SCOPE = "00000000-0000-0000-0000-000000000000"
MIGRATIONS_TABLE = "__diesel_schema_migrations"
MAX_DETAILS = 8
TIMEOUT = 900


class DrillError(RuntimeError):
    """A step could not run: missing tool, unreachable server, unreadable file."""


def last_line(text: str) -> str:
    """The most useful line of a psql error, without the trailing noise."""
    lines = [line.strip() for line in text.splitlines() if line.strip()]
    return lines[-1] if lines else ""


def mask(dsn: str) -> str:
    """Drop the userinfo from a DSN so credentials never reach a log."""
    return re.sub(r"^([a-zA-Z][a-zA-Z0-9+.\-]*)://[^@/]*@", r"\1://***@", dsn)


def swap_database(dsn: str, name: str) -> str:
    """Point a connection URL at another database, keeping credentials and params."""
    base, separator, query = dsn.partition("?")
    cut = base.rfind("/")
    if cut < 0:
        raise DrillError(f"cannot find the database name in {mask(dsn)}")
    swapped = f"{base[:cut]}/{name}"
    return f"{swapped}?{query}" if separator else swapped


def psql(dsn: str, sql: str) -> list[list[str]]:
    """Run one statement and return tab-separated rows."""
    try:
        proc = subprocess.run(
            ["psql", dsn, "-tA", "-F", "\t", "-v", "ON_ERROR_STOP=1", "-q", "-c", sql],
            capture_output=True,
            text=True,
            timeout=TIMEOUT,
        )
    except FileNotFoundError as error:
        raise DrillError(f"psql is not installed: {error}") from error
    except subprocess.TimeoutExpired as error:
        raise DrillError(f"psql timed out after {TIMEOUT}s") from error
    if proc.returncode != 0:
        raise DrillError(f"psql failed: {last_line(proc.stderr) or last_line(proc.stdout)}")
    return [line.split("\t") for line in proc.stdout.splitlines() if line]


def quote(identifier: str) -> str:
    """Quote an identifier discovered in the catalog."""
    return '"' + identifier.replace('"', '""') + '"'


def base_tables(dsn: str) -> list[str]:
    rows = psql(
        dsn,
        "select table_name from information_schema.tables "
        "where table_schema = 'public' and table_type = 'BASE TABLE' order by table_name",
    )
    return [row[0] for row in rows if row]


def column_inventory(dsn: str) -> dict[str, set[str]]:
    """Every tenant-scope column, grouped by the table that carries it."""
    columns = ", ".join(f"'{column}'" for column in SCOPE_COLUMNS)
    rows = psql(
        dsn,
        "select table_name, column_name from information_schema.columns "
        f"where table_schema = 'public' and data_type = 'uuid' and column_name in ({columns})",
    )
    inventory: dict[str, set[str]] = {}
    for row in rows:
        if len(row) == 2:
            inventory.setdefault(row[0], set()).add(row[1])
    return inventory


def table_counts(dsn: str, tables: list[str]) -> dict[str, int]:
    """Exact row count for every table, in one round trip."""
    if not tables:
        return {}
    union = " union all ".join(
        f"select '{table}' as table_name, count(*) from public.{quote(table)}" for table in tables
    )
    return {row[0]: int(row[1]) for row in psql(dsn, union) if len(row) == 2}


def migration_count(dsn: str, tables: list[str]) -> int:
    if MIGRATIONS_TABLE not in tables:
        return 0
    rows = psql(dsn, f"select count(*) from public.{quote(MIGRATIONS_TABLE)}")
    return int(rows[0][0]) if rows else 0


def probe_statements(inventory: dict[str, set[str]], tables: list[str]) -> dict[str, str]:
    """Cross-tenant probes: every one of them must return zero rows."""
    probes: dict[str, str] = {}
    tenant = f"x.org_id is not null and x.org_id <> '{GLOBAL_SCOPE}'::uuid"
    for table in sorted(inventory):
        columns = inventory[table]
        target = f"public.{quote(table)} x"
        if "org_id" in columns and table != "organizations":
            probes[f"org_orphan:{table}"] = (
                f"select count(*) from {target} where {tenant} and not exists "
                "(select 1 from public.organizations o where o.org_id = x.org_id)"
            )
        if "branch_id" in columns and "org_id" in columns:
            probes[f"branch_cross_org:{table}"] = (
                f"select count(*) from {target} join public.branches b on b.id = x.branch_id "
                f"where {tenant} and b.org_id <> x.org_id"
            )
        if "bot_id" in columns and "org_id" in columns:
            probes[f"bot_cross_org:{table}"] = (
                f"select count(*) from {target} join public.bots b on b.id = x.bot_id "
                f"where {tenant} and b.org_id <> x.org_id"
            )
    if "bots" in tables and "branches" in tables:
        probes["bots_branch_cross_org"] = (
            "select count(*) from public.bots b join public.branches br on br.id = b.branch_id "
            f"where b.org_id <> '{GLOBAL_SCOPE}'::uuid and br.org_id <> b.org_id"
        )
    if "branches" in tables and "organizations" in tables:
        probes["branches_org_orphan"] = (
            f"select count(*) from public.branches br where br.org_id <> '{GLOBAL_SCOPE}'::uuid "
            "and not exists (select 1 from public.organizations o where o.org_id = br.org_id)"
        )
    if "organizations" in tables and "tenants" in tables:
        probes["organizations_tenant_orphan"] = (
            "select count(*) from public.organizations o where o.tenant_id is not null "
            "and not exists (select 1 from public.tenants t where t.id = o.tenant_id)"
        )
    return probes


def run_probes(dsn: str, inventory: dict[str, set[str]], tables: list[str]) -> dict[str, int]:
    results: dict[str, int] = {}
    for name, sql in probe_statements(inventory, tables).items():
        rows = psql(dsn, sql)
        results[name] = int(rows[0][0]) if rows else 0
    return results


def observe(dsn: str) -> dict:
    """Collect everything the assertions need from one database."""
    tables = base_tables(dsn)
    if not tables:
        raise DrillError(f"no tables in the public schema of {mask(dsn)} — is this the right database?")
    inventory = column_inventory(dsn)
    return {
        "dsn": mask(dsn),
        "taken_at": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "migrations": migration_count(dsn, tables),
        "tables": table_counts(dsn, tables),
        "probes": run_probes(dsn, inventory, tables),
        "core_missing": [table for table in CORE_TABLES if table not in tables],
    }


def compare(observed: dict, expected: dict | None = None, min_migrations: int = 1) -> list[str]:
    """Return every reason the restored copy cannot be trusted (empty = pass)."""
    failures: list[str] = []
    for table in observed.get("core_missing", []):
        failures.append(f"core table `{table}` is missing — this is not a General Bots schema")

    migrations = int(observed.get("migrations", 0))
    if migrations < min_migrations:
        failures.append(
            f"`{MIGRATIONS_TABLE}` has {migrations} row(s), expected at least {min_migrations} — "
            "the migration history did not survive the restore"
        )

    if expected:
        wanted = expected.get("tables", {})
        restored = observed.get("tables", {})
        differences: list[str] = []
        for table in sorted(wanted):
            if table not in restored:
                differences.append(f"`{table}` is missing (the source had {wanted[table]} rows)")
            elif restored[table] != wanted[table]:
                differences.append(f"`{table}`: {restored[table]} rows, the source had {wanted[table]}")
        extra = sorted(set(restored) - set(wanted))
        if extra:
            names = ", ".join(f"`{name}`" for name in extra[:MAX_DETAILS])
            differences.append(f"{len(extra)} table(s) exist only in the restore: {names}")
        if differences:
            failures.append(f"{len(differences)} row-count difference(s) against the source manifest")
            failures.extend(differences[:MAX_DETAILS])
            if len(differences) > MAX_DETAILS:
                failures.append(f"... and {len(differences) - MAX_DETAILS} more difference(s)")

    for name, rows in sorted(observed.get("probes", {}).items()):
        if rows:
            failures.append(f"`{name}`: {rows} row(s) point outside their own tenant")
    return failures


def summary(observed: dict, failures: list[str]) -> str:
    tables = observed.get("tables", {})
    probes = observed.get("probes", {})
    total = sum(tables.values())
    verdict = "PASS" if not failures else f"FAIL ({len(failures)} failure(s))"
    return (
        f"restore drill: {verdict} — {observed.get('migrations', 0)} migration(s), "
        f"{len(tables)} table(s), {total} row(s), {len(probes)} tenant probe(s) "
        f"returning {sum(probes.values())} foreign row(s)"
    )


def write_report(path: Path, observed: dict, failures: list[str], label: str, note: str) -> None:
    lines = [
        f"# Restore drill — {observed.get('taken_at', '')}",
        "",
        f"- target: `{observed.get('dsn', '')}`",
        f"- mode: {label}",
    ]
    if note:
        lines.append(f"- source: {note}")
    lines += [
        f"- migrations applied: {observed.get('migrations', 0)}",
        f"- tables compared: {len(observed.get('tables', {}))} "
        f"({sum(observed.get('tables', {}).values())} rows)",
        f"- tenant probes: {len(observed.get('probes', {}))}, "
        f"{sum(observed.get('probes', {}).values())} row(s) outside their tenant",
        f"- result: **{'PASS' if not failures else 'FAIL'}**",
        "",
    ]
    if failures:
        lines += ["## Failures", ""] + [f"- {failure}" for failure in failures] + [""]
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("\n".join(lines), encoding="utf-8")


def fixture_self_test() -> int:
    """Prove the assertions offline: a clean copy passes, every defect fails."""
    inventory = {
        "bots": {"org_id", "branch_id", "bot_id"},
        "messages": {"org_id", "bot_id"},
        "organizations": {"org_id"},
        "branches": {"org_id"},
    }
    probes = {name: 0 for name in probe_statements(inventory, ["bots", "branches", "messages", "organizations"])}
    tables = {"organizations": 3, "branches": 4, "bots": 6, "users": 9, "messages": 120}
    clean = {
        "migrations": 137,
        "tables": dict(tables),
        "probes": dict(probes),
        "core_missing": [],
    }
    manifest = {"tables": dict(tables)}

    cases: list[tuple[str, dict, dict | None, int, bool, str]] = [
        ("clean restore", clean, manifest, 1, True, ""),
        ("no manifest still checks tenancy", clean, None, 1, True, ""),
        (
            "short table",
            {**clean, "tables": {**tables, "users": 7}},
            manifest,
            1,
            False,
            "`users`",
        ),
        (
            "cross-tenant bot",
            {**clean, "probes": {**probes, "bot_cross_org:messages": 2}},
            manifest,
            1,
            False,
            "bot_cross_org:messages",
        ),
        (
            "missing table",
            {**clean, "tables": {key: value for key, value in tables.items() if key != "messages"}},
            manifest,
            1,
            False,
            "`messages`",
        ),
        (
            "empty migration history",
            {**clean, "migrations": 0},
            manifest,
            1,
            False,
            MIGRATIONS_TABLE,
        ),
        (
            "not a bot schema",
            {**clean, "core_missing": ["bots"], "tables": {"users": 9}},
            None,
            1,
            False,
            "core table `bots`",
        ),
        (
            "extra table only in the restore",
            {**clean, "tables": {**tables, "leftover": 4}},
            manifest,
            1,
            False,
            "`leftover`",
        ),
    ]

    failures: list[str] = []
    for name, observed, expected, minimum, should_pass, needle in cases:
        found = compare(observed, expected, minimum)
        if should_pass and found:
            failures.append(f"case `{name}`: expected PASS, got {found}")
        if not should_pass and not found:
            failures.append(f"case `{name}`: expected FAIL, got a pass")
        if needle and not any(needle in message for message in found):
            failures.append(f"case `{name}`: no failure mentions {needle}: {found}")

    swapped = swap_database("postgres://gb:secret@127.0.0.1:5432/botserver?sslmode=disable", "gb_restore_drill_42")
    if swapped != "postgres://gb:secret@127.0.0.1:5432/gb_restore_drill_42?sslmode=disable":
        failures.append(f"scratch URL rewrite is wrong: {swapped}")
    if mask("postgres://gb:secret@127.0.0.1:5432/botserver") != "postgres://***@127.0.0.1:5432/botserver":
        failures.append("mask() leaked credentials")
    org_probe = probe_statements(inventory, ["bots", "branches", "messages", "organizations"])["org_orphan:messages"]
    if GLOBAL_SCOPE not in org_probe:
        failures.append("org probes must treat the nil UUID as the global scope, not as a tenant")

    if failures:
        for failure in failures:
            print(f"FAIL: {failure}", file=sys.stderr)
        return 1
    print(f"self-test: ok ({len(cases)} assertion case(s), scratch URL and masking)")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(
        prog="restore_drill_checks.py",
        description="Assertions for the restore drill: schema, row counts and tenant isolation.",
    )
    parser.add_argument("--self-test", action="store_true", help="prove the assertions offline")
    parser.add_argument("--dsn", help="connection URL of the database to inspect")
    parser.add_argument("--expected", help="manifest written from the source database")
    parser.add_argument("--manifest", help="write a source manifest (row counts) to this file")
    parser.add_argument("--out", help="write the observation as JSON to this file")
    parser.add_argument("--report", help="write a Markdown report to this file")
    parser.add_argument("--label", default="restored copy", help="mode shown in the report")
    parser.add_argument("--note", default="", help="one line about where the dump came from")
    parser.add_argument("--min-migrations", type=int, default=1, help="minimum applied migrations")
    parser.add_argument("--scratch-url", help="print this URL pointed at --name and exit")
    parser.add_argument("--name", help="database name for --scratch-url")
    args = parser.parse_args(argv)

    if args.self_test:
        return fixture_self_test()
    if args.scratch_url:
        if not args.name:
            parser.error("--scratch-url needs --name")
        print(swap_database(args.scratch_url, args.name))
        return 0
    if not args.dsn:
        parser.error("--dsn is required (or use --self-test / --scratch-url)")

    try:
        if args.manifest:
            observed = observe(args.dsn)
            path = Path(args.manifest)
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(
                json.dumps({"dsn": observed["dsn"], "tables": observed["tables"], "migrations": observed["migrations"]}, indent=2),
                encoding="utf-8",
            )
            print(f"manifest: {path} ({len(observed['tables'])} tables)")
            return 0

        expected = None
        if args.expected:
            path = Path(args.expected)
            if not path.is_file():
                raise DrillError(f"expected manifest not found: {path}")
            expected = json.loads(path.read_text(encoding="utf-8"))
        observed = observe(args.dsn)
        failures = compare(observed, expected, args.min_migrations)
    except DrillError as error:
        print(f"error: {error}", file=sys.stderr)
        return 2
    except json.JSONDecodeError as error:
        print(f"error: expected manifest is not valid JSON: {error}", file=sys.stderr)
        return 2

    if args.out:
        Path(args.out).write_text(json.dumps(observed, indent=2), encoding="utf-8")
    if args.report:
        write_report(Path(args.report), observed, failures, args.label, args.note)

    for failure in failures:
        print(f"MISMATCH: {failure}", file=sys.stderr)
    print(summary(observed, failures))
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
