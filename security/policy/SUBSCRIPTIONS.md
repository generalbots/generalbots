# Advisory subscription channels (operational contract)

Security advisories are **pushed** to administrators. Checking a forum or a web
page by hand is not an incident-notification path. This document describes the
channels shipped today for issue #2719; progress and open work live in the
issue, not here.

## One source of truth

Every channel reads
`botlib/src/security/advisory/data/advisories.json` — the same canonical bundle
that backs the advisory API (`GET /api/advisories`), `gb audit`, CSAF and OSV.
There are no hand-maintained advisory lists in feeds, digests or webhook
payloads. Editing a channel artifact by hand is wrong: regenerate it.

`scripts/security/advisory_source.py` is the only loader. It validates ids,
titles, severity, product and publication dates before a channel can emit
anything, so a malformed bundle fails closed.

## Channels

| Channel | Artifact / surface | Trigger | Source |
|---------|--------------------|---------|--------|
| Atom (all) | `security/feeds/advisories.atom` | pull | canonical bundle |
| RSS (all) | `security/feeds/advisories.rss` | pull | canonical bundle |
| Per-product Atom/RSS | `security/feeds/product-<slug>.{atom,rss}` | pull | canonical bundle filtered by `product` |
| Email digest | opt-in, plain text | new advisories | canonical bundle |
| Signed webhook | `POST` JSON to a subscriber's HTTPS endpoint | new advisories | canonical bundle |

Regenerate the pull channels and regenerate the generated head fragment with:

```bash
python3 scripts/security/gen_feeds.py --out security/feeds
python3 scripts/security/gen_feeds.py --check-head website/security/index.html
```

`security/feeds/head-links.html` contains the exact `<link rel="alternate">`
tags for the bulletin page `<head>`. The bulletin site is a separate static
tree; its build must include that fragment in `<head>`. `--check-head` is the
gate: it fails when a rendered page omits any generated feed. Feed order,
counts and entry ids come from the bundle, so a new advisory appears in every
channel without editing this file.

## Signed webhooks

A delivery body is compact, sorted JSON:

```json
{
  "schema": "https://generalbots.org/security/schema/webhook-1.0.json",
  "event": "advisory.published",
  "timestamp": 1800000000,
  "nonce": "<32 lowercase hex chars>",
  "advisories": [ ... canonical records ... ]
}
```

The signature is HMAC-SHA256 over this canonical string (`\n` is a byte LF):

```
GBADV1
<timestamp>
<nonce>
<sha256-hex-of-exact-body-bytes>
```

It is sent in `X-GB-Signature: v1=<hex>` together with `X-GB-Timestamp` and
`X-GB-Nonce`. The receiver MUST, in order: require all three headers and a
recognised schema/event; reject a timestamp more than 300 s from its clock;
recompute the HMAC and compare in constant time; then consume the nonce in a
replay cache scoped to the signing key. A nonce may be accepted once. The
cache is bounded and TTL-limited; the signing key is never transmitted.
Targets must be HTTPS and redirects to plaintext are refused.

`scripts/security/sub_webhook.py` provides `build_delivery`, `verify_delivery`
and `post_webhook` and an offline `--self-test`. `dispatch(bundle, url, secret)`
is the one-shot sender for a newly published advisory. Secrets come from the
deployment's secret store (Vault), never from source or this document.

## Email digest and LGPD

`scripts/security/sub_subscribers.py` stores exactly four fields per
subscriber: `email`, `locale`, `status`, `consent_at`. Addresses are
normalised and syntax-checked; the file is written atomically with `0600`
permissions. No name, IP, user agent, device, open or click data is collected.

- **Double opt-in:** `subscribe()` records a `pending` consent and returns a
  signed token; the address is not mailed to until `confirm(token)` activates
  it. Confirming twice is a no-op.
- **One-click unsubscribe:** `list_headers()` emits `List-Unsubscribe` with a
  signed HTTPS link and `List-Unsubscribe-Post: List-Unsubscribe=One-Click`
  (RFC 8058). `unsubscribe(token)` erases the record outright (LGPD art. 18
  VI); no login is required and no reason is collected.
- **No tracking:** `sub_digest.py` renders plain text only — no images, no
  remote CSS, no open/click beacons, no third-party assets. The digest is
  rebuilt from the canonical bundle on every send.
- **Retention:** pending consents that are never confirmed should be purged by
  the operator's retention job; erasure is immediate on unsubscribe.

Confirmation and unsubscribe links are signed capabilities (HMAC over purpose,
address and expiry). Rotating the signing key invalidates all outstanding
links; send a fresh confirmation before the old key is retired.

## Verification

All checks are offline and need no server, browser or paid model call:

```bash
python3 -m py_compile scripts/security/*.py
python3 scripts/security/subscriptions_selftest.py
```

The self-tests prove feed validity and per-product isolation, deterministic
output, `<head>` link coverage, HMAC/skew/replay rejection, double opt-in and
erasure, and a pixel-free digest in all eight locales.
