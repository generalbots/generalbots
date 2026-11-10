#!/usr/bin/env python3
"""Baseline I/O for the structural adequacy guard (#1735).

`scripts/adequacy_guard.py` owns the scan and the rule table; this module owns
the checked-in ceiling file, so the guard stays inside the 450-line limit. The
contract is unchanged: the baseline is a ceiling, not a snapshot — a count may
stay equal or fall, never rise.

The rule table (`RULE_META`) and the checkout root are passed in, so the module
has no import back into the guard and no global state.
"""

from __future__ import annotations

import json
import os
import time


def load_baseline(path: str, rule_meta: dict) -> dict:
    """Read the baseline and refuse one that does not cover every rule."""
    try:
        with open(path, "r", encoding="utf-8") as handle:
            data = json.load(handle)
    except (OSError, json.JSONDecodeError) as error:
        raise SystemExit(f"adequacy_guard: cannot read baseline {path}: {error}")
    rules = data.get("rules")
    if not isinstance(rules, dict):
        raise SystemExit(f"adequacy_guard: baseline {path} has no `rules` object")
    missing = [rule for rule in rule_meta if rule not in rules]
    if missing:
        raise SystemExit(f"adequacy_guard: baseline {path} is missing: {', '.join(missing)}")
    return data


def baseline_count(entry) -> int:
    """A ceiling as an int, accepting both the old and the annotated shape."""
    if isinstance(entry, int):
        return entry
    if isinstance(entry, dict) and isinstance(entry.get("count"), int):
        return entry["count"]
    raise SystemExit(f"adequacy_guard: malformed baseline entry: {entry!r}")


def write_baseline(
    path: str, counts: dict[str, int], previous: dict | None, rule_meta: dict, root: str
) -> None:
    """Recompute the baseline from the tree, keeping each rule's owner."""
    old_rules = (previous or {}).get("rules", {})
    rules = {
        rule: {
            "count": counts[rule],
            "owner": (old_rules.get(rule) or {}).get("owner", rule_meta[rule][0]),
        }
        for rule in rule_meta
    }
    data = {
        "issue": "#1735",
        "generated": time.strftime("%Y-%m-%d"),
        "contract": (
            "Ceilings, not snapshots: a count may stay equal or fall, never rise. "
            "Lower a number in the change that removes the deviation."
        ),
        "rules": rules,
    }
    directory = os.path.dirname(path)
    if directory:
        os.makedirs(directory, exist_ok=True)
    with open(path, "w", encoding="utf-8") as handle:
        json.dump(data, handle, indent=2)
        handle.write("\n")
    print(f"adequacy_guard: baseline written to {os.path.relpath(path, root)}")
