"""Backend registry.

One instance per capability, created once from the resolved tier and cached for
the process lifetime. The tier is frozen at startup, so the registry never
re-resolves.
"""

from typing import Optional

from ...core.config import settings
from ...core.logging import get_logger
from ...core.mode import backends_for
from ..errors import UnsupportedBackendError
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

    selected = backends_for(settings.resolved_mode or "min")[capability]
    registry = REGISTRY[capability]

    if selected not in registry:
        raise UnsupportedBackendError(capability, selected, list(registry))

    return registry[selected]()


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