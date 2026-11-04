#!/usr/bin/env python3
"""Provision the open-source model bundle BotModels runs locally.

The chat model stays external; every other model-backed capability is served
from this host. This script does not install Python packages (the environment
manager owns that) — it verifies the imports and fetches the weights.

Examples
--------
    # Verify everything the min tier needs, without touching the network.
    python scripts/provision_models.py --check

    # Verify only the importable dependencies (safe in CI, no weights).
    python scripts/provision_models.py --check --deps-only

    # Download the min-tier weights into the Hugging Face cache and the voice.
    python scripts/provision_models.py --install

    # Show the max-tier checkpoints (not downloaded by default).
    python scripts/provision_models.py --tier max --check
"""

from __future__ import annotations

import argparse
import hashlib
import importlib.util
import json
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MODELS_DIR = Path(
    __import__("os").environ.get("BOTMODELS_MODELS_DIR", ROOT / "models")
).resolve()
MANIFEST = MODELS_DIR / "manifest.json"
PIPER_VOICE = "en_US-lessac-medium"

# Entries are intentionally declarative: the health endpoint and the docs read
# the same list, so a new checkpoint cannot be documented without being checked.
MIN_ENTRIES = [
    {
        "name": "whisper-small",
        "kind": "hf",
        "repo": "Systran/faster-whisper-small",
        "module": "faster_whisper",
        "capability": "stt",
        "note": "faster-whisper small, int8 CPU (~460 MB)",
    },
    {
        "name": "blip-caption-base",
        "kind": "hf",
        "repo": "Salesforce/blip-image-captioning-base",
        "module": "transformers",
        "capability": "vision",
        "note": "BLIP base captioner (~990 MB float32), fits <8 GB RAM",
    },
    {
        "name": "piper-lessac-medium",
        "kind": "file",
        "path": f"piper/{PIPER_VOICE}.onnx",
        "module": "piper",
        "capability": "tts",
        "note": "Piper voice (~61 MB)",
    },
    {
        "name": "rapidocr",
        "kind": "module",
        "module": "rapidocr_onnxruntime",
        "capability": "ocr",
        "note": "PP-OCR ONNX fallback when the tesseract binary is absent",
    },
]

MAX_ENTRIES = [
    {
        "name": "qwen3-vl-4b",
        "kind": "hf",
        "repo": "Qwen/Qwen3-VL-4B-Instruct",
        "module": "transformers",
        "capability": "vision",
        "note": "max tier VQA/OCR (needs newer transformers + >=16 GB VRAM)",
    },
    {
        "name": "qwen3-asr",
        "kind": "hf",
        "repo": "Qwen/Qwen3-ASR-1.7B",
        "module": "transformers",
        "capability": "stt",
        "note": "max tier multilingual ASR",
    },
    {
        "name": "moondream2",
        "kind": "hf",
        "repo": "vikhyatk/moondream2",
        "module": "transformers",
        "capability": "vision",
        "note": "CPU-capable VQA alternative to BLIP (~3.7 GB fp32); "
        "select with BOTMODELS_VISION_BACKEND=moondream",
    },
    {
        "name": "kokoro-82m",
        "kind": "hf",
        "repo": "hexgrad/Kokoro-82M",
        "module": "kokoro",
        "capability": "tts",
        "note": "max tier CPU TTS",
    },
    {
        "name": "qwen-image",
        "kind": "hf",
        "repo": "Qwen/Qwen-Image",
        "module": "diffusers",
        "capability": "image",
        "note": "max tier image generation/editing",
    },
    {
        "name": "wan2.2-ti2v-5b",
        "kind": "hf",
        "repo": "Wan-AI/Wan2.2-TI2V-5B",
        "module": "diffusers",
        "capability": "video",
        "note": "max tier 720p text/image-to-video",
    },
    {
        "name": "paddleocr-vl",
        "kind": "hf",
        "repo": "PaddlePaddle/PaddleOCR-VL",
        "module": "paddleocr",
        "capability": "ocr",
        "note": "max tier structured markdown OCR",
    },
]


def module_available(name: str) -> bool:
    """Check an import without paying the cost of importing torch."""
    try:
        return importlib.util.find_spec(name) is not None
    except (ImportError, ValueError):
        return False


def _hf_snapshot(repo: str, local_only: bool) -> str | None:
    try:
        from huggingface_hub import snapshot_download
    except ImportError:
        return None
    try:
        return snapshot_download(repo_id=repo, local_files_only=local_only)
    except Exception:  # noqa: BLE001 - cache miss, offline, or network error
        return None


def _dir_size(path: str) -> int:
    total = 0
    for item in Path(path).rglob("*"):
        try:
            if item.is_file():
                total += item.stat().st_size
        except OSError:
            continue
    return total


