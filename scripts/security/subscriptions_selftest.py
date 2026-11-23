#!/usr/bin/env python3
"""Run every offline self-test for the advisory subscription channels (#2719)."""

from __future__ import annotations

import sys

import csaf_sync
import gen_feeds
import sub_crypto
import sub_digest
import sub_subscribers
import sub_webhook

CHECKS = (
    ("CSAF feed", csaf_sync.self_test),
    ("canonical feeds", gen_feeds.self_test),
    ("signing and replay", sub_crypto.self_test),
    ("subscriber privacy", sub_subscribers.self_test),
    ("signed webhooks", sub_webhook.self_test),
    ("email digest", sub_digest.self_test),
)


def main() -> int:
    failed = 0
    for label, check in CHECKS:
        print(f"[{label}]")
        if check() != 0:
            failed += 1
    print(f"subscriptions: {len(CHECKS) - failed}/{len(CHECKS)} checks passed")
    return 1 if failed else 0


if __name__ == "__main__":
    raise SystemExit(main())
