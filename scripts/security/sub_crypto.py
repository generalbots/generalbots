#!/usr/bin/env python3
"""Signing, replay protection and opt-in tokens for advisory channels (#2719).

Outbound webhooks and the two email links share one primitive: HMAC-SHA256 over
a canonical byte string. Nothing here accepts an empty secret, an unsigned
delivery or a repeated nonce, and every comparison is constant time.
"""

from __future__ import annotations

import base64
import hashlib
import hmac
import json
import secrets
import threading
import time

MAX_SKEW_SECONDS = 300
NONCE_BYTES = 16
MIN_SECRET_BYTES = 32
MAX_BODY_BYTES = 2 * 1024 * 1024
SIGNATURE_PREFIX = "v1="
TOKEN_TTL_DAYS = 30
UNSUBSCRIBE_TTL_DAYS = 365


class SubscriptionError(ValueError):
    """A subscription delivery or token failed closed."""


def check_secret(secret: str) -> bytes:
    """A webhook/token secret must be high entropy; never accept an empty one."""
    if not isinstance(secret, str) or len(secret) < MIN_SECRET_BYTES:
        raise SubscriptionError("secret is missing or too short")
    return secret.encode("utf-8")


def new_nonce() -> str:
    return secrets.token_hex(NONCE_BYTES)


