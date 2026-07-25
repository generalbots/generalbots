#!/usr/bin/env python3
"""
Migrate sensitive bot config keys from bot_configuration table → Vault.

Reads all rows from bot_configuration where config_key matches sensitive patterns
(token, key, password, secret, auth, credential, private, cert),
groups them by (org_id, branch_id, bot_id), and writes each group to Vault at:

    secret/gbo/bot/{org_id}/{branch_id}/{bot_id}

After successful migration, deletes the migrated rows from bot_configuration.

Usage:
    python3 /tmp/migrate_bot_config_to_vault.py

Environment variables needed:
    VAULT_ADDR         - Vault server URL (e.g. https://127.0.0.1:8200)
    VAULT_TOKEN        - Vault authentication token
    VAULT_CACERT       - CA cert path (optional, uses -k if not found)

Database credentials are read from Vault at `secret/gbo/tables`.
"""

import os
import sys
import json
import subprocess


SENSITIVE_PATTERNS = [
    "key", "token", "password", "passwd", "secret",
    "auth", "credential", "private", "cert", "certificate",
    "api_key", "apikey",
]


def is_sensitive_key(key: str) -> bool:
    lower = key.lower()
    return any(p in lower for p in SENSITIVE_PATTERNS)


def get_env_or_die(name: str) -> str:
    val = os.environ.get(name)
    if not val:
        print(f"ERROR: {name} not set")
        sys.exit(1)
    return val


def vault_curl(method: str, path: str, data: dict | None = None) -> dict | None:
    vault_addr = get_env_or_die("VAULT_ADDR")
    vault_token = get_env_or_die("VAULT_TOKEN")
    ca_cert = os.environ.get("VAULT_CACERT", "")

    url = f"{vault_addr}/v1/{path}"
    args = ["curl", "-sf"]

    if ca_cert and os.path.exists(ca_cert):
        args += ["--cacert", ca_cert]
    else:
        args += ["-k"]

    args += ["-H", f"X-Vault-Token: {vault_token}"]

    if method == "GET":
        args += [url]
        result = subprocess.run(args, capture_output=True, text=True, timeout=30)
    elif method == "PUT":
        args += ["-X", "PUT", "--data", json.dumps(data), url]
        result = subprocess.run(args, capture_output=True, text=True, timeout=30)
    elif method == "DELETE":
        args += ["-X", "DELETE", url]
        result = subprocess.run(args, capture_output=True, text=True, timeout=30)
    else:
        return None

    if result.returncode != 0:
        print(f"  curl failed: {result.stderr.strip()}")
        return None

    try:
        return json.loads(result.stdout)
    except json.JSONDecodeError:
        return None


def vault_health() -> bool:
    result = vault_curl("GET", "sys/health")
    return result is not None


def vault_get_secret(path: str) -> dict[str, str] | None:
    """Read all fields from a Vault KV v2 secret."""
    result = vault_curl("GET", f"secret/data/{path}")
    if result and "data" in result:
        data = result["data"]
        if "data" in data:
            return data["data"]
    return None


def vault_put_secret(path: str, data: dict[str, str]) -> bool:
    """Write all fields to a Vault KV v2 secret."""
    payload = {"data": data}
    result = vault_curl("PUT", f"secret/data/{path}", payload)
    return result is not None


def get_bot_identity(cursor, bot_id: str) -> tuple[str, str]:
    """Resolve org_id and branch_id for a bot_id."""
    if bot_id == "00000000-0000-0000-0000-000000000000":
        return (bot_id, bot_id)
    try:
        cursor.execute(
            "SELECT org_id::text, branch_id::text FROM bots WHERE id = %s",
            (bot_id,)
        )
        row = cursor.fetchone()
        if row:
            return (row[0], row[1])
    except Exception as e:
        print(f"  WARNING: failed to resolve bot {bot_id}: {e}")
    return ("00000000-0000-0000-0000-000000000000",
            "00000000-0000-0000-0000-000000000000")


