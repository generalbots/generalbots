#!/usr/bin/env python3
"""Deterministic CPU-tier wheelhouse planner/builder for the BotModels pythonbox.

The desktop bundle ships a wheelhouse so the first run never resolves torch and
friends from PyPI. This helper is the single source of truth for the tier:

* which requirement files make up the CPU tier;
* the pip download command (CPython ABI + manylinux tags);
* a size estimate used by ``install.sh`` and the wizard;
* the CPU-only invariant (torch/torchaudio/torchvision come from PyTorch's CPU
  index, so no ``nvidia-*`` CUDA wheels can leak into the archive).

Usage::

    python3 botmodels/scripts/wheelhouse.py plan
    python3 botmodels/scripts/wheelhouse.py build --output /tmp/wheelhouse
    python3 botmodels/scripts/wheelhouse.py inspect /tmp/wheelhouse
    python3 botmodels/scripts/wheelhouse.py estimate /tmp/wheelhouse

``plan`` is offline and deterministic. ``build`` needs network access; CI calls
it once per pythonbox release.
"""

from __future__ import annotations

import argparse
import json
import re
import shlex
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Iterable, Optional

ROOT = Path(__file__).resolve().parent.parent
REQUIREMENTS = (
    ROOT / "requirements.txt",
    ROOT / "requirements-barcode.txt",
)
TORCH_PACKAGES = ("torch", "torchaudio", "torchvision")
TORCH_ROOT = "torch"
TORCH_INDEX = "https://download.pytorch.org/whl/cpu"
PYPI_INDEX = "https://pypi.org/simple"
# manylinux2014 is the compatibility floor (glibc 2.17). Newer tags are included
# because onnxruntime and recent torch builds only publish manylinux_2_28.
PLATFORMS = (
    "manylinux2014_x86_64",
    "manylinux_2_17_x86_64",
    "manylinux_2_27_x86_64",
    "manylinux_2_28_x86_64",
)
GIB = 1024**3
MIN_FREE_GIB = 6
ONLINE_DOWNLOAD_GIB = 4.0
ONLINE_MIN_FREE_GIB = 12
WHEELHOUSE_INSTALL_FACTOR = 3
WHEELHOUSE_HEADROOM_GIB = 1
DEFAULT_WHEEL_BYTES = 2 * 1024**2
TRANSITIVE_BUDGET_BYTES = 250 * 1024**2

# Approximate compressed wheel sizes for the top-level packages. They are only
# used by `plan`; `inspect` reads the real wheelhouse. Kept in MiB for legibility.
SIZE_ESTIMATES_MIB = {
    "torch": 220,
    "torchvision": 8,
    "torchaudio": 4,
    "diffusers": 4,
    "transformers": 11,
    "accelerate": 0.3,
    "safetensors": 0.5,
    "pillow": 4,
    "faster-whisper": 1,
    "piper-tts": 3,
    "onnxruntime": 17,
    "av": 40,
    "scipy": 45,
    "librosa": 0.3,
    "imageio": 3,
    "imageio-ffmpeg": 30,
    "opencv-python": 65,
    "timm": 2,
    "pyzbar": 0.8,
    "pytesseract": 0.1,
    "rapidocr-onnxruntime": 15,
    "fastapi": 0.1,
    "uvicorn": 0.06,
    "pydantic": 0.4,
    "pydantic-settings": 0.03,
    "email-validator": 0.05,
    "psutil": 0.5,
    "structlog": 0.07,
    "python-json-logger": 0.01,
    "httpx": 0.4,
    "aiofiles": 0.1,
    "python-multipart": 0.03,
    "requests": 0.1,
    "prometheus-client": 0.1,
    "python-dotenv": 0.03,
    "typing-extensions": 0.05,
}

_REQUIREMENT_NAME = re.compile(r"^\s*([A-Za-z0-9][A-Za-z0-9_.-]*)")


def parse_python_version(value: str) -> tuple[str, str]:
    """Return ``(version, abi)`` for a ``MAJOR.MINOR`` string."""
    text = value.strip()
    if not re.fullmatch(r"\d+\.\d+", text):
        raise SystemExit(f"python version must look like 3.12, got {value!r}")
    return text, "cp" + text.replace(".", "")


def requirement_names(path: Path) -> list[str]:
    """Top-level distribution names from one requirements file."""
    names: list[str] = []
    for raw in path.read_text(encoding="utf-8").splitlines():
        line = raw.split("#", 1)[0].strip()
        if not line or line.startswith("-"):
            continue
        match = _REQUIREMENT_NAME.match(line)
        if match:
            name = match.group(1).lower()
            if name not in names:
                names.append(name)
    return names


