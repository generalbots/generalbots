"""Command line entry point for the screen generator."""
import os
import re
import sys

from .extract import load_catalog, resolve_entry
from .markup import load_ftl
from .paths import OUT
from .render import render


def main():
    labels = load_ftl()
    catalog = load_catalog()
    mode = "--report" in sys.argv
    check = "--check" in sys.argv
    wanted = [a for a in sys.argv[1:] if not a.startswith("--")]

    written, failed, report = 0, [], []
    for app in catalog:
        app_id, url = app["id"], app["url"]
        if wanted and app_id not in wanted:
            continue
        entry, sources = resolve_entry(app_id, url)
        if not entry:
            failed.append((app_id, "no entry file"))
            continue
        title = labels.get(f"{app_id}-title") or app["title"]
        try:
            svg = render(entry, labels, title, sources)
        except Exception as exc:                      # noqa: BLE001
            failed.append((app_id, f"{type(exc).__name__}: {exc}"))
            continue
        regions = len(re.findall(r'class="(chip|row|sidebar-bg)"', svg))
        report.append((app_id, title, len(svg), regions))
        if mode or check:
            continue
        with open(os.path.join(OUT, f"{app_id}-screen.svg"), "w",
                  encoding="utf-8") as fh:
            fh.write(svg)
        written += 1

    if mode:
        print(f"{'app':24} {'title':24} bytes visuals")
        for app_id, title, size, regions in report:
            print(f"{app_id:24} {title[:23]:24} {size:5} {regions:4}")
    else:
        print(f"wrote {written} screens" if not check else
              f"checked {len(report)} screens")
    for app_id, err in failed:
        print(f"  FAILED {app_id}: {err}")
