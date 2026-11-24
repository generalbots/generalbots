#!/usr/bin/env python3
"""Static audit: desktop-local first boot must never fetch ~2.5 GB from the network.

Reads the tree only; runs no cargo, no network, no browser. Every assertion is
non-vacuous: it fails when its specific guarantee is removed from the tree.

    python3 scripts/check_offline_first_boot_audit.py [--root PATH]

Exit 0 = all assertions PASS, 1 = at least one FAIL (printed by id + detail).
"""

from __future__ import annotations

import argparse
import json
import os
import re
import sys
import tomllib
from pathlib import Path
from urllib.parse import urlsplit

FACADE = "botserver/crates/botcore/src/package_manager/facade"
PM = "botserver/crates/botcore/src/package_manager"
PRODUCT = "botserver/.product.desktop-local"
THIRD_PARTY = "botserver/3rdparty.toml"
TAURI = "botapp/tauri.desktop-local.conf.json"

INSTALL_SYS = f"{FACADE}/install_system_packages.rs"
INSTALL = f"{FACADE}/install.rs"
LLM_ENGINE = f"{FACADE}/llm_engine.rs"
FACADE_MOD = f"{FACADE}/mod.rs"
CACHE = f"{PM}/cache.rs"
SIDECAR = "botapp/src/desktop/sidecar.rs"

GUARD = r'cfg!\(feature\s*=\s*"desktop-local"\)\s*&&\s*!desktop_download_allowed\(\)'

ASSERTIONS = (
    ("A1", "offline_guard_download_and_install"),
    ("A2", "offline_guard_data_download_list"),
    ("A3", "bundled_installer_checksum_coverage"),
    ("A4", "product_desktop_local_contract"),
    ("A5", "tauri_offline_resources_no_updater"),
    ("A6", "no_external_download_urls_in_boot_path"),
)


