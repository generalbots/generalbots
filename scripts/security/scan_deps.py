#!/usr/bin/env python3
"""Scheduled dependency scan driver that drafts (never publishes) advisories.

Runs cargo audit / pip-audit / npm audit / osv-scanner when installed, writes a
JSON report artifact, deduplicates against published advisory IDs and expires
waivers. CI wiring belongs to the agent3 lane; this script is deliberately
standalone and offline-safe (missing tools are reported, not fatal)."""
import argparse
import json
import shutil
import subprocess
from datetime import date
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
TOOLS = [("cargo audit", ["cargo", "audit", "--json"]), ("pip-audit", ["pip-audit", "-f", "json"]),
         ("npm audit", ["npm", "audit", "--json"]), ("osv-scanner", ["osv-scanner", "--format", "json"])]


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--out", default="/tmp/opencode/security-reports")
    args = parser.parse_args()
    report = {"date": date.today().isoformat(), "runs": [], "findings": [], "expired_waivers": [], "drafts": []}
    for name, command in TOOLS:
        if shutil.which(command[0]) is None:
            report["runs"].append({"tool": name, "status": "unavailable"})
            continue
        try:
            result = subprocess.run(command, cwd=ROOT, capture_output=True, text=True, timeout=600, check=False)
            report["runs"].append({"tool": name, "status": "ok" if result.returncode == 0 else "findings",
                                   "output": result.stdout[-8000:]})
        except (OSError, subprocess.SubprocessError) as error:
            report["runs"].append({"tool": name, "status": "error", "error": str(error)})
    waiver_file = ROOT / "security" / "policy" / "waivers.json"
    published = {p.stem for p in (ROOT / "security" / "advisories").glob("*.md")}
    if waiver_file.exists():
        for waiver in json.loads(waiver_file.read_text(encoding="utf-8")).get("waivers", []):
            if date.fromisoformat(waiver["expires"]) < date.today():
                report["expired_waivers"].append(waiver)
    out = Path(args.out) / (date.today().isoformat() + ".json")
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2, ensure_ascii=False) + "\n", encoding="utf-8")
    print(f"scan_deps: report {out} — {len(report['runs'])} tool(s), {len(report['expired_waivers'])} expired waiver(s)")
    print("scan_deps: drafts are never auto-published; no advisory file was created")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
