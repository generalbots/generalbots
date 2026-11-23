# Advisory subscriptions

- RSS/Atom: `security/feeds/rss.xml` and `security/feeds/feed.atom`; clients poll at most hourly.
- Email digest: opt-in address stored by the mailing provider, never in this repository; one digest per new advisory, no tracking pixels.
- Webhooks: subscriber-supplied HTTPS endpoint, HMAC-SHA256 signature in `X-GeneralBots-Signature`, `X-GeneralBots-Delivery` idempotency key, three exponential retries, then dead-letter. Payload: `{id, published, severity, url}` — no personal data.
- Unsubscribe is a one-click link in every email; webhook subscribers can delete their endpoint at any time.
- `scripts/security/gen_feeds.py` regenerates both feeds from `security/advisories/` front matter; no hand-edited entries.
