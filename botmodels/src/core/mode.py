"""Model tier resolution: min / max, decided once at startup.

The mode is frozen for the process lifetime. Flipping tiers mid-run would unload
models from under in-flight requests, so `resolve()` is called exactly once from
`lifespan` and the result is read-only thereafter.
"""

from typing import Literal, Optional

from .hardware import (
    MIN_RAM_GB,
    MIN_RAM_FOR_MAX_GB,
    MIN_VRAM_GB,
    MIN_VRAM_FOR_MAX_GB,
    HardwareProfile,
    probe,
)
from .logging import get_logger

logger = get_logger("mode")

Mode = Literal["min", "max"]
ModeSetting = Literal["auto", "min", "max"]

VALID_SETTINGS: tuple[str, ...] = ("auto", "min", "max")

# Per-capability backend table. Music is identical in both modes (ACE-Step 1.5),
# so it is not keyed by tier.
BACKENDS: dict[str, dict[str, str]] = {
    "min": {
        "image": "sd-turbo",
        "vision": "blip2",
        "stt": "faster-whisper",
        "tts": "piper",
        "ocr": "tesseract",
        "video": "zeroscope",
    },
    "max": {
        "image": "qwen-image",
        "vision": "qwen3-vl",
        "stt": "qwen3-asr",
        "tts": "kokoro",
        "ocr": "paddleocr-vl",
        "video": "wan22",
    },
}

# Always the same regardless of tier.
SHARED_BACKENDS: dict[str, str] = {"music": "ace-step-1.5"}


class InvalidModeError(ValueError):
    """Raised when BOTMODELS_MODE is set to something outside {auto,min,max}."""


def _tier_for(hw: HardwareProfile) -> Mode:
    """Map a hardware profile onto a tier.

    Unknown values (no RAM probe, no GPU) resolve to `min` so a bare container
    never tries to load a 7B model.
    """
    if not hw.is_gpu:
        logger.info("No GPU detected, using min tier", device=hw.device)
        return "min"

    # Under unified memory (Apple Silicon) the GPU total_memory figure is not
    # meaningful, so key the tier off host RAM instead.
    if hw.mps:
        ram = hw.ram_gb or 0
        if ram >= MIN_RAM_FOR_MAX_GB:
            logger.info("MPS with sufficient RAM, using max tier", ram_gb=ram)
            return "max"
        logger.info("MPS with limited RAM, using min tier", ram_gb=ram)
        return "min"

    vram = hw.vram_gb or 0
    ram = hw.ram_gb or 0

    if vram >= MIN_VRAM_FOR_MAX_GB and ram >= MIN_RAM_FOR_MAX_GB:
        logger.info("GPU with headroom, using max tier", vram_gb=vram, ram_gb=ram)
        return "max"

    logger.info(
        "GPU below max threshold, using min tier",
        vram_gb=vram,
        ram_gb=ram,
        need_vram_gb=MIN_VRAM_FOR_MAX_GB,
        need_ram_gb=MIN_RAM_FOR_MAX_GB,
    )
    return "min"


def resolve_mode(setting: Optional[str]) -> tuple[Mode, str, HardwareProfile]:
    """Resolve the tier.

    Returns (mode, mode_source, hardware) where mode_source is "env" when the
    tier was forced and "auto" when it was detected.

    Raises InvalidModeError on an unrecognised BOTMODELS_MODE value, so a typo
    fails fast at startup rather than silently degrading.
    """
    raw = (setting or "auto").strip().lower()

    if raw not in VALID_SETTINGS:
        raise InvalidModeError(
            f"BOTMODELS_MODE must be one of {VALID_SETTINGS}, got {raw!r}"
        )

    hw = probe()

    if raw == "auto":
        mode = _tier_for(hw)
        source = "auto"
        logger.info("Mode resolved by hardware probe", mode=mode, **hw.to_dict())
        return mode, source, hw

    mode = raw  # type: ignore[return-value]
    source = "env"
    logger.info(
        "Mode forced by environment",
        mode=mode,
        detected_device=hw.device,
        detected_vram_gb=hw.vram_gb,
        detected_ram_gb=hw.ram_gb,
    )
    return mode, source, hw


def backends_for(mode: Mode) -> dict[str, str]:
    """The resolved backend per capability for a tier."""
    return {**BACKENDS[mode], **SHARED_BACKENDS}


def summary(mode: Mode, mode_source: str, hw: HardwareProfile) -> dict:
    """Payload for /api/health."""
    return {
        "mode": mode,
        "mode_source": mode_source,
        "detected": hw.to_dict(),
        "backends": backends_for(mode),
        "thresholds": {
            "min_vram_gb": MIN_VRAM_GB,
            "min_ram_gb": MIN_RAM_GB,
            "max_vram_gb": MIN_VRAM_FOR_MAX_GB,
            "max_ram_gb": MIN_RAM_FOR_MAX_GB,
        },
    }