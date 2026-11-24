#!/usr/bin/env python3
"""Supply-chain identity for product releases (#2717), stdlib only.

provenance  in-toto SLSA v1.0 statement whose subjects are the release assets
            and whose builder/invocation point at the tag and CI run.
verify      fail closed unless checksums, manifest, CycloneDX SBOM, provenance
            and every cosign sidecar agree (the gate CI runs before upload).
list-assets print manifest asset names, one per line.
--self-test prove the gate rejects a tampered asset.

No keys/tokens/network here: cosign signs keyless (OIDC) in CI; this module
only produces and checks data. Exit: 0 clean, 1 violation, 2 bad invocation.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path
from typing import Any

STATEMENT_TYPE = "https://in-toto.io/Statement/v1"
SLSA_TYPE = "https://slsa.dev/provenance/v1"
CYCLONEDX_FORMAT = "CycloneDX"
REQUIRED_ECOSYSTEMS = ("pkg:cargo/", "pkg:npm/", "pkg:pypi/")
SIDECARS = (".sig", ".pem", ".bundle")


class SupplyChainError(Exception):
    """A verification or generation failure; always fails the release closed."""


def error(message: str) -> SupplyChainError:
    return SupplyChainError(message)


def utc_now() -> str:
    return datetime.now(timezone.utc).isoformat(timespec="seconds").replace("+00:00", "Z")


def read_json(path: Path, label: str) -> dict[str, Any]:
    try:
        with path.open(encoding="utf-8") as handle:
            value = json.load(handle)
    except (OSError, ValueError) as exc:
        raise error(f"{label}: cannot read {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise error(f"{label}: {path} must contain a JSON object")
    return value


def write_json(path: Path, value: dict[str, Any]) -> None:
    try:
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("w", encoding="utf-8") as handle:
            json.dump(value, handle, indent=2, sort_keys=True)
            handle.write("\n")
    except OSError as exc:
        raise error(f"cannot write {path}: {exc}") from exc


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    try:
        with path.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                digest.update(chunk)
    except OSError as exc:
        raise error(f"cannot hash {path}: {exc}") from exc
    return digest.hexdigest()


def manifest_assets(manifest: dict[str, Any]) -> dict[str, str]:
    assets = manifest.get("assets")
    if not isinstance(assets, dict) or not assets:
        raise error("MANIFEST.json has no assets")
    result: dict[str, str] = {}
    for name, entry in sorted(assets.items()):
        if not isinstance(entry, dict) or not isinstance(entry.get("sha256"), str):
            raise error(f"MANIFEST.json asset {name!r} has no sha256")
        result[name] = entry["sha256"].lower()
    return result


def env(*names: str, default: str = "") -> str:
    for name in names:
        value = os.environ.get(name, "")
        if value:
            return value
    return default


def build_provenance(args: argparse.Namespace) -> dict[str, Any]:
    manifest = read_json(Path(args.manifest), "MANIFEST.json")
    assets = manifest_assets(manifest)

    commit = args.commit or str(manifest.get("commit", ""))
    ref = args.ref or str(manifest.get("ref", ""))
    repository = args.repository or env("GITHUB_REPOSITORY")
    server = args.server_url or env("GITHUB_SERVER_URL", "https://github.com")
    run_id = args.run_id or env("GITHUB_RUN_ID")
    attempt = args.run_attempt or env("GITHUB_RUN_ATTEMPT", "1")
    tag = args.tag or str(manifest.get("release", ""))
    event = env("GITHUB_EVENT_NAME", "release")
    actor = env("GITHUB_ACTOR")
    workflow_ref = args.workflow_ref or env("GITHUB_WORKFLOW_REF")

    for label, value in (
        ("commit", commit),
        ("repository", repository),
        ("ref", ref),
        ("run_id", run_id),
    ):
        if not value:
            raise error(f"provenance needs --{label} (or the matching GITHUB_* env var)")

    builder_id = f"{server}/{repository}/.github/workflows/release.yml@{ref}"
    invocation = f"{server}/{repository}/actions/runs/{run_id}/attempts/{attempt}"
    source_uri = f"{server}/{repository}/tree/{commit}"

    subjects = [
        {"name": name, "digest": {"sha256": digest}}
        for name, digest in assets.items()
    ]

    return {
        "_type": STATEMENT_TYPE,
        "subject": subjects,
        "predicateType": SLSA_TYPE,
        "predicate": {
            "buildDefinition": {
                "buildType": builder_id,
                "externalParameters": {
                    "source": {
                        "repository": repository,
                        "ref": ref,
                        "commit": commit,
                        "tag": tag,
                        "uri": source_uri,
                    },
                    "workflow": workflow_ref or f"{repository}/.github/workflows/release.yml",
                    "trigger": event,
                    "run_id": run_id,
                    "run_attempt": attempt,
                },
                "internalParameters": {
                    "runner_os": env("RUNNER_OS", "Linux"),
                    "github_actor": actor,
                    "release_tag": tag,
                },
                "resolvedDependencies": [
                    {
                        "uri": f"git+{server}/{repository}@{commit}",
                        "digest": {"gitCommit": commit},
                    }
                ],
            },
            "runDetails": {
                "builder": {"id": builder_id},
                "metadata": {
                    "invocationId": invocation,
                    "startedOn": args.started_on or utc_now(),
                    "finishedOn": utc_now(),
                    "completion": "SUCCESS",
                },
            },
        },
    }


def cmd_provenance(args: argparse.Namespace) -> int:
    statement = build_provenance(args)
    write_json(Path(args.out), statement)
    print(f"provenance: {len(statement['subject'])} subjects, builder {statement['predicate']['runDetails']['builder']['id']}")
    return 0


def cmd_list_assets(args: argparse.Namespace) -> int:
    base = Path(args.dir)
    manifest = read_json(base / args.manifest, "MANIFEST.json")
    for name in manifest_assets(manifest):
        print(name)
    return 0


def parse_checksums(path: Path) -> dict[str, str]:
    """Parse a GNU coreutils SHA256SUMS file into name -> digest."""
    try:
        lines = path.read_text(encoding="utf-8").splitlines()
    except OSError as exc:
        raise error(f"cannot read {path}: {exc}") from exc
    sums: dict[str, str] = {}
    for line in lines:
        if not line.strip() or line.startswith("#"):
            continue
        parts = line.split(None, 1)
        if len(parts) != 2:
            raise error(f"{path}: malformed checksum line: {line!r}")
        digest, name = parts[0].lower(), parts[1].strip()
        if name.startswith("*"):
            name = name[1:]
        if len(digest) != 64 or any(char not in "0123456789abcdef" for char in digest):
            raise error(f"{path}: invalid SHA-256 for {name!r}")
        sums[name] = digest
    return sums


def check_sbom(base: Path, name: str) -> None:
    sbom = read_json(base / name, "SBOM")
    if sbom.get("bomFormat") != CYCLONEDX_FORMAT:
        raise error(f"{name}: bomFormat is not {CYCLONEDX_FORMAT}")
    if not str(sbom.get("specVersion", "")).startswith("1."):
        raise error(f"{name}: missing CycloneDX specVersion")
    components = sbom.get("components")
    if not isinstance(components, list) or len(components) < 10:
        raise error(f"{name}: SBOM has too few components to be credible")
    purls = " ".join(str(component.get("purl", "")) for component in components if isinstance(component, dict))
    missing = [eco for eco in REQUIRED_ECOSYSTEMS if eco not in purls]
    if missing:
        raise error(f"{name}: SBOM misses vendored ecosystems: {', '.join(missing)}")


def check_provenance(base: Path, name: str, manifest: dict[str, Any], assets: dict[str, str]) -> None:
    statement = read_json(base / name, "provenance")
    if statement.get("_type") != STATEMENT_TYPE:
        raise error(f"{name}: not an in-toto v1 statement")
    if statement.get("predicateType") != SLSA_TYPE:
        raise error(f"{name}: not an SLSA v1.0 provenance")

    subjects = statement.get("subject")
    if not isinstance(subjects, list):
        raise error(f"{name}: statement has no subjects")
    stated: dict[str, str] = {}
    for subject in subjects:
        if not isinstance(subject, dict):
            raise error(f"{name}: subject is not an object")
        digest = subject.get("digest")
        if not isinstance(digest, dict):
            raise error(f"{name}: subject has no digest")
        stated[str(subject.get("name"))] = str(digest.get("sha256", "")).lower()
    if stated != assets:
        raise error(f"{name}: subjects do not match MANIFEST.json assets")

    predicate = statement.get("predicate")
    if not isinstance(predicate, dict):
        raise error(f"{name}: statement has no predicate")
    definition = predicate.get("buildDefinition", {})
    external = definition.get("externalParameters", {})
    source = external.get("source", {})
    details = predicate.get("runDetails", {})
    metadata = details.get("metadata", {})
    builder = details.get("builder", {})

    commit = str(manifest.get("commit", ""))
    if source.get("commit") != commit:
        raise error(f"{name}: provenance commit is not the release commit {commit}")
    if str(external.get("run_id", "")) != str(manifest.get("run_id", "")):
        raise error(f"{name}: provenance run_id is not the release CI run")
    if str(manifest.get("run_id", "")) not in str(metadata.get("invocationId", "")):
        raise error(f"{name}: invocationId does not reference the CI run")
    expected_builder = f"https://github.com/{manifest.get('repository', '')}/.github/workflows/release.yml@{manifest.get('ref', '')}"
    if builder.get("id") != expected_builder:
        raise error(f"{name}: builder id does not reference the release workflow/ref")


def cmd_verify(args: argparse.Namespace) -> int:
    base = Path(args.dir)
    manifest = read_json(base / args.manifest, "MANIFEST.json")
    assets = manifest_assets(manifest)
    sums = parse_checksums(base / args.checksums)

    problems: list[str] = []

    for name, expected in assets.items():
        path = base / name
        if not path.is_file():
            problems.append(f"missing asset: {name}")
            continue
        actual = sha256_file(path)
        if actual != expected:
            problems.append(f"checksum mismatch: {name} (manifest {expected}, file {actual})")
        if sums.get(name) != expected:
            problems.append(f"SHA256SUMS mismatch for {name}")
        for suffix in SIDECARS:
            sidecar = base / f"{name}{suffix}"
            if not sidecar.is_file() or sidecar.stat().st_size == 0:
                problems.append(f"missing signature sidecar: {sidecar.name}")

    for extra in (args.checksums, args.manifest, args.sbom, args.provenance):
        path = base / extra
        if not path.is_file() or path.stat().st_size == 0:
            problems.append(f"missing trust metadata: {extra}")
            continue
        for suffix in SIDECARS:
            sidecar = base / f"{extra}{suffix}"
            if not sidecar.is_file() or sidecar.stat().st_size == 0:
                problems.append(f"missing signature sidecar: {sidecar.name}")

    try:
        check_sbom(base, args.sbom)
    except SupplyChainError as exc:
        problems.append(str(exc))
    try:
        check_provenance(base, args.provenance, manifest, assets)
    except SupplyChainError as exc:
        problems.append(str(exc))

    if problems:
        for problem in problems:
            print(f"::error::{problem}", file=sys.stderr)
        print(f"FAIL CLOSED: {len(problems)} supply-chain violation(s)", file=sys.stderr)
        return 1

    print(f"verified {len(assets)} assets, {len(assets) * len(SIDECARS)} signatures, SBOM and provenance")
    return 0


def self_test() -> int:
    """Prove the checksum/provenance gate fails on tampering, using only /tmp."""
    with tempfile.TemporaryDirectory(prefix="gb-supply-chain-") as tmp:
        base = Path(tmp)
        digest = hashlib.sha256(b"abc").hexdigest()
        manifest = {
            "release": "v0.0.0-test",
            "commit": "b" * 40,
            "ref": "refs/tags/v0.0.0-test",
            "run_id": "42",
            "repository": "generalbots/generalbots",
            "assets": {"toy.bin": {"sha256": digest, "size": 3}},
        }
        (base / "MANIFEST.json").write_text(json.dumps(manifest), encoding="utf-8")
        (base / "SHA256SUMS").write_text(f"{digest}  toy.bin\n", encoding="utf-8")
        (base / "toy.bin").write_bytes(b"abc")
        statement = build_provenance(
            argparse.Namespace(
                manifest=str(base / "MANIFEST.json"),
                commit="", ref="", repository="generalbots/generalbots", server_url="https://github.com",
                run_id="42", run_attempt="1", tag="", workflow_ref="", started_on="",
            )
        )
        (base / "provenance.slsa.json").write_text(json.dumps(statement), encoding="utf-8")
        sbom = {
            "bomFormat": "CycloneDX", "specVersion": "1.5",
            "components": [{"purl": f"pkg:{eco}/pkg@1.0.0"} for eco in ("cargo", "npm", "pypi")] * 4,
        }
        (base / "sbom.cdx.json").write_text(json.dumps(sbom), encoding="utf-8")
        for name in ("toy.bin", "SHA256SUMS", "MANIFEST.json", "sbom.cdx.json", "provenance.slsa.json"):
            for suffix in SIDECARS:
                (base / f"{name}{suffix}").write_bytes(b"sig")

        args = argparse.Namespace(
            dir=str(base), manifest="MANIFEST.json", checksums="SHA256SUMS",
            sbom="sbom.cdx.json", provenance="provenance.slsa.json",
        )
        if cmd_verify(args) != 0:
            raise error("self-test: healthy fixture did not verify")
        (base / "toy.bin").write_bytes(b"bad")
        if cmd_verify(args) == 0:
            raise error("self-test: tampered asset was not rejected")
    print("ok: supply-chain gate rejects a tampered asset")
    return 0


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(description=__doc__)
    sub = parser.add_subparsers(dest="command", required=True)

    prov = sub.add_parser("provenance", help="write an SLSA v1.0 provenance statement")
    prov.add_argument("--manifest", required=True)
    prov.add_argument("--out", required=True)
    prov.add_argument("--commit", default="")
    prov.add_argument("--ref", default="")
    prov.add_argument("--repository", default="")
    prov.add_argument("--server-url", default="")
    prov.add_argument("--run-id", default="")
    prov.add_argument("--run-attempt", default="")
    prov.add_argument("--tag", default="")
    prov.add_argument("--workflow-ref", default="")
    prov.add_argument("--started-on", default="")
    prov.set_defaults(func=cmd_provenance)

    verify = sub.add_parser("verify", help="fail closed unless the whole chain agrees")
    verify.add_argument("--dir", default="release")
    verify.add_argument("--manifest", default="MANIFEST.json")
    verify.add_argument("--checksums", default="SHA256SUMS")
    verify.add_argument("--sbom", default="sbom-source.cdx.json")
    verify.add_argument("--provenance", default="provenance.slsa.json")
    verify.set_defaults(func=cmd_verify)

    listing = sub.add_parser("list-assets", help="print manifest asset names")
    listing.add_argument("--dir", default="release")
    listing.add_argument("--manifest", default="MANIFEST.json")
    listing.set_defaults(func=cmd_list_assets)

    return parser


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    if argv == ["--self-test"]:
        try:
            return self_test()
        except SupplyChainError as exc:
            print(f"::error::{exc}", file=sys.stderr)
            return 1
    args = build_parser().parse_args(argv)
    try:
        return args.func(args)
    except SupplyChainError as exc:
        print(f"::error::{exc}", file=sys.stderr)
        return 1


if __name__ == "__main__":
    sys.exit(main())
