#!/usr/bin/env python3
"""Offline static audit of the OS image's user-data protection (issue #2470).

Reads the working tree only. Runs no cargo, no browser, no server, no disk
encryption and no destructive disk command. The only child process is the
read-only suite webtest (`node ...webtest-data-protection.js`), whose own only
child is `bash -n` on restore.sh.

The audit separates two kinds of rows:

  [AUTOMATED] static guarantees that MUST hold — a failure exits 1 and names
              the guarantee (and its file:line evidence). Non-vacuous by design.
  [GAP]       criteria that can only be proven by a real LUKS/TPM unlock, a
              backup/restore cycle or an interactive Settings UI on an actual
              image/VM. Printed explicitly but NEVER fail this static audit.

    python3 scripts/check_data_protection_audit.py [--root PATH]

Exit 0 = every [AUTOMATED] guarantee passes (GAPs may remain), 1 = at least one
[AUTOMATED] guarantee failed.
"""

from __future__ import annotations

import argparse
import os
import re
import shutil
import subprocess
import sys
from pathlib import Path

GD = "desktop/os/data-protection/extra"
BIN = f"{GD}/usr/local/libexec/gb-data"
WEBTEST = "botui/ui/suite/tests/webtest-data-protection.js"
RESTORE = f"{BIN}/restore.sh"
PROVISION = f"{BIN}/provision.sh"
SNAPSHOT = f"{BIN}/snapshot.sh"
LIB = f"{BIN}/lib.sh"
RESET = f"{BIN}/reset-pc.sh"
PROV_UNIT = f"{GD}/usr/lib/systemd/system/gb-data-provision.service"
SNAP_TIMER = f"{GD}/usr/lib/systemd/system/gb-snapshot.timer"
CONFIG = f"{GD}/etc/generalbots/data-protection.conf"
SECURE_BOOT = "desktop/os/mkosi.conf.d/20-secure-boot.conf"

WEBTEST_EXPECTED_CASES = 16

# id -> (title, label). GAP rows have no static check and never fail the audit.
STATIC_ROWS = (
    ("A1", "suite webtest 16/16 regression", "[AUTOMATED]"),
    ("A2", "restore requires --confirm (typed error)", "[AUTOMATED]"),
    ("A3", "path args validated; metacharacters rejected", "[AUTOMATED]"),
    ("A4", "no silent overwrite: pre-restore snapshot first", "[AUTOMATED]"),
    ("A5", "LUKS2 first-boot wiring in image", "[AUTOMATED]"),
    ("A6", "recovery key: separate slot, 0600, export note", "[AUTOMATED]"),
    ("A7", "backup schedule + retention declared", "[AUTOMATED]"),
    ("A8", "reset requires RESET-THIS-PC token", "[AUTOMATED]"),
    ("A9", "honest docs: no false first-boot sealing claim", "[AUTOMATED]"),
)
GAP_ROWS = (
    ("G1", "LUKS unlock at first boot (TPM/token/passphrase, wrong-passphrase, no-TPM) on a real image"),
    ("G2", "recovery key actually printed/exported to offline storage at first run"),
    ("G3", "real scheduled backup of /var/lib/gbdesk completes; retention applied on an image"),
    ("G4", "destructive restore proven on a real image (destroy slot, restore from snapshot/archive)"),
    ("G5", "Settings 'Export my data' / 'Reset this PC' interactive buttons end-to-end"),
)


def line_of(src: str, needle: str) -> int:
    """1-based line of the first literal occurrence, or 0 when absent."""
    for no, line in enumerate(src.splitlines(), 1):
        if needle in line:
            return no
    return 0


def re_line_of(src: str, pattern: str) -> int:
    """1-based line of the first regex match, or 0 when absent."""
    match = re.search(pattern, src)
    if not match:
        return 0
    return src.count("\n", 0, match.start()) + 1


