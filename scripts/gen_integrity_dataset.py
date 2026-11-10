#!/usr/bin/env python3
"""Deterministic production-like dataset for the data-integrity suite (#2117).

The suite in `bottest/tests/integration/data_integrity.rs` needs a database that
looks like production — several orgs, several branches per org, several bots per
branch, tens of thousands of scoped rows — so that "migrate", "restore" and
"tenant isolation" are proven on real shapes instead of nil UUIDs.

This generator writes two files into `--out`:

    integrity_dataset.sql   COPY blocks, parents first, loaded with `psql -f`
    integrity_manifest.json per-table / per-scope row counts + id checksums

Determinism: every identifier is derived from `sha256(seed|kind|path)`, so two
runs with the same `--seed` are byte identical and the manifest checksums can be
recomputed in SQL as

    md5(string_agg(id::text, ',' ORDER BY id::text))

Scope kinds recorded in the manifest (the test owns the matching SQL):

    org_id          rows carry `org_id`
    branch_id       rows carry `branch_id` (plus NULL-branch "global" rows)
    bot_id          rows carry `bot_id`
    session_branch  rows hang off `user_sessions.branch_id`
    ticket_branch   rows hang off `support_tickets.branch_id`

Deliberately excluded: `crm_contacts`. Two migrations put contradictory foreign
keys on its `org_id` (`6.5.15.1-consolidated/up.sql` -> organizations,
`6.5.16-branch-id-isolation/up.sql` -> branches), so a row cannot satisfy both;
seeding it would make the restore drill fail for a reason unrelated to this
suite. Tracked separately — see the note in the test header.

Usage:
    python3 scripts/gen_integrity_dataset.py --out /tmp/gb-integrity
    python3 scripts/gen_integrity_dataset.py --out /tmp/gb-integrity --scale 10
"""

from __future__ import annotations

import argparse
import hashlib
import json
import uuid
from pathlib import Path

DEFAULT_SEED = 2117
ORGS = 3
BRANCHES_PER_ORG = 2
BOTS_PER_BRANCH = 2
USERS_PER_ORG = 8

# table -> (scope kind, rows per org at scale 1.0)
SCALED = {
    "products": ("org_id", 250),
    "support_tickets": ("org_id", 600),
    "user_sessions": ("branch_id", 400),
    "drive_files": ("branch_id", 800),
    "bot_memories": ("bot_id", 200),
    "audit_log": ("bot_id", 1000),
}
COMMENTS_PER_TICKET = 3
MESSAGES_PER_SESSION = 8
GLOBAL_SESSIONS = 20
GLOBAL_FILES = 40

# COPY order: parents before children.
COPY_ORDER = [
    ("tenants", ["id", "name", "slug"]),
    ("organizations", ["org_id", "name", "slug", "tenant_id"]),
    ("branches", ["id", "org_id", "tenant_id", "slug", "name"]),
    ("bots", ["id", "name", "llm_provider", "context_provider", "org_id", "slug",
              "branch_id", "tenant_id"]),
    ("users", ["id", "username", "email", "password_hash"]),
    ("user_organizations", ["id", "user_id", "org_id", "role", "is_default"]),
    ("products", ["id", "org_id", "bot_id", "sku", "name", "product_type", "price",
                  "branch_id"]),
    ("support_tickets", ["id", "org_id", "bot_id", "ticket_number", "subject",
                         "status", "priority", "requester_email", "branch_id"]),
    ("ticket_comments", ["id", "ticket_id", "content"]),
    ("user_sessions", ["id", "user_id", "bot_id", "title", "branch_id"]),
    ("message_history", ["id", "session_id", "user_id", "role",
                         "content_encrypted", "message_index"]),
    ("bot_memories", ["id", "bot_id", "key", "value", "branch_id"]),
    ("drive_files", ["id", "file_path", "file_type", "name", "branch_id"]),
    ("audit_log", ["id", "event_type", "actor_type", "actor_id", "action", "bot_id"]),
]


