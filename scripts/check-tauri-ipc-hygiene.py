#!/usr/bin/env python3
"""Static guard for the Tauri IPC surface (#2497) — no browser required."""
import json, re, sys
from pathlib import Path
ROOT = Path(__file__).resolve().parent.parent
fail = []
conf = json.loads((ROOT / "botapp/tauri.conf.json").read_text())
csp = conf.get("app", {}).get("security", {}).get("csp", "")
if not csp or csp == "null" or "unsafe-inline" in csp:
    fail.append("CSP must be a real policy with no unsafe-inline")
for cap in (ROOT / "botapp/capabilities").glob("*.json"):
    data = json.loads(cap.read_text())
    for url in data.get("remote", {}).get("urls", []):
        if "*" in url:
            fail.append(f"wildcard remote URL in {cap.name}: {url}")
drive = (ROOT / "botapp/src/desktop/drive.rs").read_text()
for marker in ["confirmation", "resolve_in_scopes", "consume"]:
    if marker not in drive:
        fail.append(f"drive.rs missing {marker}")
if fail:
    print("tauri IPC hygiene: FAIL")
    for f in fail: print(" -", f)
    sys.exit(1)
print("tauri IPC hygiene: OK")
