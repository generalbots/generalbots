"""Backend registry.

One instance per capability, created once from the resolved tier and cached for
the process lifetime. The tier is frozen at startup, so the registry never
re-resolves.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ...core.mode import backends_for
from ..errors import BackendNotLoadedError, UnsupportedBackendError
from .base import Backend
from .image import IMAGE_BACKENDS
from .ocr import OCR_BACKENDS
from .speech import STT_BACKENDS, TTS_BACKENDS
from .video import VIDEO_BACKENDS
from .vision import VISION_BACKENDS

logger = get_logger("registry")

REGISTRY: dict[str, dict[str, type[Backend]]] = {
    "image": IMAGE_BACKENDS,
    "vision": VISION_BACKENDS,
    "stt": STT_BACKENDS,
    "tts": TTS_BACKENDS,
    "ocr": OCR_BACKENDS,
    "video": VIDEO_BACKENDS,
}

_instances: dict[str, Backend] = {}


def build(capability: str) -> Backend:
    """Instantiate the backend this capability resolves to under the current tier."""
    if capability not in REGISTRY:
        raise UnsupportedBackendError(capability, capability, list(REGISTRY))

    import os

    tier_selected = backends_for(settings.resolved_mode or "min")[capability]
    override = (os.environ.get(f"BOTMODELS_{capability.upper()}_BACKEND") or "").strip()
    selected = override or tier_selected
    registry = REGISTRY[capability]

    if selected not in registry:
        if override:
            logger.warning(
                "Ignoring unknown backend override",
                capability=capability,
                override=selected,
                fallback=tier_selected,
            )
            selected = tier_selected
        else:
            raise UnsupportedBackendError(capability, selected, list(registry))

    # Prefer the tier's backend, but fall back to another registered backend
    # when the preferred one cannot even import (e.g. tesseract's system
    # binary is absent and RapidOCR is installed). Model-weight failures are
    # handled at load time, not here: probing must stay cheap.
    for name in [selected, *[n for n in registry if n != selected]]:
        instance = registry[name]()
        try:
            available = instance.is_available()
        except Exception:  # noqa: BLE001 - availability probes must not crash
            available = False
        if available:
            if name != selected:
                logger.warning(
                    "Preferred backend unavailable, using fallback",
                    capability=capability,
                    preferred=selected,
                    selected=name,
                )
            return instance

    raise BackendNotLoadedError(
        capability,
        selected,
        f"no registered backend is available: {list(registry)}",
    )


def get(capability: str) -> Backend:
    """Cached backend instance for a capability."""
    if capability not in _instances:
        instance = build(capability)
        logger.info(
            "Backend registered", capability=capability, backend=instance.name
        )
        _instances[capability] = instance
    return _instances[capability]


def status() -> dict:
    """Per-capability backend status for /api/health.

    Reports availability without loading anything — probing must stay cheap
    enough for a health endpoint.
    """
    report: dict[str, dict] = {}
    for capability, backend in _instances.items():
        report[capability] = backend.status()
    return report


def reset() -> None:
    """Drop cached instances. For tests only."""
    _instances.clear()