"""Backend base class: lazy load guarded by a lock, cached per process.

Subclasses implement `_load()` and `is_available()`. `ensure_loaded()` performs
the load at most once even under concurrent first requests, and raises
BackendNotLoadedError carrying the cause when the model cannot be loaded, so the
caller returns a clean 503 rather than calling None.
"""

import asyncio
import threading
from typing import Optional

from ...core.logging import get_logger
from ..errors import BackendNotLoadedError

logger = get_logger("backends")


class Backend:
    """Base class for all model backends."""

    name: str = "base"
    capability: str = "base"

    def __init__(self) -> None:
        self._loaded = False
        self._load_error: Optional[str] = None
        self._async_lock = asyncio.Lock()
        self._thread_lock = threading.Lock()

    @property
    def loaded(self) -> bool:
        return self._loaded

    @property
    def load_error(self) -> Optional[str]:
        return self._load_error

    def is_available(self) -> bool:
        """Whether the backend's dependencies are importable. Never loads."""
        return True

    def _load(self) -> None:
        """Load weights. Raise on failure; the caller records the cause."""
        raise NotImplementedError

    def ensure_loaded(self) -> None:
        """Load once. Thread-safe; a failed load is not retried per request."""
        if self._loaded:
            return

        with self._thread_lock:
            if self._loaded:
                return
            if self._load_error is not None:
                # Already failed: do not re-run from_pretrained on every request.
                raise BackendNotLoadedError(
                    self.capability, self.name, self._load_error
                )
            try:
                self._load()
            except Exception as exc:  # noqa: BLE001 - recorded and re-raised as 503
                self._load_error = str(exc)
                logger.warning(
                    "Backend failed to load",
                    capability=self.capability,
                    backend=self.name,
                    error=self._load_error,
                )
                raise BackendNotLoadedError(
                    self.capability, self.name, self._load_error
                ) from exc
            self._loaded = True
            logger.info("Backend loaded", capability=self.capability, backend=self.name)

    async def ensure_loaded_async(self) -> None:
        """Async entry point; serialises concurrent first requests."""
        if self._loaded:
            return
        async with self._async_lock:
            # Blocking load runs in the default thread pool, not the event loop.
            await asyncio.to_thread(self.ensure_loaded)

    def reset(self) -> None:
        """Clear load state. For tests only."""
        self._loaded = False
        self._load_error = None

    def status(self) -> dict:
        return {
            "backend": self.name,
            "capability": self.capability,
            "loaded": self._loaded,
            "available": self.is_available(),
            "error": self._load_error,
        }