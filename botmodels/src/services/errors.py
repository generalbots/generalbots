"""Shared errors for BotModels services."""

from typing import Optional


class BotModelsError(Exception):
    """Base class for service errors."""


class BackendNotLoadedError(BotModelsError):
    """A backend was selected but its model failed to load.

    Carries enough context for an HTTP 503 to name the backend and the cause,
    instead of surfacing a TypeError from calling None.
    """

    def __init__(
        self,
        capability: str,
        backend: str,
        cause: Optional[str] = None,
    ):
        self.capability = capability
        self.backend = backend
        self.cause = cause
        detail = f"{capability} backend {backend!r} is not loaded"
        if cause:
            detail = f"{detail}: {cause}"
        super().__init__(detail)


class UnsupportedBackendError(BotModelsError):
    """The requested backend is not registered for this capability."""

    def __init__(self, capability: str, backend: str, available: list[str]):
        self.capability = capability
        self.backend = backend
        self.available = available
        super().__init__(
            f"Unknown {capability} backend {backend!r}; available: {available}"
        )