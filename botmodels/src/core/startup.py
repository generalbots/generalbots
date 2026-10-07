"""Startup finalization for BotModels.

Resolves the model tier from BOTMODELS_MODE (or hardware detection) exactly
once, records it on the settings object, and exposes the payload for
/api/health. The resolved tier is immutable for the process lifetime: switching
tiers mid-run would unload models from under in-flight requests.
"""

from typing import Optional

from .config import settings
from .hardware import HardwareProfile
from .logging import get_logger
from .mode import InvalidModeError, backends_for, resolve_mode, summary

logger = get_logger("startup")

_finalized = False
_cached_hw: Optional[HardwareProfile] = None


def finalize_mode() -> None:
    """Resolve and freeze the model tier. Idempotent.

    Raises InvalidModeError if BOTMODELS_MODE is not auto/min/max, so a typo
    fails at startup instead of silently degrading.
    """
    global _finalized
    if _finalized:
        return

    global _cached_hw

    mode, source, hw = resolve_mode(settings.mode)

    settings.resolved_mode = mode
    settings.mode_source = source
    _cached_hw = hw
    _finalized = True

    if not settings.device:
        settings.device = hw.device

    logger.info(
        "Model tier resolved",
        mode=mode,
        source=source,
        device=settings.device,
        backends=backends_for(mode),
    )


def is_finalized() -> bool:
    return _finalized


def health_summary() -> dict:
    """Mode payload for /api/health. Safe to call before finalization."""
    if not _finalized or settings.resolved_mode is None:
        return {
            "mode": None,
            "mode_source": settings.mode_source,
            "detail": "mode not resolved yet",
        }

    if _cached_hw is None:
        return {
            "mode": settings.resolved_mode,
            "mode_source": settings.mode_source,
            "detail": "hardware profile unavailable",
        }
    return summary(settings.resolved_mode, settings.mode_source, _cached_hw)


__all__ = [
    "InvalidModeError",
    "finalize_mode",
    "health_summary",
    "is_finalized",
]