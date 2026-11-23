#!/usr/bin/env python3
"""Signed outbound advisory webhooks for CMDB/ticketing ingest (#2719).

A delivery is a canonical JSON body carrying `timestamp` and `nonce` plus an
HMAC-SHA256 signature over (version, timestamp, nonce, SHA-256 of body). The
same bytes are verified before ingest, with clock-skew and replay checks, so a
captured delivery cannot be replayed. The signing key is never sent on the wire.

Usage: `python3 scripts/security/sub_webhook.py --self-test`
"""

from __future__ import annotations

import json
import sys
import urllib.error
import urllib.parse
import urllib.request

from advisory_source import load_bundle
from sub_crypto import (
    MAX_BODY_BYTES,
    ReplayGuard,
    SubscriptionError,
    canonical_body,
    new_nonce,
    sign,
    verify,
)

EVENT = "advisory.published"
SCHEMA = "https://generalbots.org/security/schema/webhook-1.0.json"
HEADER_SIGNATURE = "X-GB-Signature"
HEADER_TIMESTAMP = "X-GB-Timestamp"
HEADER_NONCE = "X-GB-Nonce"
TIMEOUT_SECONDS = 10


def build_delivery(
    bundle: dict,
    secret: str,
    *,
    timestamp: int | None = None,
    nonce: str | None = None,
) -> tuple[bytes, dict[str, str]]:
    """Return (body, headers) for one delivery of the canonical advisories."""
    import time

    issued = int(time.time()) if timestamp is None else timestamp
    value = new_nonce() if nonce is None else nonce
    if issued <= 0:
        raise SubscriptionError("invalid delivery timestamp")
    payload = {
        "schema": SCHEMA,
        "event": EVENT,
        "timestamp": issued,
        "nonce": value,
        "advisories": bundle["advisories"],
    }
    body = canonical_body(payload)
    headers = {
        "Content-Type": "application/json; charset=utf-8",
        "User-Agent": "generalbots-advisory-webhook/1.0",
        HEADER_TIMESTAMP: str(issued),
        HEADER_NONCE: value,
        HEADER_SIGNATURE: sign(secret, body, issued, value),
    }
    return body, headers


def verify_delivery(
    body: bytes,
    headers: dict[str, str],
    secret: str,
    guard: ReplayGuard | None = None,
    *,
    now: int | None = None,
) -> dict:
    """Authenticate a delivery and return its payload, or fail closed."""
    if len(body) > MAX_BODY_BYTES:
        raise SubscriptionError("webhook body exceeds the size limit")
    for name in (HEADER_SIGNATURE, HEADER_TIMESTAMP, HEADER_NONCE):
        if not headers.get(name):
            raise SubscriptionError(f"missing {name}")
    try:
        payload = json.loads(body.decode("utf-8"))
    except (UnicodeDecodeError, ValueError) as error:
        raise SubscriptionError("invalid webhook JSON") from error
    if not isinstance(payload, dict) or payload.get("schema") != SCHEMA:
        raise SubscriptionError("unsupported webhook schema")
    if payload.get("event") != EVENT or not isinstance(payload.get("advisories"), list):
        raise SubscriptionError("unsupported webhook event")
    timestamp, nonce = payload.get("timestamp"), payload.get("nonce")
    try:
        header_timestamp = int(headers[HEADER_TIMESTAMP])
    except ValueError as error:
        raise SubscriptionError("invalid timestamp header") from error
    if timestamp != header_timestamp or nonce != headers[HEADER_NONCE]:
        raise SubscriptionError("headers do not match the signed body")
    if not isinstance(timestamp, int) or not isinstance(nonce, str):
        raise SubscriptionError("invalid timestamp or nonce in body")
    verify(secret, body, timestamp, nonce, headers[HEADER_SIGNATURE], guard=guard, now=now)
    return payload