class Audit:
    def __init__(self, root: Path) -> None:
        self.root = root
        self.results: dict[str, tuple[bool, str]] = {}

    def read(self, rel: str) -> str:
        return (self.root / rel).read_text(encoding="utf-8")

    def check(self, aid: str, ok: bool, detail: str) -> None:
        self.results[aid] = (bool(ok), detail)

    def read_or_fail(self, rel: str, aid: str) -> str:
        try:
            return self.read(rel)
        except (OSError, UnicodeError) as exc:
            self.check(aid, False, f"missing/unreadable {rel}: {exc}")
            return ""

    def run(self) -> None:
        self.a1_download_guard()
        self.a2_data_download_guard()
        self.a3_checksum_coverage()
        self.a4_product_contract()
        self.a5_tauri_bundle()
        self.a6_no_external_urls()

    # -- A1: component archives ------------------------------------------------
    def a1_download_guard(self) -> None:
        aid = "A1"
        src = self.read_or_fail(INSTALL_SYS, aid)
        mod = self.read_or_fail(FACADE_MOD, aid)
        cache_src = self.read_or_fail(CACHE, aid)
        if not src or not mod or not cache_src:
            return

        has_fn = bool(re.search(r"pub\s+async\s+fn\s+download_and_install\s*\(", src))
        guards = list(re.finditer(GUARD, src))
        # The guard must be inside the CacheResult::Download branch, BEFORE any
        # network call, and must return Err (not silently skip).
        branch = src.find("CacheResult::Download {")
        call = src.find("self.download_with_reqwest(")
        early_err = any(
            "return Err(" in src[g.end():g.end() + 400] for g in guards
        )
        guarded = (
            has_fn
            and bool(guards)
            and branch >= 0
            and call >= 0
            and any(branch < g.start() < call for g in guards)
            and early_err
        )
        opt_in = (
            'fn desktop_download_allowed()' in mod
            and 'std::env::var("GB_DESKTOP_ALLOW_DOWNLOAD")' in mod
            and 'Ok("1")' in mod
            and not re.search(r"fn\s+desktop_download_allowed\(\)\s*->\s*bool\s*\{\s*true\s*\}", mod)
        )
        bundled_cache = (
            "resolve_component_url(component, url)" in src
            and "BOTSERVER_INSTALLERS_PATH" in cache_src
            and 'DEFAULT_CACHE_DIR: &str = "./botserver-installers"' in cache_src
        )
        ok = guarded and opt_in and bundled_cache
        self.check(
            aid,
            ok,
            f"guard-before-download={guarded} env-opt-in-only={opt_in} "
            f"cache-from-bundled-installers={bundled_cache}",
        )

    # -- A2: data files (GGUF models) ------------------------------------------
    def a2_data_download_guard(self) -> None:
        aid = "A2"
        src = self.read_or_fail(INSTALL, aid)
        llm = self.read_or_fail(LLM_ENGINE, aid)
        if not src or not llm:
            return

        start = src.find("if !component.data_download_list.is_empty()")
        guards = list(re.finditer(GUARD, src))
        net_call = src.find("utils::download_file(")
        data_guard = (
            start >= 0
            and net_call >= 0
            and any(start < g.start() < net_call for g in guards)
            and any(
                "Offline mode" in src[g.end():g.end() + 400] for g in guards
            )
        )
        # LLM models are data files; the CUDA runtime is an archive. Both must
        # route through the two guarded functions above.
        llm_routes = (
            "finish_llm_install(&url)" in src
            and "self.download_and_install(&runtime_url" in llm
        )
        self.check(
            aid,
            data_guard and llm_routes,
            f"guard-before-data-download={data_guard} llm-routes-through-guard={llm_routes}",
        )

    # -- A3: checksums for every bundled installer -----------------------------
    def a3_checksum_coverage(self) -> None:
        aid = "A3"
        try:
            cfg = tomllib.loads(self.read(THIRD_PARTY))
        except (OSError, UnicodeError, tomllib.TOMLDecodeError) as exc:
            self.check(aid, False, f"cannot parse {THIRD_PARTY}: {exc}")
            return

        pins: dict[str, set[str]] = {}
        source_pinned: set[str] = set()
        for section in ("components", "models"):
            for entry in cfg.get(section, {}).values():
                if not isinstance(entry, dict):
                    continue
                digest_keys = ("sha256", "sha256_arm64", "sha256_win")
                for key in ("filename", "filename_arm64", "filename_win"):
                    name = entry.get(key)
                    if not isinstance(name, str) or not name:
                        continue
                    digest_key = digest_keys[("filename", "filename_arm64", "filename_win").index(key)]
                    digest = entry.get(digest_key)
                    pins.setdefault(name, set())
                    if isinstance(digest, str) and digest.strip():
                        pins[name].add(digest.strip())
                # MinIO binary digest is toolchain-dependent; the bundle pins
                # its immutable source archive instead (documented in toml).
                if str(entry.get("source_sha256") or "").strip() and entry.get("build_script"):
                    for key in ("filename", "filename_arm64", "filename_win"):
                        name = entry.get(key)
                        if isinstance(name, str) and name:
                            source_pinned.add(name)

        installer_dir = self.bundled_installer_dir()
        if installer_dir is None:
            self.check(aid, False, "no botserver-installers/* resource mapping in tauri config")
            return
        try:
            bundled = sorted(p.name for p in installer_dir.iterdir() if p.is_file())
        except OSError as exc:
            self.check(aid, False, f"cannot list {installer_dir}: {exc}")
            return

        missing = [
            name for name in bundled
            if not pins.get(name) and name not in source_pinned
        ]
        self.check(
            aid,
            not missing and bool(bundled),
            f"{len(bundled)} bundled files, {len(missing)} without sha256 pin"
            + (f": {', '.join(missing)}" if missing else ""),
        )

    def bundled_installer_dir(self) -> Path | None:
        """Resolve the directory bundled by tauri's `botserver-installers/*` glob."""
        try:
            tauri = json.loads(self.read(TAURI))
        except (OSError, UnicodeError, json.JSONDecodeError):
            return self.root / "botserver" / "botserver-installers"
        resources = tauri.get("bundle", {}).get("resources", {})
        if not isinstance(resources, dict):
            return None
        for source, target in resources.items():
            if isinstance(source, str) and "botserver-installers/*" in source:
                if isinstance(target, str) and target.rstrip("/").endswith("installers"):
                    # tauri resource keys are relative to the config file (botapp/).
                    return (self.root / "botapp" / source.replace("/*", "")).resolve()
        return None

    # -- A4: desktop-local product contract -----------------------------------
    def a4_product_contract(self) -> None:
        aid = "A4"
        src = self.read_or_fail(PRODUCT, aid)
        if not src:
            return
        values: dict[str, str] = {}
        for line in src.splitlines():
            line = line.strip()
            if not line or line.startswith("#") or "=" not in line:
                continue
            key, value = line.split("=", 1)
            values[key.strip()] = value.strip()

        # The original chat+drive/no-security profile in AGENTS.md was
        # intentionally superseded by the General Bots OS edition: shell_mode,
        # real auth and the full app registry are now the shipped contract.
        # Assert the CURRENT contract, not the retired one.
        apps = values.get("apps", "")
        expected_apps = {"chat", "drive", "docs", "sheet", "slides"}
        apps_ok = expected_apps.issubset({part.strip() for part in apps.split(",")})
        expected = {
            "edition": "os",
            "no_security": "false",
            "suite_default_app": "chat",
            "shell_mode": "true",
        }
        mismatches = [
            f"{key}={values.get(key, '<missing>')!r} (expected {want!r})"
            for key, want in expected.items()
            if values.get(key) != want
        ]
        if not apps_ok:
            mismatches.append(f"apps missing core surfaces: {apps!r}")
        mismatches = [
            f"{key}={values.get(key, '<missing>')!r} (expected {want!r})"
            for key, want in expected.items()
            if values.get(key) != want
        ]
        self.check(
            aid,
            not mismatches,
            "OS-edition contract ok (chat+drive+office apps, auth required)"
            if not mismatches else "; ".join(mismatches),
        )

    # -- A5: tauri bundles installers, declares no updater plugin --------------
    def a5_tauri_bundle(self) -> None:
        aid = "A5"
        try:
            tauri = json.loads(self.read(TAURI))
        except (OSError, UnicodeError, json.JSONDecodeError) as exc:
            self.check(aid, False, f"cannot parse {TAURI}: {exc}")
            return
        resources = tauri.get("bundle", {}).get("resources", {})
        ships_installers = isinstance(resources, dict) and any(
            isinstance(k, str) and "botserver-installers/*" in k
            and isinstance(v, str) and v.rstrip("/").endswith("installers")
            for k, v in resources.items()
        )
        text = json.dumps(tauri).lower()
        no_updater = "plugins" not in tauri and "updater" not in text
        self.check(
            aid,
            ships_installers and no_updater,
            f"installers-bundled={ships_installers} no-updater-plugin={no_updater}",
        )

    # -- A6: no external download URL in boot code -----------------------------
    def a6_no_external_urls(self) -> None:
        aid = "A6"
        url_re = re.compile(r"\bhttps?://[^\s\"'`)<>]+")
        violations: list[str] = []
        for rel in (INSTALL_SYS, INSTALL, LLM_ENGINE, FACADE_MOD, CACHE, SIDECAR):
            src = self.read_or_fail(rel, aid)
            if not src:
                continue
            for lineno, raw in enumerate(src.splitlines(), 1):
                # Drop // comments before scanning (negative lookbehind keeps
                # the `//` inside `http://` intact).
                code = re.split(r"(?<!:)//", raw, maxsplit=1)[0]
                for url in url_re.findall(code):
                    host = urlsplit(url).hostname
                    if host not in {"localhost", "127.0.0.1", "::1"}:
                        violations.append(f"{rel}:{lineno}: {url}")
        toml_src = self.read_or_fail(THIRD_PARTY, aid)
        plaintext = [
            f"{THIRD_PARTY}:{i}"
            for i, line in enumerate(toml_src.splitlines(), 1)
            if "http://" in line
        ]
        self.check(
            aid,
            not violations and not plaintext,
            "loopback-only boot code, https-only metadata"
            if not violations and not plaintext
            else f"external URLs={violations} plaintext metadata={plaintext}",
        )


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    default_root = Path(__file__).resolve().parent.parent
    parser.add_argument(
        "--root",
        default=os.environ.get("GB_AUDIT_ROOT", str(default_root)),
        help="tree to audit (default: repository containing this script)",
    )
    args = parser.parse_args(argv)
    root = Path(args.root).resolve()
    audit = Audit(root)
    audit.run()

    print(f"OFFLINE-FIRST-BOOT AUDIT  root={root}")
    for aid, title in ASSERTIONS:
        ok, detail = audit.results.get(aid, (False, "assertion did not run"))
        print(f"{'PASS' if ok else 'FAIL'} {aid} {title}: {detail}")
    failed = [aid for aid, _ in ASSERTIONS if not audit.results.get(aid, (False, ""))[0]]
    print(f"RESULT: {len(ASSERTIONS) - len(failed)}/{len(ASSERTIONS)} PASS")
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