def cpu_tier_names() -> list[str]:
    names: list[str] = []
    for path in REQUIREMENTS:
        for name in requirement_names(path):
            if name not in names:
                names.append(name)
    return names


def estimate_wheelhouse_bytes() -> int:
    """Deterministic download estimate for the top-level CPU tier."""
    total = TRANSITIVE_BUDGET_BYTES
    for name in cpu_tier_names():
        total += int(SIZE_ESTIMATES_MIB.get(name, DEFAULT_WHEEL_BYTES / 1024**2) * 1024**2)
    return total


def install_required_bytes(wheelhouse_bytes: int) -> int:
    """Free space required to unpack a wheelhouse (documented formula)."""
    estimate = wheelhouse_bytes * WHEELHOUSE_INSTALL_FACTOR + WHEELHOUSE_HEADROOM_GIB * GIB
    return max(MIN_FREE_GIB * GIB, estimate)


def directory_bytes(path: Path) -> int:
    return sum(entry.stat().st_size for entry in path.rglob("*") if entry.is_file())


def wheel_records(directory: Path) -> list[dict]:
    records = []
    for wheel in sorted(directory.glob("*.whl")):
        records.append({"filename": wheel.name, "size": wheel.stat().st_size})
    return records


def pip_command(python_version: str, abi: str, destination: Path) -> list[str]:
    command = [
        sys.executable,
        "-m",
        "pip",
        "download",
        "--dest",
        str(destination),
        "--only-binary=:all:",
        "--python-version",
        python_version,
        "--implementation",
        "cp",
        "--abi",
        abi,
    ]
    for platform in PLATFORMS:
        command.extend(["--platform", platform])
    return command


def build_commands(python_version: str, abi: str, destination: Path) -> list[list[str]]:
    # First pass: only the PyTorch CPU index, without dependencies. This is the
    # one place a CUDA wheel could sneak in, so do not let PyPI's torch compete.
    torch = pip_command(python_version, abi, destination) + [
        "--no-deps",
        "--index-url",
        TORCH_INDEX,
        TORCH_ROOT,
    ]
    # Second pass: everything else from PyPI, with torch pinned to the CPU
    # version downloaded above (the constraint carries the +cpu local tag). The
    # resolver is then free to pick the matching torchvision/torchaudio CPU
    # builds from the extra index instead of us guessing their versions here.
    rest = pip_command(python_version, abi, destination) + [
        "--index-url",
        PYPI_INDEX,
        "--extra-index-url",
        TORCH_INDEX,
    ]
    for path in REQUIREMENTS:
        rest.extend(["-r", str(path)])
    rest.extend(["-c", "<torch-cpu-constraints>"])
    return [torch, rest]


def torch_versions(directory: Path) -> dict[str, str]:
    versions: dict[str, str] = {}
    for wheel in directory.glob("*.whl"):
        parts = wheel.name.split("-")
        if len(parts) < 2:
            continue
        name = parts[0].lower().replace("_", "-")
        if name in TORCH_PACKAGES:
            versions[name] = parts[1]
    return versions


def run(command: list[str]) -> None:
    print("+ " + shlex.join(command), flush=True)
    subprocess.run(command, check=True)


def build(output: Path, python_version: str, dry_run: bool) -> int:
    version, abi = parse_python_version(python_version)
    output.mkdir(parents=True, exist_ok=True)
    if not dry_run:
        # A rebuild must not inherit wheels from an older tier: the manifest and
        # the archive have to describe exactly this run.
        for stale in output.glob("*.whl"):
            stale.unlink()
        (output / "wheelhouse-manifest.json").unlink(missing_ok=True)

    torch_command, rest_command = build_commands(version, abi, output)
    if dry_run:
        print("+ " + shlex.join(torch_command))
        print("+ " + shlex.join(rest_command))
        return 0

    run(torch_command)
    versions = torch_versions(output)
    if "torch" not in versions:
        raise SystemExit("the torch CPU download produced no torch wheel")
    constraints = tempfile.NamedTemporaryFile(
        "w", suffix="-torch-cpu-constraints.txt", delete=False, encoding="utf-8"
    )
    try:
        with constraints:
            for name in TORCH_PACKAGES:
                if name in versions:
                    constraints.write(f"{name}=={versions[name]}\n")
        rest_command = [part.replace("<torch-cpu-constraints>", constraints.name) for part in rest_command]
        run(rest_command)
    finally:
        Path(constraints.name).unlink(missing_ok=True)

    forbidden = [
        wheel.name
        for wheel in output.glob("*.whl")
        if wheel.name.lower().startswith(("nvidia_", "nvidia-"))
        or "cuda" in wheel.name.lower()
    ]
    if forbidden:
        raise SystemExit(
            "CPU tier invariant violated; CUDA wheels in the wheelhouse: "
            + ", ".join(sorted(forbidden))
        )

    records = wheel_records(output)
    total = sum(record["size"] for record in records)
    manifest = {
        "schema": 1,
        "python": version,
        "abi": abi,
        "platforms": list(PLATFORMS),
        "torch_index": TORCH_INDEX,
        "requirements": [path.name for path in REQUIREMENTS],
        "wheels": records,
        "total_size": total,
        "install_estimate": install_required_bytes(total),
        "min_free_bytes": MIN_FREE_GIB * GIB,
    }
    (output / "wheelhouse-manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    print(
        f"wheelhouse ready: {len(records)} wheels, "
        f"{total / GIB:.2f} GiB archived, "
        f"{install_required_bytes(total) / GIB:.1f} GiB install budget"
    )
    return 0


