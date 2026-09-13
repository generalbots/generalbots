#!/usr/bin/env python3
"""Check version claims in the suite screen SVGs against the installer manifest.

The screen mockups are hand-drawn, so nothing updates them when a dependency
moves. `botserver/3rdparty.toml` IS what the installer downloads, so it is the
authority here: a version in the artwork that disagrees with that file is stale.

Run after bumping a dependency. Exit status 1 means something needs redrawing.
"""
import os
import re
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SVG_DIR = os.path.join(ROOT, "botbook/src/assets/suite")
MANIFEST = os.path.join(ROOT, "botserver/3rdparty.toml")
ROOT_MANIFEST = os.path.join(ROOT, "Cargo.toml")

# Label rendered in the artwork -> [components.*] section that pins it.
COMPONENT_MAP = {
    "PostgreSQL": "tables",
    "Qdrant": "vector_db",
    "Vault": "vault",
    "InfluxDB": "timeseries_db",
    "Valkey": "cache",
    "Redis": "cache",
}


def manifest_sections():
    """Return {section_name: body} for every [components.*] block."""
    text = open(MANIFEST, encoding="utf-8").read()
    out = {}
    for m in re.finditer(r"\[components\.([a-z_]+)\](.*?)(?=\n\[|\Z)", text, re.S):
        out[m.group(1)] = m.group(2)
    return out


def suite_version():
    text = open(ROOT_MANIFEST, encoding="utf-8").read()
    m = re.search(r'^version\s*=\s*"([0-9]+\.[0-9]+\.[0-9]+)"', text, re.M)
    return m.group(1) if m else None


def pinned_version(body):
    """First semver-looking number in a component body, or None."""
    m = re.search(r"(\d+\.\d+(?:\.\d+)?)", body)
    return m.group(1) if m else None


def main():
    sections = manifest_sections()
    real_suite = suite_version()
    findings = []

    for f in sorted(os.listdir(SVG_DIR)):
        if not f.endswith(".svg"):
            continue
        text = open(os.path.join(SVG_DIR, f), encoding="utf-8", errors="ignore").read()

        # Each version label sits in a <text> immediately after its component
        # name, so walk the labels and remember the last name seen.
        current_name = None
        for m in re.finditer(r"<text[^>]*>([^<]{1,60})</text>", text):
            label = m.group(1).strip()
            if label in COMPONENT_MAP:
                current_name = label
                continue
            vm = re.fullmatch(r"v(\d+\.\d+(?:\.\d+)?)(?: • .*)?", label)
            if not vm or not current_name:
                continue
            claimed = vm.group(1)
            section = COMPONENT_MAP[current_name]
            actual = pinned_version(sections.get(section, ""))
            if actual is None:
                findings.append(
                    f"{f}: {current_name} claims v{claimed}, "
                    f"but components.{section} pins no version"
                )
            elif not actual.startswith(claimed) and not claimed.startswith(actual):
                findings.append(
                    f"{f}: {current_name} claims v{claimed}, "
                    f"components.{section} pins {actual}"
                )
            current_name = None

        # Suite version badge, e.g. "v6.3.1 • Rust".
        for m in re.finditer(r'>v(\d+\.\d+\.\d+) • Rust<', text):
            if real_suite and m.group(1) != real_suite:
                findings.append(
                    f"{f}: suite badge v{m.group(1)}, workspace is v{real_suite}"
                )

    if findings:
        print(f"{len(findings)} stale version claim(s):")
        for x in findings:
            print(f"  {x}")
        return 1
    print(f"all version claims in {len(os.listdir(SVG_DIR))} SVGs match 3rdparty.toml")
    return 0


if __name__ == "__main__":
    sys.exit(main())
