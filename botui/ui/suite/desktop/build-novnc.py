#!/usr/bin/env python3
"""Bundle noVNC ES modules into a single browser-compatible script.

Reads the noVNC source from vendor/novnc/core/ and produces
vendor/novnc-bundle.js that can be loaded with a regular <script> tag.

Usage: python3 build-novnc.py
"""
import os
import re
import sys

WORKSPACE = os.path.dirname(os.path.abspath(__file__))
NOVNC_CORE = os.path.join(WORKSPACE, "vendor", "novnc", "core")
OUTPUT = os.path.join(WORKSPACE, "vendor", "novnc-bundle.js")

# Files to bundle in dependency order
BUNDLE_ORDER = [
    "util/int.js",
    "util/logging.js",
    "util/strings.js",
    "util/browser.js",
    "util/element.js",
    "util/events.js",
    "util/eventtarget.js",
    "util/cursor.js",
    "input/keysym.js",
    "input/xtscancodes.js",
    "input/keyboard.js",
    "input/gesturehandler.js",
    "encodings.js",
    "decoders/raw.js",
    "decoders/copyrect.js",
    "decoders/rre.js",
    "decoders/hextile.js",
    "decoders/tight.js",
    "decoders/tightpng.js",
    "decoders/zrle.js",
    "decoders/jpeg.js",
    "base64.js",
    "inflator.js",
    "deflator.js",
    "websock.js",
    "display.js",
    "ra2.js",
    "crypto/crypto.js",
    "rfb.js",
]

SKIP_IMPORTS = {
    "./util/int.js",
    "./util/logging.js",
    "./util/strings.js",
    "./util/browser.js",
    "./util/element.js",
    "./util/events.js",
    "./util/eventtarget.js",
    "./util/cursor.js",
    "./input/keysym.js",
    "./input/xtscancodes.js",
    "./input/keyboard.js",
    "./input/gesturehandler.js",
    "./encodings.js",
    "./decoders/raw.js",
    "./decoders/copyrect.js",
    "./decoders/rre.js",
    "./decoders/hextile.js",
    "./decoders/tight.js",
    "./decoders/tightpng.js",
    "./decoders/zrle.js",
    "./decoders/jpeg.js",
    "./base64.js",
    "./inflator.js",
    "./deflator.js",
    "./websock.js",
    "./display.js",
    "./ra2.js",
    "./crypto/crypto.js",
}


def strip_es_module(content):
    """Remove import/export statements, keep the code."""
    lines = []
    for line in content.split("\n"):
        stripped = line.strip()
        # Skip import lines
        if stripped.startswith("import "):
            continue
        # Skip export default
        if stripped.startswith("export default "):
            line = line.replace("export default ", "")
        # Skip export { ... }
        if stripped.startswith("export {") or stripped == "export {":
            continue
        # Skip export statements inside blocks
        if re.match(r"^\s*export\s+", stripped):
            line = re.sub(r"\bexport\s+", "", line)
        lines.append(line)
    return "\n".join(lines)


def get_export_name(content):
    """Try to find the default export name."""
    m = re.search(r"export\s+default\s+(\w+)", content)
    if m:
        return m.group(1)
    return None


def main():
    os.makedirs(os.path.dirname(OUTPUT), exist_ok=True)

    parts = []
    exports = {}

    for rel_path in BUNDLE_ORDER:
        full_path = os.path.join(NOVNC_CORE, rel_path)
        if not os.path.exists(full_path):
            print(f"SKIP (not found): {rel_path}", file=sys.stderr)
            continue

        with open(full_path, "r") as f:
            content = f.read()

        cleaned = strip_es_module(content)
        export_name = get_export_name(content)

        module_name = rel_path.replace("/", "_").replace(".js", "")
        parts.append(f"// === {rel_path} ===")
        parts.append(f"(function() {{ // module {module_name}")
        parts.append(cleaned)
        if export_name:
            parts.append(f"window['{export_name}'] = typeof {export_name} !== 'undefined' ? {export_name} : window['{export_name}'];")
        parts.append("}})();")
        parts.append("")

    bundle = "\n".join(parts)

    # Add noVNC namespace at the end
    bundle += """
// noVNC namespace for RFB class access
if (typeof window.noVNC === 'undefined') {
    window.noVNC = {};
}
if (typeof RFB !== 'undefined') {
    window.noVNC.RFB = RFB;
}
if (typeof Display !== 'undefined') {
    window.noVNC.Display = Display;
}
if (typeof Websock !== 'undefined') {
    window.noVNC.Websock = Websock;
}
"""

    with open(OUTPUT, "w") as f:
        f.write(bundle)

    size_kb = os.path.getsize(OUTPUT) / 1024
    print(f"Bundled noVNC -> {OUTPUT} ({size_kb:.0f} KB)")


if __name__ == "__main__":
    main()
