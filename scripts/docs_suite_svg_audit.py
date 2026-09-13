#!/usr/bin/env python3
"""Audit the suite screen SVGs: rendered geometry and version claims.

Two things rot in this artwork. Text can be drawn outside the canvas, which no
one notices until the figure is opened; and the version labels record whatever
the dependency was on the day the screen was drawn.

`botserver/3rdparty.toml` IS what the installer downloads, so it is the
authority for versions here: a version in the artwork that disagrees with that
file is stale.

Run after editing a screen or bumping a dependency. Exit status 1 means
something needs redrawing.
"""
import os
import re
import sys
import xml.etree.ElementTree as ET

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
SVG_DIR = os.path.join(ROOT, "botbook/src/assets/suite")
MANIFEST = os.path.join(ROOT, "botserver/3rdparty.toml")
ROOT_MANIFEST = os.path.join(ROOT, "Cargo.toml")

NS = "{http://www.w3.org/2000/svg}"
DEFAULT_CANVAS = (900.0, 600.0)
TRANSLATE = re.compile(r"translate\(\s*(-?[\d.]+)[ ,]+(-?[\d.]+)\s*\)")
NUMBER = re.compile(r"-?[\d.]+(?:e-?\d+)?")

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


def canvas_of(root):
    """The document's own canvas: viewBox if present, else width/height.

    This folder holds flow diagrams and launcher art beside the app screens, so
    the canvas size is taken from each document rather than assumed.
    """
    view = root.get("viewBox")
    if view:
        parts = [float(n) for n in NUMBER.findall(view)]
        if len(parts) == 4:
            return parts[2], parts[3]
    try:
        return float(root.get("width")), float(root.get("height"))
    except (TypeError, ValueError):
        return DEFAULT_CANVAS


def geometry_findings(path):
    """Report text drawn outside the canvas, in canvas coordinates.

    The hand-authored screens compose with ``translate()`` groups, so a text
    element's own x/y is relative to its ancestors: the offsets are accumulated
    before the comparison, or every centred composition would be misread.
    """
    name = os.path.basename(path)
    try:
        root = ET.parse(path).getroot()
    except ET.ParseError as exc:
        return [f"{name}: not valid XML ({exc})"]
    width, height = canvas_of(root)
    findings = []

    def walk(node, dx, dy):
        transform = node.get("transform") or ""
        m = TRANSLATE.search(transform)
        if m:
            dx += float(m.group(1))
            dy += float(m.group(2))
        for kid in node:
            if kid.tag == f"{NS}text":
                label = "".join(kid.itertext()).strip()
                try:
                    x = float(kid.get("x", 0)) + dx
                    y = float(kid.get("y", 0)) + dy
                except ValueError:
                    walk(kid, dx, dy)
                    continue
                if label and (x < 0 or x > width or y < 0 or y > height):
                    findings.append(f"{name}: {label[:28]!r} drawn at "
                                    f"({x:.0f}, {y:.0f}), outside "
                                    f"{width:.0f}x{height:.0f}")
            walk(kid, dx, dy)

    walk(root, 0.0, 0.0)
    return findings


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
        findings.extend(geometry_findings(os.path.join(SVG_DIR, f)))
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
        print(f"{len(findings)} problem(s):")
        for x in findings:
            print(f"  {x}")
        return 1
    total = len([f for f in os.listdir(SVG_DIR) if f.endswith(".svg")])
    print(f"{total} screen SVGs: text inside the canvas, versions match 3rdparty.toml")
    return 0


if __name__ == "__main__":
    sys.exit(main())