def _sha256_path(path: str) -> str | None:
    """Deterministic digest of a file or a snapshot directory.

    Directory digests include each relative path so two snapshots with the
    same files in different layouts do not collide.
    """
    target = Path(path)
    if target.is_file():
        digest = hashlib.sha256()
        with target.open("rb") as handle:
            for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                digest.update(chunk)
        return digest.hexdigest()
    if target.is_dir():
        digest = hashlib.sha256()
        for item in sorted(target.rglob("*")):
            if not item.is_file():
                continue
            digest.update(item.relative_to(target).as_posix().encode("utf-8"))
            with item.open("rb") as handle:
                for chunk in iter(lambda: handle.read(1024 * 1024), b""):
                    digest.update(chunk)
        return digest.hexdigest()
    return None


def _piper_install() -> bool:
    target = MODELS_DIR / "piper"
    target.mkdir(parents=True, exist_ok=True)
    command = [
        sys.executable,
        "-m",
        "piper.download_voices",
        "--download-dir",
        str(target),
        PIPER_VOICE,
    ]
    try:
        completed = subprocess.run(command, check=False)
        return completed.returncode == 0
    except OSError:
        return False


def evaluate(entry: dict, install: bool, deps_only: bool) -> dict:
    status = {
        "name": entry["name"],
        "kind": entry["kind"],
        "capability": entry["capability"],
        "note": entry["note"],
        "module": entry["module"],
        "module_ok": module_available(entry["module"]),
        "status": "unknown",
        "path": None,
        "size_bytes": 0,
        "sha256": None,
    }

    if not status["module_ok"]:
        status["status"] = "missing-module"
        return status

    if deps_only or entry["kind"] == "module":
        status["status"] = "ok"
        return status

    if entry["kind"] == "file":
        path = MODELS_DIR / entry["path"]
        if not path.exists() and install:
            _piper_install()
        if path.exists():
            status["status"] = "ok"
            status["path"] = str(path)
            status["size_bytes"] = path.stat().st_size
            status["sha256"] = _sha256_path(str(path))
        else:
            status["status"] = "missing-weights"
        return status

    if entry["kind"] == "hf":
        path = _hf_snapshot(entry["repo"], local_only=not install)
        if path is None:
            status["status"] = "missing-weights"
            return status
        status["status"] = "ok"
        status["path"] = path
        status["size_bytes"] = _dir_size(path)
        status["sha256"] = _sha256_path(path)
        return status

    status["status"] = "unknown-kind"
    return status


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--tier", choices=("min", "max"), default="min")
    parser.add_argument(
        "--check", action="store_true", help="verify without downloading (default)"
    )
    parser.add_argument(
        "--install", action="store_true", help="download missing weights"
    )
    parser.add_argument(
        "--deps-only",
        action="store_true",
        help="only verify importable dependencies (CI-safe)",
    )
    parser.add_argument("--json", action="store_true", help="machine-readable output")
    args = parser.parse_args()

    if args.check and args.install:
        parser.error("--check and --install are mutually exclusive")
    install = args.install

    entries = MIN_ENTRIES if args.tier == "min" else MAX_ENTRIES
    results = [evaluate(entry, install=install, deps_only=args.deps_only) for entry in entries]

    # A check compares against the last install manifest, so a truncated or
    # swapped checkpoint is reported instead of silently passing on size alone.
    previous = {}
    if MANIFEST.exists() and not args.deps_only:
        try:
            previous = {
                item["name"]: item
                for item in json.loads(MANIFEST.read_text(encoding="utf-8")).get("entries", [])
            }
        except (OSError, ValueError, KeyError):
            previous = {}
        for result in results:
            stored = previous.get(result["name"], {}).get("sha256")
            if (
                result["status"] == "ok"
                and stored
                and result.get("sha256")
                and stored != result["sha256"]
            ):
                result["status"] = "hash-mismatch"

    ok = all(result["status"] == "ok" for result in results)

    manifest = {
        "schema": 1,
        "tier": args.tier,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "deps_only": args.deps_only,
        "ok": ok,
        "entries": results,
    }

    if install:
        MODELS_DIR.mkdir(parents=True, exist_ok=True)
        MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    if args.json:
        print(json.dumps(manifest, indent=2))
        return 0 if ok else 1

    print(f"BotModels provisioning — tier={args.tier} mode={'install' if install else 'check'}")
    for result in results:
        size = f"{result['size_bytes'] / (1024 * 1024):.0f} MB" if result["size_bytes"] else "-"
        print(f"  {result['status']:<15} {result['name']:<24} {result['capability']:<9} {size}")
        if result["status"] != "ok":
            print(f"      {result['note']}")
    if args.tier == "max":
        print("  max tier is optional; it needs >=16 GB VRAM and newer transformers.")
    if args.deps_only:
        print("  dependencies only; weights were not checked.")
    print("OK" if ok else "INCOMPLETE — run with --install (or install the missing modules)")
    return 0 if ok else 1


if __name__ == "__main__":
    sys.exit(main())