def plan(as_json: bool) -> int:
    names = cpu_tier_names()
    estimated = estimate_wheelhouse_bytes()
    version, abi = parse_python_version("3.12")
    torch_command, rest_command = build_commands(version, abi, Path("<wheelhouse>"))
    payload = {
        "requirements": [path.name for path in REQUIREMENTS],
        "packages": names,
        "python": version,
        "abi": abi,
        "platforms": list(PLATFORMS),
        "torch_index": TORCH_INDEX,
        "estimated_wheelhouse_bytes": estimated,
        "install_required_bytes": install_required_bytes(estimated),
        "min_free_bytes": MIN_FREE_GIB * GIB,
        "online_download_bytes": int(ONLINE_DOWNLOAD_GIB * GIB),
        "online_min_free_bytes": ONLINE_MIN_FREE_GIB * GIB,
        "commands": [shlex.join(torch_command), shlex.join(rest_command)],
    }
    if as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    print("BotModels CPU-tier wheelhouse")
    print(f"  requirements: {', '.join(payload['requirements'])}")
    print(f"  target:       CPython {version} ({abi}), linux x86_64")
    print(f"  platforms:    {', '.join(PLATFORMS)}")
    print(f"  torch index:  {TORCH_INDEX}")
    print(f"  packages:     {', '.join(names)}")
    print(
        f"  estimate:     {estimated / GIB:.2f} GiB download, "
        f"{install_required_bytes(estimated) / GIB:.1f} GiB install budget"
    )
    print(f"  minimum free: {MIN_FREE_GIB} GiB")
    print("  commands:")
    for command in payload["commands"]:
        print("    " + command)
    return 0


def inspect(directory: Path, as_json: bool) -> int:
    if not directory.is_dir():
        raise SystemExit(f"wheelhouse not found: {directory}")
    records = wheel_records(directory)
    total = sum(record["size"] for record in records)
    payload = {
        "directory": str(directory),
        "wheels": records,
        "count": len(records),
        "total_size": total,
        "install_required_bytes": install_required_bytes(total),
    }
    if as_json:
        print(json.dumps(payload, indent=2, sort_keys=True))
        return 0
    for record in records:
        print(f"{record['size']:>12}  {record['filename']}")
    print(
        f"{len(records)} wheels, {total / GIB:.2f} GiB archived, "
        f"{install_required_bytes(total) / GIB:.1f} GiB install budget"
    )
    return 0


def main(argv: Optional[Iterable[str]] = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = parser.add_subparsers(dest="command", required=True)

    plan_parser = sub.add_parser("plan", help="list the CPU-tier wheel set and size estimate")
    plan_parser.add_argument("--json", action="store_true")

    build_parser = sub.add_parser("build", help="download the CPU-tier wheelhouse")
    build_parser.add_argument("--output", required=True, type=Path)
    build_parser.add_argument("--python-version", default="3.12")
    build_parser.add_argument("--dry-run", action="store_true")

    inspect_parser = sub.add_parser("inspect", help="list an existing wheelhouse")
    inspect_parser.add_argument("directory", type=Path)
    inspect_parser.add_argument("--json", action="store_true")

    estimate_parser = sub.add_parser("estimate", help="print the install space budget")
    estimate_parser.add_argument("directory", type=Path)

    args = parser.parse_args(list(argv) if argv is not None else None)
    if args.command == "plan":
        return plan(args.json)
    if args.command == "build":
        return build(args.output, args.python_version, args.dry_run)
    if args.command == "inspect":
        return inspect(args.directory, args.json)
    if args.command == "estimate":
        if not args.directory.is_dir():
            raise SystemExit(f"wheelhouse not found: {args.directory}")
        total = directory_bytes(args.directory)
        print(install_required_bytes(total))
        return 0
    parser.error(f"unknown command: {args.command}")
    return 2


if __name__ == "__main__":
    raise SystemExit(main())
