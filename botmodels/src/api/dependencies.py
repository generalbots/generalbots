import hmac
from typing import Optional

from fastapi import Header, HTTPException, Query

from ..core.config import settings


def _compare(candidate: str, expected: str) -> bool:
    """Constant-time comparison. `!=` leaks length and prefix information."""
    return hmac.compare_digest(candidate.encode("utf-8"), expected.encode("utf-8"))


async def verify_api_key(x_api_key: str = Header(...)):
    """Standard header auth for HTTP routes. Raises 401 on mismatch."""
    if not _compare(x_api_key, settings.api_key):
        raise HTTPException(status_code=401, detail="Invalid API key")
    return x_api_key


def verify_api_key_optional(x_api_key: Optional[str] = Header(None)):
    """Auth that reports validity instead of raising.

    Returns the key when valid, None otherwise. Callers that need the result must
    check for None — the previous implementation returned None for a missing key
    *and* a wrong one, and the /api/detect endpoint then ran unauthenticated.
    """
    if x_api_key is None:
        return None
    if not _compare(x_api_key, settings.api_key):
        return None
    return x_api_key


def require_api_key(x_api_key: Optional[str] = Header(None)):
    """Header auth that tolerates a missing header. Raises 401 regardless."""
    if x_api_key is None:
        raise HTTPException(status_code=401, detail="Missing API key")
    if not _compare(x_api_key, settings.api_key):
        raise HTTPException(status_code=401, detail="Invalid API key")
    return x_api_key


def verify_ws_key(
    x_api_key: Optional[str] = Query(None, alias="api_key"),
    token: Optional[str] = Query(None),
) -> str:
    """Auth for WebSocket upgrades.

    A Header dependency cannot run on a WS upgrade here, so the key arrives as a
    query parameter. Both voice sockets were previously unauthenticated.
    """
    candidate = x_api_key or token
    if candidate is None:
        raise HTTPException(status_code=401, detail="Missing API key")
    if not _compare(candidate, settings.api_key):
        raise HTTPException(status_code=401, detail="Invalid API key")
    return candidate