def canonical_body(payload: dict) -> bytes:
    """Deterministic bytes: the signature covers exactly what the peer sees."""
    body = json.dumps(payload, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
    encoded = body.encode("utf-8")
    if len(encoded) > MAX_BODY_BYTES:
        raise SubscriptionError("webhook body exceeds the size limit")
    return encoded


def _signing_input(body: bytes, timestamp: int, nonce: str) -> bytes:
    digest = hashlib.sha256(body).hexdigest().encode("ascii")
    return b"GBADV1\n" + str(timestamp).encode("ascii") + b"\n" + nonce.encode("ascii") + b"\n" + digest + b"\n"


def sign(secret: str, body: bytes, timestamp: int, nonce: str) -> str:
    key = check_secret(secret)
    if not isinstance(timestamp, int) or timestamp <= 0:
        raise SubscriptionError("invalid timestamp")
    if len(nonce) != NONCE_BYTES * 2 or any(c not in "0123456789abcdef" for c in nonce):
        raise SubscriptionError("invalid nonce")
    mac = hmac.new(key, _signing_input(body, timestamp, nonce), hashlib.sha256).hexdigest()
    return SIGNATURE_PREFIX + mac


def _b64url(raw: bytes) -> str:
    return base64.urlsafe_b64encode(raw).rstrip(b"=").decode("ascii")


def _unb64url(text: str) -> bytes:
    if not isinstance(text, str) or not text or any(c not in "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_" for c in text):
        raise SubscriptionError("malformed base64url value")
    return base64.urlsafe_b64decode(text + "=" * (-len(text) % 4))


class ReplayGuard:
    """Bounded, TTL-limited nonce cache. One namespace per signing secret."""

    def __init__(self, ttl_seconds: int = MAX_SKEW_SECONDS * 2, capacity: int = 8192) -> None:
        if ttl_seconds <= 0 or capacity <= 0:
            raise SubscriptionError("invalid replay guard configuration")
        self._ttl = ttl_seconds
        self._capacity = capacity
        self._seen: dict[tuple[str, str], float] = {}
        self._lock = threading.Lock()

    def _namespace(self, secret: str) -> str:
        return hashlib.sha256(check_secret(secret)).hexdigest()[:16]

    def consume(self, secret: str, nonce: str, now: float | None = None) -> None:
        """Record a nonce once. Raises if it was already used (a replay)."""
        instant = time.time() if now is None else now
        key = (self._namespace(secret), nonce)
        with self._lock:
            expired = [item for item, until in self._seen.items() if until <= instant]
            for item in expired:
                del self._seen[item]
            if key in self._seen:
                raise SubscriptionError("replayed delivery: nonce already seen")
            if len(self._seen) >= self._capacity:
                oldest = min(self._seen, key=lambda item: self._seen[item])
                del self._seen[oldest]
            self._seen[key] = instant + self._ttl

    def size(self) -> int:
        with self._lock:
            return len(self._seen)


def verify(
    secret: str,
    body: bytes,
    timestamp: int,
    nonce: str,
    signature: str,
    guard: ReplayGuard | None = None,
    now: int | None = None,
    max_skew: int = MAX_SKEW_SECONDS,
) -> None:
    """Verify a delivery end to end. Raises SubscriptionError on any failure.

    Order matters: cheap shape and clock checks first, the HMAC before the
    replay cache, so an attacker cannot poison the cache with forged nonces.
    """
    key = check_secret(secret)
    if not isinstance(timestamp, int) or timestamp <= 0:
        raise SubscriptionError("invalid timestamp")
    if len(nonce) != NONCE_BYTES * 2 or any(c not in "0123456789abcdef" for c in nonce):
        raise SubscriptionError("invalid nonce")
    current = int(time.time()) if now is None else now
    if abs(current - timestamp) > max_skew:
        raise SubscriptionError("delivery timestamp outside the allowed window")
    expected = SIGNATURE_PREFIX + hmac.new(
        key, _signing_input(body, timestamp, nonce), hashlib.sha256
    ).hexdigest()
    if not isinstance(signature, str) or not hmac.compare_digest(expected, signature):
        raise SubscriptionError("signature mismatch")
    if guard is not None:
        guard.consume(secret, nonce, now=float(current))


def make_token(secret: str, purpose: str, email: str, days: int, now: int | None = None) -> str:
    """Signed, expiring capability for a confirm/unsubscribe link."""
    key = check_secret(secret)
    if purpose not in ("confirm", "unsubscribe"):
        raise SubscriptionError("unknown token purpose")
    issued = int(time.time()) if now is None else now
    payload = {"p": purpose, "e": email, "iat": issued, "exp": issued + days * 86400}
    body = json.dumps(payload, sort_keys=True, separators=(",", ":")).encode("utf-8")
    mac = hmac.new(key, b"GBTOKEN1\n" + body, hashlib.sha256).digest()
    return _b64url(body) + "." + _b64url(mac)


def read_token(secret: str, token: str, purpose: str, now: int | None = None) -> str:
    """Return the email in a valid token, or raise. Never trusts token text."""
    key = check_secret(secret)
    if not isinstance(token, str) or token.count(".") != 1:
        raise SubscriptionError("malformed token")
    body_part, mac_part = token.split(".", 1)
    body = _unb64url(body_part)
    expected = hmac.new(key, b"GBTOKEN1\n" + body, hashlib.sha256).digest()
    if not hmac.compare_digest(expected, _unb64url(mac_part)):
        raise SubscriptionError("token signature mismatch")
    try:
        payload = json.loads(body.decode("utf-8"))
    except (UnicodeDecodeError, ValueError) as error:
        raise SubscriptionError("invalid token payload") from error
    if not isinstance(payload, dict) or payload.get("p") != purpose:
        raise SubscriptionError("token purpose mismatch")
    email = payload.get("e")
    issued, expires = payload.get("iat"), payload.get("exp")
    if not isinstance(email, str) or not isinstance(issued, int) or not isinstance(expires, int):
        raise SubscriptionError("invalid token claims")
    current = int(time.time()) if now is None else now
    if current > expires or current < issued - MAX_SKEW_SECONDS:
        raise SubscriptionError("token expired")
    return email


def self_test() -> int:
    secret = "s" * MIN_SECRET_BYTES
    try:
        body = canonical_body({"event": "advisory.published", "id": "GBSA-2026-0001"})
        ts, nonce = 1_800_000_000, new_nonce()
        signature = sign(secret, body, ts, nonce)
        verify(secret, body, ts, nonce, signature, now=ts)
        guard = ReplayGuard()
        verify(secret, body, ts, nonce, signature, guard=guard, now=ts)
        for label, call in (
            ("tampered body", lambda: verify(secret, body + b" ", ts, nonce, signature, now=ts)),
            ("bad signature", lambda: verify(secret, body, ts, nonce, SIGNATURE_PREFIX + "0" * 64, now=ts)),
            ("expired timestamp", lambda: verify(secret, body, ts, nonce, signature, now=ts + 10_000)),
            ("replayed nonce", lambda: verify(secret, body, ts, nonce, signature, guard=guard, now=ts)),
            ("empty secret", lambda: sign("", body, ts, nonce)),
        ):
            try:
                call()
            except SubscriptionError:
                continue
            raise SubscriptionError(f"{label} was accepted")
        token = make_token(secret, "confirm", "ops@example.org", 1, now=ts)
        if read_token(secret, token, "confirm", now=ts) != "ops@example.org":
            raise SubscriptionError("token round trip failed")
        try:
            read_token(secret, token, "unsubscribe", now=ts)
        except SubscriptionError:
            pass
        else:
            raise SubscriptionError("token purpose was not enforced")
    except SubscriptionError as error:
        print(f"sub_crypto self-test FAILED: {error}", file=sys.stderr)
        return 1
    print("sub_crypto self-test: ok (HMAC, skew, replay, tokens)")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(self_test())