def main():
    print("=" * 60)
    print("Bot Config Migration: bot_configuration → Vault")
    print("=" * 60)

    # 1. Check Vault
    print("\n[1/5] Checking Vault connectivity...")
    if not vault_health():
        print("  Vault is NOT reachable. Make sure VAULT_ADDR and VAULT_TOKEN are set.")
        print("  Script will continue and save report — run again when Vault is up.")
        vault_ok = False
    else:
        print("  Vault is reachable!")
        vault_ok = True

    # 2. Get DB credentials from Vault
    print("\n[2/5] Reading database credentials from Vault...")
    db_secret = vault_get_secret("gbo/tables")
    if not db_secret:
        print("  ERROR: Could not read DB credentials from Vault at secret/gbo/tables")
        sys.exit(1)

    db_host = db_secret.get("host", "localhost")
    db_port = db_secret.get("port", "5432")
    db_name = db_secret.get("database", "botserver")
    db_user = db_secret.get("username", "postgres")
    db_pass = db_secret.get("password", "postgres")
    db_url = f"postgresql://{db_user}:{db_pass}@{db_host}:{db_port}/{db_name}"

    try:
        import psycopg2
        conn = psycopg2.connect(db_url)
        cur = conn.cursor()
        print(f"  Connected to {db_host}:{db_port}/{db_name} as {db_user}")
    except Exception as e:
        print(f"  ERROR: Cannot connect to database: {e}")
        sys.exit(1)

    # 3. Find sensitive keys
    print("\n[3/5] Searching for sensitive keys in bot_configuration...")

    like_clauses = " OR ".join(
        f"lower(config_key) LIKE '%{p}%'"
        for p in SENSITIVE_PATTERNS
    )
    cur.execute(f"""
        SELECT bot_id::text, config_key, config_value
        FROM bot_configuration
        WHERE {like_clauses}
        ORDER BY bot_id, config_key
    """)
    rows = cur.fetchall()

    if not rows:
        print("  No sensitive keys found in bot_configuration. Nothing to migrate.")
        sys.exit(0)

    print(f"  Found {len(rows)} sensitive key-value pairs.")

    # 4. Group by bot_id
    print("\n[4/5] Grouping by bot and resolving org/branch...")
    groups: dict[str, dict[str, str]] = {}
    for bot_id, config_key, config_value in rows:
        # Skip empty/placeholder values
        val = (config_value or "").strip()
        if not val or val.lower() in ("none", "null", "n/a", ""):
            continue
        groups.setdefault(bot_id, {})[config_key] = val

    print(f"  Grouped into {len(groups)} bot(s):")
    for bot_id in sorted(groups.keys()):
        keys = list(groups[bot_id].keys())
        display = bot_id[:8] + "..."
        print(f"    bot {display}: {len(keys)} key(s) — {', '.join(keys[:3])}{'...' if len(keys) > 3 else ''}")

    # 5. Migrate to Vault
    print("\n[5/5] Writing to Vault...")

    migrated_count = 0
    failed_count = 0
    vault_paths_written = []

    for bot_id, config_data in groups.items():
        org_id, branch_id = get_bot_identity(cur, bot_id)
        vault_path = f"gbo/bot/{org_id}/{branch_id}/{bot_id}"

        if vault_ok:
            # Read existing vault data and merge
            existing = vault_get_secret(vault_path) or {}
            existing.update(config_data)

            if vault_put_secret(vault_path, existing):
                vault_paths_written.append(vault_path)
                migrated_count += len(config_data)
                print(f"  ✅ bot {bot_id[:8]}... → {vault_path} ({len(config_data)} keys)")
            else:
                failed_count += len(config_data)
                print(f"  ❌ bot {bot_id[:8]}... → FAILED")
        else:
            # Dry-run: just report
            print(f"  📋 DRY-RUN bot {bot_id[:8]}... → {vault_path} ({len(config_data)} keys)")
            vault_paths_written.append(f"[DRY-RUN] {vault_path}")

    # 6. Delete migrated rows from DB (only if Vault writes succeeded)
    print()
    if vault_ok and migrated_count > 0 and failed_count == 0:
        confirm = input(f"  Delete {migrated_count} migrated rows from bot_configuration? [y/N]: ")
        if confirm.lower() == "y":
            bot_ids = list(groups.keys())
            for bid in bot_ids:
                keys = list(groups[bid].keys())
                for k in keys:
                    cur.execute(
                        "DELETE FROM bot_configuration WHERE bot_id = %s::uuid AND config_key = %s",
                        (bid, k)
                    )
            conn.commit()
            print(f"  ✅ Deleted {migrated_count} rows from bot_configuration")
        else:
            print("  Skipped deletion. Rows remain in bot_configuration.")
    elif vault_ok and failed_count > 0:
        print(f"  ⚠️  {failed_count} key(s) failed to write. Not deleting anything.")
    else:
        print("  📋 DRY-RUN complete. No data was written or deleted.")

    # Summary
    print()
    print("=" * 60)
    print("SUMMARY")
    print("=" * 60)
    print(f"  Total sensitive key-value pairs found:  {len(rows)}")
    print(f"  Grouped into bots:                     {len(groups)}")
    if vault_ok:
        print(f"  Successfully written to Vault:         {migrated_count}")
        print(f"  Failed:                                {failed_count}")
    else:
        print(f"  DRY-RUN (Vault not available):         {len(rows)} keys ready")
    print(f"  Vault path pattern:                     gbo/bot/{{org}}/{{branch}}/{{bot}}")

    # Save vault paths for reference
    report_path = "/tmp/migrate_bot_config_vault_report.txt"
    with open(report_path, "w") as f:
        f.write(f"Migration Report - {__import__('datetime').datetime.now()}\n")
        f.write(f"{'='*60}\n")
        for vp in vault_paths_written:
            f.write(f"{vp}\n")
    print(f"\n  Report saved to: {report_path}")

    cur.close()
    conn.close()


if __name__ == "__main__":
    main()