def det_uuid(seed: int, kind: str, path: str) -> uuid.UUID:
    """A stable UUID for (seed, kind, path) — same input, same id, forever."""
    digest = hashlib.sha256(f"{seed}|{kind}|{path}".encode()).digest()[:16]
    return uuid.UUID(bytes=digest, version=4)


def checksum(ids: list[uuid.UUID]) -> str:
    """Same digest the test recomputes in SQL over `id::text` sorted ascending."""
    joined = ",".join(sorted(str(i) for i in ids))
    return hashlib.md5(joined.encode()).hexdigest()


def copy_cell(value) -> str:
    if value is None:
        return r"\N"
    if isinstance(value, bool):
        return "t" if value else "f"
    text = str(value)
    return (text.replace("\\", "\\\\").replace("\t", "\\t")
                .replace("\n", "\\n").replace("\r", "\\r"))


def build(seed: int, scale: float) -> tuple[dict, list[tuple[str, list[str], list[list]]]]:
    rows: dict[str, list[list]] = {name: [] for name, _ in COPY_ORDER}
    scopes: dict[str, dict[str, list[uuid.UUID]]] = {t: {} for t in SCALED}
    scopes.setdefault("ticket_comments", {})
    scopes.setdefault("message_history", {})
    null_scope: dict[str, list[uuid.UUID]] = {"user_sessions": [], "drive_files": []}
    orgs: list[dict] = []

    def n(per_org: int) -> int:
        return max(1, int(round(per_org * scale)))

    for oi in range(ORGS):
        org_slug = f"integrity-{chr(ord('a') + oi)}"
        tenant_id = det_uuid(seed, "tenant", org_slug)
        org_id = det_uuid(seed, "org", org_slug)
        rows["tenants"].append([tenant_id, f"Tenant {org_slug}", org_slug])
        rows["organizations"].append([org_id, f"Org {org_slug}", org_slug, tenant_id])

        branch_ids: list[uuid.UUID] = []
        bot_ids: list[uuid.UUID] = []
        for bi in range(BRANCHES_PER_ORG):
            bpath = f"{org_slug}/branch-{bi}"
            branch_id = det_uuid(seed, "branch", bpath)
            branch_ids.append(branch_id)
            rows["branches"].append([branch_id, org_id, tenant_id,
                                     f"{org_slug}-b{bi}", f"Branch {bi} of {org_slug}"])
            for bo in range(BOTS_PER_BRANCH):
                bpath2 = f"{bpath}/bot-{bo}"
                bot_id = det_uuid(seed, "bot", bpath2)
                bot_ids.append(bot_id)
                rows["bots"].append([bot_id, f"bot-{org_slug}-{bi}{bo}", "openai",
                                     "qdrant", org_id, f"{org_slug}-b{bi}-{bo}",
                                     branch_id, tenant_id])

        user_ids: list[uuid.UUID] = []
        for ui in range(USERS_PER_ORG):
            upath = f"{org_slug}/user-{ui}"
            user_id = det_uuid(seed, "user", upath)
            user_ids.append(user_id)
            rows["users"].append([user_id, f"{org_slug}-u{ui}",
                                  f"{org_slug}-u{ui}@integrity.test", "x"])
            rows["user_organizations"].append(
                [det_uuid(seed, "membership", upath), user_id, org_id, "member", "f"])
        orgs.append({"slug": org_slug, "org_id": str(org_id),
                     "tenant_id": str(tenant_id),
                     "branch_ids": [str(b) for b in branch_ids],
                     "bot_ids": [str(b) for b in bot_ids],
                     "user_ids": [str(u) for u in user_ids]})

        def note(table: str, scope_id: uuid.UUID, row_id: uuid.UUID) -> None:
            scopes[table].setdefault(str(scope_id), []).append(row_id)

        for i in range(n(SCALED["products"][1])):
            row_id = det_uuid(seed, "product", f"{org_slug}/{i}")
            bot_id, branch_id = bot_ids[i % len(bot_ids)], branch_ids[i % len(branch_ids)]
            rows["products"].append([row_id, org_id, bot_id, f"SKU-{org_slug}-{i}",
                                     f"Product {i}", "physical", "19.90", branch_id])
            note("products", org_id, row_id)

        ticket_ids: list[uuid.UUID] = []
        for i in range(n(SCALED["support_tickets"][1])):
            row_id = det_uuid(seed, "ticket", f"{org_slug}/{i}")
            ticket_ids.append(row_id)
            bot_id, branch_id = bot_ids[i % len(bot_ids)], branch_ids[i % len(branch_ids)]
            rows["support_tickets"].append(
                [row_id, org_id, bot_id, f"T-{org_slug}-{i}", f"Ticket {i}", "open",
                 "medium", f"{org_slug}-{i}@integrity.test", branch_id])
            note("support_tickets", org_id, row_id)
        for i, ticket_id in enumerate(ticket_ids):
            branch_id = branch_ids[i % len(branch_ids)]
            for ci in range(COMMENTS_PER_TICKET):
                row_id = det_uuid(seed, "comment", f"{org_slug}/{i}/{ci}")
                rows["ticket_comments"].append([row_id, ticket_id, f"Comment {ci}"])
                note("ticket_comments", branch_id, row_id)

        for i in range(n(SCALED["user_sessions"][1])):
            row_id = det_uuid(seed, "session", f"{org_slug}/{i}")
            bot_id, branch_id = bot_ids[i % len(bot_ids)], branch_ids[i % len(branch_ids)]
            user_id = user_ids[i % len(user_ids)]
            rows["user_sessions"].append([row_id, user_id, bot_id, f"Chat {i}", branch_id])
            note("user_sessions", branch_id, row_id)
            for mi in range(MESSAGES_PER_SESSION):
                msg_id = det_uuid(seed, "message", f"{org_slug}/{i}/{mi}")
                rows["message_history"].append(
                    [msg_id, row_id, user_id, str(mi % 2), f"cipher-{msg_id}", str(mi)])
                note("message_history", branch_id, msg_id)

        for i in range(n(SCALED["bot_memories"][1])):
            row_id = det_uuid(seed, "memory", f"{org_slug}/{i}")
            bot_id, branch_id = bot_ids[i % len(bot_ids)], branch_ids[i % len(branch_ids)]
            rows["bot_memories"].append([row_id, bot_id, f"key-{i}", f"value-{i}", branch_id])
            note("bot_memories", bot_id, row_id)

        for i in range(n(SCALED["drive_files"][1])):
            row_id = det_uuid(seed, "file", f"{org_slug}/{i}")
            branch_id = branch_ids[i % len(branch_ids)]
            rows["drive_files"].append([row_id, f"/{org_slug}/docs/{i}.pdf", "file",
                                        f"doc-{i}.pdf", branch_id])
            note("drive_files", branch_id, row_id)

        for i in range(n(SCALED["audit_log"][1])):
            row_id = det_uuid(seed, "audit", f"{org_slug}/{i}")
            bot_id = bot_ids[i % len(bot_ids)]
            rows["audit_log"].append([row_id, "tool.exec", "user", user_ids[0],
                                      "files.read", bot_id])
            note("audit_log", bot_id, row_id)

    # "Global" rows: no branch, the nil-branch fallback path. They must never
    # show up in a tenant-scoped query.
    for i in range(GLOBAL_SESSIONS):
        row_id = det_uuid(seed, "session", f"global/{i}")
        user_id = det_uuid(seed, "user", f"integrity-a/user-{i % USERS_PER_ORG}")
        bot_id = det_uuid(seed, "bot", f"integrity-a/branch-0/bot-{i % BOTS_PER_BRANCH}")
        rows["user_sessions"].append([row_id, user_id, bot_id, f"Global {i}", None])
        null_scope["user_sessions"].append(row_id)
    for i in range(GLOBAL_FILES):
        row_id = det_uuid(seed, "file", f"global/{i}")
        rows["drive_files"].append([row_id, f"/global/{i}.pdf", "file", f"g-{i}.pdf", None])
        null_scope["drive_files"].append(row_id)

    manifest_tables: dict[str, dict] = {}
    for table, (kind, _per_org) in SCALED.items():
        entries = {scope_id: {"rows": len(ids), "checksum": checksum(ids)}
                   for scope_id, ids in sorted(scopes[table].items())}
        manifest_tables[table] = {
            "scope": kind,
            "rows": sum(e["rows"] for e in entries.values()),
            "null_rows": len(null_scope.get(table, [])),
            "null_checksum": checksum(null_scope[table]) if null_scope.get(table) else None,
            "scopes": entries,
        }
    for table, kind in (("message_history", "session_branch"),
                        ("ticket_comments", "ticket_branch")):
        entries = {scope_id: {"rows": len(ids), "checksum": checksum(ids)}
                   for scope_id, ids in sorted(scopes[table].items())}
        manifest_tables[table] = {"scope": kind,
                                  "rows": sum(e["rows"] for e in entries.values()),
                                  "null_rows": 0, "null_checksum": None,
                                  "scopes": entries}

    manifest = {
        "version": 1,
        "seed": seed,
        "scale": scale,
        "sql_file": "integrity_dataset.sql",
        "generated_by": "scripts/gen_integrity_dataset.py",
        "orgs": orgs,
        "tables": manifest_tables,
        "total_rows": sum(len(v) for v in rows.values()),
    }
    ordered = [(name, cols, rows[name]) for name, cols in COPY_ORDER]
    return manifest, ordered


