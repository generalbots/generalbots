# Adequacy baseline (#1735)

`scripts/adequacy_guard.py` measures the structural rules from `AGENTS.md`
(`#[allow]`, `table!` outside `botschema`, production `unwrap`/`expect`/`panic`,
raw `Command::new`, backups, CDN assets, part-suffixed modules/styles, files over
450 lines, commented-out code). This file is the ceiling for those counts.

**The numbers only move down.** A count may stay equal or fall; it may never
grow. When a change removes a deviation, lower the matching `count` in the same
change (or run `python3 scripts/adequacy_guard.py --update`).

`owner` names the issue that owns the count — the one that lowers a non-zero
backlog, or the one that closed it. `null` means the rule is at zero with no
dedicated issue; #1735 protects it. `unassigned` marks a non-zero backlog that
no cluster owns yet (raw `Command::new`).

```json
"rules": {
  "allow_attributes": { "count": 0, "owner": null },
  "part_suffixed_rust": { "count": 41, "owner": "#1728" }
}
```

Commands:

```bash
# CI check — exit 1 when any count grows
python3 scripts/adequacy_guard.py --baseline issues/adequacy/baseline.json

# after removing deviations, rewrite the ceilings
python3 scripts/adequacy_guard.py --update

# fail on part-suffix / over-limit violations under a directory a PR touches
python3 scripts/adequacy_guard.py --baseline issues/adequacy/baseline.json \
  --strict botserver/crates/botvibe
```

The file records the date it was generated (`generated`) and the issue that owns
the whole guard (`issue`).
