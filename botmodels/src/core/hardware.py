"""Hardware detection for BotModels mode resolution.

Probes CUDA/MPS availability, GPU VRAM and host RAM so the service can pick a
model tier at startup. Detection is deliberately dependency-light: psutil is
used when installed, with /proc/meminfo and sysctl fallbacks so the service
still resolves a tier without it.
"""

import os
import platform
import sys
from dataclasses import asdict, dataclass
from typing import Optional

from .logging import get_logger

logger = get_logger("hardware")

# Tier thresholds (GB). `max` requires all three.
MIN_VRAM_FOR_MAX_GB = 16
MIN_RAM_FOR_MAX_GB = 32

# Below this we stay on the light tier even with a large GPU.
MIN_VRAM_GB = 8
MIN_RAM_GB = 16


@dataclass(frozen=True)
class HardwareProfile:
    """What the host actually offers, as probed at startup."""

    device: str
    cuda: bool
    mps: bool
    vram_gb: Optional[float]
    ram_gb: Optional[float]
    cpu_count: Optional[int]
    platform: str
    python: str

    @property
    def is_gpu(self) -> bool:
        return self.cuda or self.mps

    def to_dict(self) -> dict:
        data = asdict(self)
        data["is_gpu"] = self.is_gpu
        return data


def _visible_devices_restricted() -> bool:
    """True when CUDA_VISIBLE_DEVICES explicitly hides every device."""
    value = os.environ.get("CUDA_VISIBLE_DEVICES")
    if value is None:
        return False
    stripped = value.strip()
    return stripped == "" or stripped == "-1"


def _probe_cuda() -> tuple[bool, Optional[float]]:
    """Return (cuda_available, vram_gb). Honours CUDA_VISIBLE_DEVICES."""
    if _visible_devices_restricted():
        logger.info("CUDA_VISIBLE_DEVICES hides all devices, treating as CPU")
        return False, None

    try:
        import torch
    except ImportError:
        logger.info("torch not installed, CUDA unavailable")
        return False, None

    try:
        if not torch.cuda.is_available() or torch.cuda.device_count() == 0:
            return False, None
        total = torch.cuda.get_device_properties(0).total_memory
        return True, round(total / (1024**3), 1)
    except Exception as exc:  # noqa: BLE001 - probing must never crash startup
        logger.warning("CUDA probe failed, assuming unavailable", error=str(exc))
        return False, None


def _probe_mps() -> bool:
    """Apple Silicon Metal Performance Shaders availability."""
    if sys.platform != "darwin":
        return False
    try:
        import torch

        return bool(torch.backends.mps.is_available())
    except Exception:  # noqa: BLE001
        return False


def _ram_from_proc() -> Optional[float]:
    """Linux /proc/meminfo MemTotal in GB."""
    try:
        with open("/proc/meminfo", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("MemTotal:"):
                    kb = int(line.split()[1])
                    return round(kb / (1024**2), 1)
    except (OSError, ValueError, IndexError):
        return None
    return None


def _ram_from_sysctl() -> Optional[float]:
    """macOS `sysctl -n hw.memsize` in GB."""
    if sys.platform != "darwin":
        return None
    try:
        import subprocess

        raw = subprocess.run(
            ["sysctl", "-n", "hw.memsize"],
            capture_output=True,
            text=True,
            check=True,
            timeout=5,
        ).stdout.strip()
        return round(int(raw) / (1024**3), 1)
    except Exception:  # noqa: BLE001
        return None


def _probe_ram() -> Optional[float]:
    """Host RAM in GB: psutil when available, else /proc or sysctl."""
    try:
        import psutil

        return round(psutil.virtual_memory().total / (1024**3), 1)
    except ImportError:
        logger.info("psutil not installed, using /proc/meminfo or sysctl fallback")
    except Exception as exc:  # noqa: BLE001
        logger.warning("psutil RAM probe failed, using fallback", error=str(exc))

    for probe in (_ram_from_proc, _ram_from_sysctl):
        value = probe()
        if value is not None:
            return value
    return None


def probe() -> HardwareProfile:
    """Probe the host once. Never raises."""
    cuda, vram_gb = _probe_cuda()
    mps = False if cuda else _probe_mps()

    if cuda:
        device = "cuda"
    elif mps:
        device = "mps"
    else:
        device = "cpu"

    return HardwareProfile(
        device=device,
        cuda=cuda,
        mps=mps,
        vram_gb=vram_gb,
        ram_gb=_probe_ram(),
        cpu_count=os.cpu_count(),
        platform=platform.platform(),
        python=sys.version.split()[0],
    )