class NoDowngrade(urllib.request.HTTPRedirectHandler):
    """Refuse a redirect that would downgrade HTTPS to plaintext."""

    def redirect_request(self, req, fp, code, msg, headers, newurl):
        if not newurl.lower().startswith("https://"):
            raise SubscriptionError("refusing webhook redirect to a non-HTTPS URL")
        return super().redirect_request(req, fp, code, msg, headers, newurl)


def _http_post(url: str, headers: dict[str, str], body: bytes, timeout: int) -> int:
    parsed = urllib.parse.urlparse(url)
    if parsed.scheme != "https" or not parsed.netloc:
        raise SubscriptionError("webhook target must be an HTTPS URL")
    opener = urllib.request.build_opener(NoDowngrade())
    request = urllib.request.Request(url, data=body, headers=headers, method="POST")
    with opener.open(request, timeout=timeout) as response:
        return int(response.status)


def post_webhook(
    url: str,
    headers: dict[str, str],
    body: bytes,
    *,
    timeout: int = TIMEOUT_SECONDS,
    sender=None,
) -> int:
    """POST one signed delivery. `sender` is injectable for offline tests."""
    if timeout <= 0:
        raise SubscriptionError("invalid webhook timeout")
    deliver = sender or _http_post
    try:
        return deliver(url, headers, body, timeout)
    except (OSError, urllib.error.URLError, ValueError) as error:
        raise SubscriptionError(f"webhook delivery failed: {error}") from error


def dispatch(bundle: dict, url: str, secret: str, *, sender=None) -> int:
    """Build and post a signed delivery of the canonical advisories."""
    body, headers = build_delivery(bundle, secret)
    return post_webhook(url, headers, body, sender=sender)


def self_test() -> int:
    secret = "w" * 32
    ts = 1_800_000_000
    try:
        bundle = load_bundle()
        body, headers = build_delivery(bundle, secret, timestamp=ts, nonce="ab" * 16)
        payload = verify_delivery(body, headers, secret, now=ts)
        if len(payload["advisories"]) != len(bundle["advisories"]):
            raise SubscriptionError("delivery lost canonical advisories")
        guard = ReplayGuard()
        verify_delivery(body, headers, secret, guard=guard, now=ts)

        sent: dict = {}

        def fake_send(url, outgoing_headers, outgoing_body, timeout):
            sent.update(url=url, headers=outgoing_headers, body=outgoing_body, timeout=timeout)
            return 202

        status = post_webhook("https://cmdb.example.org/advisories", headers, body, sender=fake_send)
        if status != 202 or sent["body"] != body or sent["headers"][HEADER_SIGNATURE] != headers[HEADER_SIGNATURE]:
            raise SubscriptionError("delivery was not sent byte-for-byte")
        for label, mutate in (
            ("tampered body", lambda: verify_delivery(body + b" ", headers, secret, now=ts)),
            ("changed nonce header", lambda: verify_delivery(
                body, {**headers, HEADER_NONCE: "cd" * 16}, secret, now=ts)),
            ("missing signature", lambda: verify_delivery(
                body, {k: v for k, v in headers.items() if k != HEADER_SIGNATURE}, secret, now=ts)),
            ("clock skew", lambda: verify_delivery(body, headers, secret, now=ts + 10_000)),
            ("replayed nonce", lambda: verify_delivery(body, headers, secret, guard=guard, now=ts)),
            ("plaintext target", lambda: post_webhook(
                "http://cmdb.example.org/advisories", headers, body, sender=_http_post)),
        ):
            try:
                mutate()
            except SubscriptionError:
                continue
            raise SubscriptionError(f"{label} was accepted")
    except (SubscriptionError, ValueError, OSError) as error:
        print(f"sub_webhook self-test FAILED: {error}", file=sys.stderr)
        return 1
    print("sub_webhook self-test: ok (HMAC, skew, replay, HTTPS-only dispatch)")
    return 0


if __name__ == "__main__":
    raise SystemExit(self_test())