class Audit:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.results: dict[str, tuple[bool, str]] = {}
        self.read_errors: list[str] = []

    def read(self, rel: str) -> str:
        try:
            return (self.root / rel).read_text(encoding="utf-8")
        except (OSError, UnicodeError) as exc:
            self.read_errors.append(f"{rel}: {exc}")
            return ""

    def static(self, aid: str, ok: bool, detail: str) -> None:
        self.results[aid] = (bool(ok), detail)

    def run(self) -> None:
        self.a1_webtest()
        restore = self.read(RESTORE)
        self.a2_confirm(restore)
        self.a3_path_validation(restore)
        self.a4_no_silent_overwrite(restore)
        provision = self.read(PROVISION)
        self.a5_luks_wiring(provision)
        self.a6_recovery_key(provision, self.read(LIB))
        self.a7_backup_schedule(self.read(SNAPSHOT), self.read(SNAP_TIMER), self.read(CONFIG))
        self.a8_reset_token(self.read(RESET))
        self.a9_honest_docs(self.read(SECURE_BOOT))

    # -- A1: the committed suite regression harness still passes 16/16 --------
    def a1_webtest(self) -> None:
        node = shutil.which("node") or "node"
        try:
            proc = subprocess.run(
                [node, WEBTEST],
                cwd=str(self.root),
                capture_output=True,
                text=True,
                timeout=120,
                check=False,
            )
        except (OSError, subprocess.SubprocessError) as exc:
            self.static("A1", False, f"cannot run node webtest: {exc}")
            return
        out = proc.stdout + proc.stderr
        match = re.search(
            r"(?m)^TOTAL:\s*(\d+)\s*\|\s*PASS:\s*(\d+)\s*\|\s*FAIL:\s*(\d+)\s*$", out
        )
        if not match:
            self.static("A1", False, f"no TOTAL line in webtest output (rc={proc.returncode})")
            return
        total, passed, failed = (int(value) for value in match.groups())
        ok = proc.returncode == 0 and total == WEBTEST_EXPECTED_CASES and passed == total and failed == 0
        self.static(
            "A1",
            ok,
            f"TOTAL={total} PASS={passed} FAIL={failed} rc={proc.returncode} (expected 16/16, rc 0)",
        )

    # -- A2: destructive restore needs an explicit, typed confirmation -------
    def a2_confirm(self, src: str) -> None:
        gate = line_of(src, 'if [ "$confirm" -ne 1 ]; then')
        die = line_of(src, "refusing destructive restore without --confirm")
        self.static(
            "A2",
            gate > 0 and die > 0 and gate < die,
            f"{RESTORE}:{gate} gate -> :{die} typed error"
            if gate and die else f"missing --confirm gate (gate={gate}, die={die})",
        )

    # -- A3: path args are validated before reaching run_sh shell fragments --
    def a3_path_validation(self, src: str) -> None:
        defined = line_of(src, "validate_path_arg() {")
        reject = line_of(src, "shell metacharacter (refusing to run)")
        snap_call = line_of(src, 'validate_path_arg "--snapshot" "$name"')
        arch_call = line_of(src, 'validate_path_arg "--archive" "$archive"')
        ok = all(value > 0 for value in (defined, reject, snap_call, arch_call))
        self.static(
            "A3",
            ok,
            f"{RESTORE}:{defined} defined, :{reject} typed reject, "
            f":{snap_call} / :{arch_call} wired"
            if ok else f"validate_path_arg evidence missing "
            f"(defined={defined}, reject={reject}, calls={snap_call}/{arch_call})",
        )

    # -- A4: archive restore is not a silent overwrite ------------------------
    def a4_no_silent_overwrite(self, src: str) -> None:
        start = src.find("restore_archive()")
        end = src.find("\nmain()", start) if start >= 0 else -1
        body = src[start:end] if start >= 0 and end > start else ""
        snap = body.find("btrfs subvolume snapshot")
        decrypt = re.search(r"gpg[^\n]*\|\s*tar", body)
        ok = start >= 0 and snap >= 0 and decrypt is not None and snap < decrypt.start()
        base = src.count("\n", 0, start) + 1 if start >= 0 else 0
        detail = (
            f"{RESTORE}:{base + src[start:start + snap].count(chr(10))} pre-restore snapshot "
            f"before gpg|tar decrypt"
            if ok else "restore_archive() has no pre-restore btrfs snapshot before the gpg|tar write"
        )
        self.static("A4", ok, detail)

    # -- A5: first-boot LUKS2 provisioning is wired into the image -----------
    def a5_luks_wiring(self, prov: str) -> None:
        fmt = line_of(prov, "cryptsetup luksFormat --type luks2")
        enroll = line_of(prov, "systemd-cryptenroll --tpm2-device=auto")
        init_flag = line_of(prov, 'if [ -e "$INIT_FLAG" ]')
        unit = self.read(PROV_UNIT)
        unit_exec = line_of(unit, "provision.sh --boot")
        ok = fmt > 0 and enroll > 0 and init_flag > 0 and unit_exec > 0
        self.static(
            "A5",
            ok,
            f"{PROVISION}:{fmt} luksFormat, :{enroll} TPM enroll, :{init_flag} first-boot gate; "
            f"{PROV_UNIT}:{unit_exec} unit"
            if ok else f"LUKS wiring missing (fmt={fmt}, enroll={enroll}, init={init_flag}, unit={unit_exec})",
        )

    # -- A6: recovery key lives in its own slot and a protected file ---------
    def a6_recovery_key(self, prov: str, lib: str) -> None:
        gen = line_of(prov, 'generate_secret "$RECOVERY_KEY_FILE" 32')
        add = line_of(prov, "cryptsetup luksAddKey")
        note = line_of(prov, "export $RECOVERY_KEY_FILE to offline storage")
        perm = line_of(lib, "install -m 0600 -o root -g root")
        ok = gen > 0 and add > 0 and note > 0 and perm > 0
        self.static(
            "A6",
            ok,
            f"{PROVISION}:{gen} runtime key, :{add} separate slot, :{note} export note; "
            f"{LIB}:{perm} 0600 root-owned"
            if ok else f"recovery-key evidence missing (gen={gen}, add={add}, note={note}, perm={perm})",
        )

    # -- A7: a daily schedule and a documented retention policy exist --------
    def a7_backup_schedule(self, snap: str, timer: str, conf: str) -> None:
        calendar = line_of(timer, "OnCalendar=daily")
        keep_daily = line_of(conf, "SNAPSHOT_KEEP_DAILY=")
        keep_weekly = line_of(conf, "SNAPSHOT_KEEP_WEEKLY=")
        retention = line_of(snap, "apply_retention()")
        archive = re_line_of(snap, r"--symmetric --cipher-algo AES256")
        ok = calendar > 0 and keep_daily > 0 and keep_weekly > 0 and retention > 0 and archive > 0
        self.static(
            "A7",
            ok,
            f"{SNAP_TIMER}:{calendar} daily timer; {CONFIG}:{keep_daily}/{keep_weekly} retention; "
            f"{SNAPSHOT}:{retention} apply, :{archive} encrypted archive"
            if ok else "backup schedule/retention evidence missing "
            f"(calendar={calendar}, daily={keep_daily}, weekly={keep_weekly}, "
            f"retention={retention}, archive={archive})",
        )

    # -- A8: reset needs a typed token and cannot touch rescue partitions ----
    def a8_reset_token(self, reset: str) -> None:
        token = line_of(reset, '[ "$token" != "RESET-THIS-PC" ]')
        guard = line_of(reset, "assert_not_protected")
        ok = token > 0 and guard > 0
        self.static(
            "A8",
            ok,
            f"{RESET}:{token} exact token gate, :{guard} protected-partition guard"
            if ok else f"reset safety gate missing (token={token}, guard={guard})",
        )

    # -- A9: the shipped config no longer claims first-boot sealing ----------
    def a9_honest_docs(self, secure_boot: str) -> None:
        body_start = line_of(secure_boot, "DATA AT REST (#2470)")
        false_claim = re_line_of(
            secure_boot, r"sealing of the root volume is[\s\S]*?performed at first boot"
        ) or line_of(secure_boot, "performed at first boot by the provisioning wizard")
        ok = body_start > 0 and false_claim == 0
        self.static(
            "A9",
            ok,
            f"{SECURE_BOOT}:{body_start} states no LUKS/TPM flow; false claim absent"
            if ok else f"honesty marker missing ({body_start}) or false claim still present (:{false_claim})",
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    default_root = Path(__file__).resolve().parent.parent
    parser.add_argument(
        "--root",
        default=os.environ.get("GB_DATA_PROTECTION_ROOT", str(default_root)),
        help="tree to audit (default: repository containing this script)",
    )
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    audit = Audit(root)
    audit.run()

    print(f"DATA-PROTECTION STATIC AUDIT (#2470)  root={root}")
    print(f"{'ID':<4} {'Label':<12} {'Evidence / note':<62} Result")
    failed: list[str] = []
    for aid, title, label in STATIC_ROWS:
        ok, detail = audit.results.get(aid, (False, "assertion did not run"))
        if not ok:
            failed.append(aid)
        print(f"{aid:<4} {label:<12} {f'{title} — {detail}':<62} {'PASS' if ok else 'FAIL'}")
    for aid, note in GAP_ROWS:
        print(f"{aid:<4} {'[GAP]':<12} {f'{note}':<62} NOT VERIFIED")
    if audit.read_errors:
        print("unreadable sources: " + "; ".join(audit.read_errors))

    print(
        f"RESULT: {len(STATIC_ROWS) - len(failed)}/{len(STATIC_ROWS)} static guarantees PASS, "
        f"{len(GAP_ROWS)} GAP(s) explicit (non-failing)"
    )
    if failed:
        print("FAILED: " + ", ".join(failed))
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
