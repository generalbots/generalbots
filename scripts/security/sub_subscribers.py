#!/usr/bin/env python3
"""LGPD-minimal, double-opt-in subscriber list for the advisory digest (#2719).

Stored fields are exactly: email, locale, status, consent_at. No name, IP,
user agent, click, open or device data is collected; the confirmation and
one-click-unsubscribe capabilities are signed tokens, not stored secrets.
Erasure (`unsubscribe`) removes the record outright (LGPD art. 18 VI).

Usage: `python3 scripts/security/sub_subscribers.py --self-test`
"""

from __future__ import annotations

import json
import os
import re
import tempfile
from datetime import datetime, timezone
from pathlib import Path

from sub_crypto import (
    SubscriptionError,
    TOKEN_TTL_DAYS,
    UNSUBSCRIBE_TTL_DAYS,
    make_token,
    read_token,
)

EMAIL_RE = re.compile(
    r"^[A-Za-z0-9.!#$%&'*+/=?^_`{|}~-]+@[A-Za-z0-9]"
    r"(?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?"
    r"(?:\.[A-Za-z0-9](?:[A-Za-z0-9-]{0,61}[A-Za-z0-9])?)+$"
)
LOCALES = frozenset({
    "en", "pt-BR", "es", "fr", "de", "it", "ru", "ja", "ko", "zh-CN",
})
ALLOWED_FIELDS = frozenset({"email", "locale", "status", "consent_at"})
STATUSES = frozenset({"pending", "active"})
MAX_EMAIL = 254
FILE_MODE = 0o600


def normalize_email(value: str) -> str:
    """Lower-case, length- and syntax-check an address; reject header injection."""
    if not isinstance(value, str) or len(value) > MAX_EMAIL or "\n" in value or "\r" in value:
        raise SubscriptionError("invalid email address")
    email = value.strip().lower()
    if not EMAIL_RE.match(email):
        raise SubscriptionError("invalid email address")
    return email


def check_locale(value: str) -> str:
    if value not in LOCALES:
        raise SubscriptionError("unsupported digest locale")
    return value


def _timestamp(now: datetime | None) -> str:
    instant = now or datetime.now(timezone.utc)
    if instant.tzinfo is None:
        raise SubscriptionError("timestamp must be timezone aware")
    return instant.astimezone(timezone.utc).isoformat()


class SubscriberStore:
    """A tiny atomic JSON store holding only fields in ALLOWED_FIELDS."""

    def __init__(self, path: Path) -> None:
        self.path = Path(path)
        self._records: dict[str, dict] = {}
        self._load()

    def _load(self) -> None:
        if not self.path.exists():
            return
        try:
            data = json.loads(self.path.read_text(encoding="utf-8"))
        except (OSError, ValueError) as error:
            raise SubscriptionError(f"cannot read subscriber store: {error}") from error
        if not isinstance(data, dict):
            raise SubscriptionError("subscriber store is not an object")
        for key, record in data.items():
            email = normalize_email(key)
            if not isinstance(record, dict) or set(record) - ALLOWED_FIELDS:
                raise SubscriptionError("subscriber store holds unexpected data")
            if record.get("email") != email or record.get("status") not in STATUSES:
                raise SubscriptionError("subscriber store is inconsistent")
            check_locale(record["locale"])
            self._records[email] = record

    def _save(self) -> None:
        self.path.parent.mkdir(parents=True, exist_ok=True)
        fd, temp_name = tempfile.mkstemp(prefix=".subscribers-", dir=str(self.path.parent))
        try:
            with os.fdopen(fd, "w", encoding="utf-8") as handle:
                json.dump(self._records, handle, sort_keys=True, indent=2)
                handle.write("\n")
            os.chmod(temp_name, FILE_MODE)
            os.replace(temp_name, self.path)
        except OSError as error:
            try:
                os.unlink(temp_name)
            except OSError:
                pass
            raise SubscriptionError(f"cannot write subscriber store: {error}") from error

    def subscribe(self, email: str, secret: str, locale: str = "en") -> str:
        """Create/refresh a pending consent and return the confirmation token."""
        address = normalize_email(email)
        language = check_locale(locale)
        token = make_token(secret, "confirm", address, TOKEN_TTL_DAYS)
        record = self._records.get(address)
        if record is None:
            self._records[address] = {
                "email": address,
                "locale": language,
                "status": "pending",
                "consent_at": "",
            }
            self._save()
        elif record["status"] == "pending" and record["locale"] != language:
            record["locale"] = language
            self._save()
        return token

    def confirm(self, token: str, secret: str, now: datetime | None = None) -> bool:
        """Activate a pending record after the subscriber followed the link."""
        email = read_token(secret, token, "confirm")
        record = self._records.get(email)
        if record is None or record["status"] == "active":
            return False
        record["status"] = "active"
        record["consent_at"] = _timestamp(now)
        self._save()
        return True

    def unsubscribe(self, token: str, secret: str) -> bool:
        """Erase a record on a one-click unsubscribe. Returns True if removed."""
        email = read_token(secret, token, "unsubscribe")
        if email not in self._records:
            return False
        del self._records[email]
        self._save()
        return True

    def active_emails(self) -> list[str]:
        return sorted(email for email, record in self._records.items() if record["status"] == "active")

    def active_records(self) -> list[dict]:
        return [record for _, record in sorted(self._records.items()) if record["status"] == "active"]

    def pending_emails(self) -> list[str]:
        return sorted(email for email, record in self._records.items() if record["status"] == "pending")


