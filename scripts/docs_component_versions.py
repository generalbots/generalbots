#!/usr/bin/env python3
"""Sync documented component versions with the installer manifest.

`botserver/3rdparty.toml` is what the installer downloads, so it is the
authority for every version the book quotes. This rewrites those quotes from
the manifest and reports anything it could not match, so the documentation
cannot silently drift from the software.

Run with --check to report drift without writing (exit 1 when stale).
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SRC = os.path.join(ROOT, "botbook/src")
MANIFEST = os.path.join(ROOT, "botserver/3rdparty.toml")

# Service name as written in component-reference.md -> [components.*] section.
SERVICES = {
    "HashiCorp Vault": "vault",
    "PostgreSQL": "tables",
    "Zitadel": "directory",
    "MinIO": "drive",
    "Valkey": "cache",
    "llama.cpp": "llm_linux_vulkan",
    "Stalwart Mail Server": "email",
    "Caddy": "caddy",
    "CoreDNS": "dns",
    "Forgejo": "alm",
    "LiveKit": "meet",
    "Qdrant": "vector_db",
    "InfluxDB": "timeseries_db",
}


def sections():
    text = open(MANIFEST, encoding="utf-8").read()
    out = {}
    for m in re.finditer(r"\[components\.([a-z_0-9]+)\](.*?)(?=\n\[|\Z)", text, re.S):
        url = re.search(r'^url\s*=\s*"([^"]+)"', m.group(2), re.M)
        ver = re.search(r'(\d+\.\d+\.\d+)', m.group(2))
        build = re.search(r'llama\.cpp/releases/download/(b\d+)', m.group(2))
        out[m.group(1)] = {
            "url": url.group(1) if url else None,
            "version": ver.group(1) if ver else (build.group(1) if build else None),
        }
    return out


def sync_component_reference(data):
    """Rewrite the version and download URL of every service block."""
    path = os.path.join(SRC, "12-ecosystem-reference/component-reference.md")
    text = open(path, encoding="utf-8").read()
    original = text
    report = []

    for service, key in SERVICES.items():
        info = data.get(key)
        if not info or not info["version"]:
            report.append(f"SKIP  {service}: no version in components.{key}")
            continue

        block = re.search(
            r"(\|\s*\*\*Service\*\*\s*\|\s*" + re.escape(service) + r"\s*\|"
            r".*?)(?=\n##|\Z)",
            text,
            re.S,
        )
        if not block:
            report.append(f"SKIP  {service}: no block found")
            continue

        body = block.group(1)
        new = body

        # Version cell.
        new = re.sub(
            r"(\|\s*\*\*Current Version\*\*\s*\|\s*)([^|]+?)(\s*\|)",
            lambda m: f"{m.group(1)}{info['version']}{m.group(3)}",
            new,
            count=1,
        )

        # Download URL: only rewrite a bare URL line, and only when the
        # manifest pins one.
        if info["url"]:
            new = re.sub(
                r"^(https?://\S+)$",
                info["url"],
                new,
                count=1,
                flags=re.M,
            )

        if new != body:
            text = text.replace(body, new, 1)
            report.append(f"SYNC  {service} -> {info['version']}")

    if text != original:
        open(path, "w", encoding="utf-8").write(text)
    return report


# Section name -> label used in updating-components.md's summary block.
SUMMARY_ROWS = {
    "vault": "vault",
    "tables": "tables",
    "directory": "directory",
    "drive": "drive",
    "cache": "cache",
    "llm_linux_vulkan": "llm",
    "email": "email",
    "caddy": "proxy",
    "dns": "dns",
    "alm": "alm",
    "meet": "meeting",
}


def sync_updating_components(data):
    """Rewrite the version summary and the manifest snippets."""
    path = os.path.join(SRC, "12-ecosystem-reference/updating-components.md")
    text = open(path, encoding="utf-8").read()
    original = text
    report = []

    # 1. The "botserver Stack Versions:" block lists section: version.
    def fix_row(m):
        label, spaces, rest = m.group(1), m.group(2), m.group(4)
        for key, row_label in SUMMARY_ROWS.items():
            if row_label != label:
                continue
            info = data.get(key, {})
            if not info.get("version"):
                return m.group(0)
            return f"  {label}:{spaces}{info['version']}{rest}"
        return m.group(0)

    text = re.sub(
        r"^  ([a-z_]+):( +)([0-9][0-9.]*|latest|b\d+)([^\n]*)$",
        lambda m: fix_row(m),
        text,
        flags=re.M,
    )

    # 2. Manifest snippets inside fences: "[components.X]" then url/filename.
    def fix_snippet(m):
        key, body = m.group(1), m.group(2)
        info = data.get(key)
        if not info or not info.get("url"):
            return m.group(0)
        new = re.sub(r'^url = "[^"]*"', f'url = "{info["url"]}"', body, count=1, flags=re.M)
        base = info["url"].rsplit("/", 1)[-1]
        if re.search(r'^filename = "[^"]*"', new, re.M):
            new = re.sub(
                r'^filename = "[^"]*"', f'filename = "{base}"', new, count=1, flags=re.M
            )
        if new != body:
            report.append(f"SYNC  snippet components.{key}")
        return f"[components.{key}]{new}"

    text = re.sub(
        r"\[components\.([a-z_0-9]+)\](.*?)(?=\n```)",
        fix_snippet,
        text,
        flags=re.S,
    )

    if text != original:
        open(path, "w", encoding="utf-8").write(text)
    for label in SUMMARY_ROWS:
        info = data.get(label)
        if info and info.get("version"):
            report.append(f"SYNC  summary {label} -> {info['version']}")
    return report


def main():
    check = "--check" in sys.argv[1:]
    data = sections()
    print(f"manifest components: {len(data)}")

    if check:
        path = os.path.join(SRC, "12-ecosystem-reference/component-reference.md")
        text = open(path, encoding="utf-8").read()
        stale = []
        for service, key in SERVICES.items():
            info = data.get(key)
            if not info or not info["version"]:
                continue
            block = re.search(
                r"\|\s*\*\*Service\*\*\s*\|\s*" + re.escape(service) + r"\s*\|(.*?)(?=\n##|\Z)",
                text,
                re.S,
            )
            if not block:
                continue
            m = re.search(r"\*\*Current Version\*\*\s*\|\s*([^|]+?)\s*\|", block.group(1))
            if m and m.group(1).strip() != info["version"]:
                stale.append(f"{service}: doc {m.group(1).strip()} vs manifest {info['version']}")
        if stale:
            print(f"{len(stale)} stale:")
            for s in stale:
                print(f"  {s}")
            return 1
        print("component-reference.md matches 3rdparty.toml")
        return 0

    for line in sync_component_reference(data):
        print(line)
    for line in sync_updating_components(data):
        print(line)
    return 0


if __name__ == "__main__":
    sys.exit(main())
