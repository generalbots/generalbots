#!/usr/bin/env python3
"""Generate the GB6 feature page from desktop/gb6/features.json (#1649).

The page is a build artifact: edit the record, run this script, commit both.
`scripts/gb6_features_check.py` fails CI when they disagree or when a
`shipped` feature lacks end-to-end evidence.
"""

from __future__ import annotations

import html
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
RECORD = ROOT / "desktop" / "gb6" / "features.json"
PAGE = ROOT / "desktop" / "gb6" / "index.html"

STATUS_LABEL = {
    "shipped": "Shipped",
    "partial": "Partial",
    "not_shipped": "Not shipped",
}


def load_record(path: Path = RECORD) -> dict:
    with path.open(encoding="utf-8") as handle:
        return json.load(handle)


def render(record: dict) -> str:
    rows = []
    for feature in record["features"]:
        status = feature["status"]
        evidence = feature.get("evidence") or {}
        tests = evidence.get("tests") or []
        tests_html = "<br>".join(
            f"<code>{html.escape(path)}</code>" for path in tests
        )
        e2e = evidence.get("e2e")
        e2e_html = html.escape(e2e) if e2e else "<span class=\"muted\">no E2E artifact</span>"
        issue = evidence.get("issue")
        issue_html = html.escape(issue) if issue else ""
        verified = feature.get("last_verified") or {}
        verified_html = " ".join(
            filter(
                None,
                [
                    html.escape(str(verified.get("date", ""))),
                    html.escape(str(verified.get("commit", ""))),
                    html.escape(str(verified.get("environment", ""))),
                ],
            )
        )
        rows.append(
            "<tr>"
            f"<td><strong>{html.escape(feature['id'])}</strong></td>"
            f"<td>{html.escape(feature['title'])}"
            f"<div class=\"promise\">{html.escape(feature.get('promise', ''))}</div></td>"
            f"<td class=\"status status-{html.escape(status)}\">{STATUS_LABEL.get(status, status)}</td>"
            f"<td>{tests_html}<div class=\"muted\">{issue_html}</div></td>"
            f"<td>{e2e_html}<div class=\"muted\">{html.escape(evidence.get('note', ''))}</div></td>"
            f"<td class=\"muted\">{verified_html}</td>"
            "</tr>"
        )
    shipped = sum(1 for f in record["features"] if f["status"] == "shipped")
    partial = sum(1 for f in record["features"] if f["status"] == "partial")
    return f"""<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>General Bots 6 — feature status</title>
<style>
:root {{ color-scheme: dark; }}
body {{ margin: 0; padding: 32px; background: #0f172a; color: #e2e8f0;
  font: 15px/1.55 system-ui, -apple-system, "Segoe UI", sans-serif; }}
main {{ max-width: 1180px; margin: 0 auto; }}
h1 {{ margin: 0 0 4px; font-size: 26px; color: #f8fafc; }}
.lead {{ color: #94a3b8; margin: 0 0 20px; }}
.summary {{ display: flex; gap: 16px; margin: 20px 0; }}
.card {{ background: #1e293b; border: 1px solid #334155; border-radius: 10px;
  padding: 12px 18px; }}
.card strong {{ display: block; color: #f8fafc; font-size: 22px; }}
table {{ width: 100%; border-collapse: collapse; background: #1e293b;
  border: 1px solid #334155; border-radius: 10px; overflow: hidden; }}
th, td {{ text-align: left; vertical-align: top; padding: 10px 12px;
  border-bottom: 1px solid #334155; }}
th {{ color: #94a3b8; font-size: 12px; text-transform: uppercase;
  letter-spacing: 0.04em; background: #172033; }}
tr:last-child td {{ border-bottom: none; }}
.promise {{ color: #94a3b8; font-size: 13px; margin-top: 4px; }}
.muted {{ color: #64748b; font-size: 12px; margin-top: 4px; }}
.status {{ font-weight: 600; white-space: nowrap; }}
.status-shipped {{ color: #4ade80; }}
.status-partial {{ color: #fbbf24; }}
.status-not_shipped {{ color: #f87171; }}
code {{ font: 12px ui-monospace, SFMono-Regular, Menlo, monospace;
  color: #93c5fd; }}
footer {{ color: #64748b; font-size: 12px; margin-top: 18px; }}
a {{ color: #93c5fd; }}
</style>
</head>
<body>
<main>
<h1>General Bots 6 — desktop edition</h1>
<p class="lead">{html.escape(record.get('note', ''))}</p>
<p class="lead">Parent epic: <a href="https://github.com/generalbots/generalbots/issues/1650">{html.escape(((record.get('epic') or {}).get('issue', '#1650')).lstrip('#'))}</a> — {html.escape(((record.get('epic') or {}).get('title', '')))}</p>
<div class="summary">
  <div class="card"><strong>{len(record['features'])}</strong>features in the contract</div>
  <div class="card"><strong>{shipped}</strong>shipped with E2E evidence</div>
  <div class="card"><strong>{partial}</strong>partial</div>
</div>
<table>
<thead>
<tr><th>ID</th><th>Feature</th><th>Status</th><th>Tests</th><th>End-to-end evidence</th><th>Last verified</th></tr>
</thead>
<tbody>
{''.join(rows)}
</tbody>
</table>
<footer>
Generated from <code>desktop/gb6/features.json</code> by
<code>desktop/gb6/generate_features.py</code>. Editing this page by hand does
not change the record and fails the CI check.
</footer>
</main>
</body>
</html>
"""


def main() -> int:
    record = load_record()
    PAGE.write_text(render(record), encoding="utf-8")
    print(f"wrote {PAGE.relative_to(ROOT)} ({len(record['features'])} features)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