def unsubscribe_url(base: str, secret: str, email: str) -> str:
    """Signed one-click link. No login, no tracking identifier, no pixel."""
    address = normalize_email(email)
    token = make_token(secret, "unsubscribe", address, UNSUBSCRIBE_TTL_DAYS)
    return f"{base.rstrip('/')}/unsubscribe?t={token}"


def list_headers(base: str, secret: str, email: str) -> dict[str, str]:
    """RFC 8058 one-click headers; a GET link alone is never enough."""
    address = normalize_email(email)
    return {
        "List-Unsubscribe": f"<mailto:security-unsubscribe@generalbots.org?subject={address}>, <{unsubscribe_url(base, secret, address)}>",
        "List-Unsubscribe-Post": "List-Unsubscribe=One-Click",
    }


def self_test() -> int:
    import sys
    import tempfile

    secret = "k" * 32
    try:
        with tempfile.TemporaryDirectory(prefix="gb-subs-") as temp:
            store = SubscriberStore(Path(temp) / "subscribers.json")
            token = store.subscribe("Ops@Example.org ", secret, locale="pt-BR")
            if store.active_emails() or store.pending_emails() != ["ops@example.org"]:
                raise SubscriptionError("double opt-in did not start as pending")
            if not store.confirm(token, secret):
                raise SubscriptionError("confirmation did not activate the subscriber")
            if store.confirm(token, secret):
                raise SubscriptionError("confirmation was not idempotent")
            if store.active_emails() != ["ops@example.org"]:
                raise SubscriptionError("active list is wrong")
            record = store.active_records()[0]
            if set(record) != ALLOWED_FIELDS or not record["consent_at"]:
                raise SubscriptionError("store holds more than the minimum data")
            headers = list_headers("https://generalbots.org/security", secret, "ops@example.org")
            if headers["List-Unsubscribe-Post"] != "List-Unsubscribe=One-Click":
                raise SubscriptionError("one-click header missing")
            link = unsubscribe_url("https://generalbots.org/security", secret, "ops@example.org")
            if "t=" not in link:
                raise SubscriptionError("unsubscribe link is unsigned")
            from sub_crypto import read_token

            unsub_token = link.split("t=", 1)[1]
            if read_token(secret, unsub_token, "unsubscribe") != "ops@example.org":
                raise SubscriptionError("unsubscribe token is invalid")
            if not store.unsubscribe(unsub_token, secret):
                raise SubscriptionError("erasure did not remove the record")
            if store.active_emails() or store.pending_emails():
                raise SubscriptionError("erased subscriber still present")
            for bad in ("a@b\nBcc: victim@example.org", "", "no-at-sign"):
                try:
                    store.subscribe(bad, secret)
                except SubscriptionError:
                    continue
                raise SubscriptionError("invalid email was accepted")
    except (SubscriptionError, OSError, ValueError) as error:
        print(f"sub_subscribers self-test FAILED: {error}", file=sys.stderr)
        return 1
    print("sub_subscribers self-test: ok (double opt-in, erasure, LGPD minimum)")
    return 0


if __name__ == "__main__":
    import sys

    raise SystemExit(self_test())