def render_sql(ordered: list[tuple[str, list[str], list[list]]]) -> str:
    out = ["-- generated by scripts/gen_integrity_dataset.py — do not edit by hand",
           "-- load with: psql -v ON_ERROR_STOP=1 -f integrity_dataset.sql",
           "BEGIN;"]
    for table, cols, table_rows in ordered:
        out.append(f"COPY {table} ({', '.join(cols)}) FROM stdin;")
        for row in table_rows:
            out.append("\t".join(copy_cell(c) for c in row))
        out.append("\\.")
    out.append("COMMIT;")
    return "\n".join(out) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--out", default="/tmp/gb-integrity",
                        help="output directory (default: /tmp/gb-integrity)")
    parser.add_argument("--seed", type=int, default=DEFAULT_SEED,
                        help=f"deterministic seed (default: {DEFAULT_SEED})")
    parser.add_argument("--scale", type=float, default=1.0,
                        help="multiplier for per-org row counts (default: 1.0)")
    parser.add_argument("--report", action="store_true", help="print the manifest summary")
    args = parser.parse_args()

    manifest, ordered = build(args.seed, args.scale)
    out_dir = Path(args.out)
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "integrity_dataset.sql").write_text(render_sql(ordered), encoding="utf-8")
    (out_dir / "integrity_manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    print(f"wrote {out_dir}/integrity_dataset.sql "
          f"({manifest['total_rows']} rows, seed {manifest['seed']}, "
          f"scale {manifest['scale']})")
    print(f"wrote {out_dir}/integrity_manifest.json")
    if args.report:
        for org in manifest["orgs"]:
            print(f"  {org['slug']}: org={org['org_id']} "
                  f"branches={len(org['branch_ids'])} bots={len(org['bot_ids'])}")
        for table, info in sorted(manifest["tables"].items()):
            print(f"  {table:<18} scope={info['scope']:<14} rows={info['rows']:<6} "
                  f"null={info['null_rows']}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
