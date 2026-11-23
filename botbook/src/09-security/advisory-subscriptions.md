# Advisory subscriptions

General Bots publishes security advisories through pull feeds, an opt-in email
digest and signed webhooks. All three are generated from the **canonical
advisory bundle** — the same records served by `GET /api/advisories` and used
by `gb audit`, CSAF and OSV. There is no separate list to keep in sync.

## Follow advisories with a feed reader

The bulletin page links every generated feed in its `<head>`, so a reader can
discover them automatically. Two feeds cover all advisories:

- `advisories.atom` — Atom 1.0
- `advisories.rss` — RSS 2.0

Per-product feeds are also available as `product-<slug>.atom` and
`product-<slug>.rss` (for example `product-botserver.atom` and
`product-botui.rss`). The generated fragment
`security/feeds/head-links.html` holds the exact tags the bulletin site
includes; `python3 scripts/security/gen_feeds.py --check-head <page>` fails if
a published page is missing any of them.

## Email digest

The digest is opt-in with **double confirmation**: subscribing creates a
pending request, and the address is added only after the subscriber follows
the signed confirmation link. Messages are plain text with no tracking pixels,
no remote images and no click or open measurement. Every message includes a
signed **one-click unsubscribe** link and standard `List-Unsubscribe-Post`
headers; unsubscribing erases the subscriber's record immediately.

Only the address, locale, consent status and consent timestamp are stored — no
name, IP address, device or behavioural data. This is the minimum needed to
deliver the digest and to honour LGPD access and erasure requests.

## Signed webhooks for CMDB and ticketing

Downstream systems can ingest new advisories automatically. Each delivery is
an HTTPS `POST` with a JSON body containing the advisory records plus a
timestamp and a random nonce. An HMAC-SHA256 signature in the
`X-GB-Signature` header covers the exact body, timestamp and nonce. Receivers
must reject requests with an old timestamp, a bad signature or a nonce that
has already been used — a captured delivery cannot be replayed. The signing
key is distributed out of band through the deployment secret store and is
never included in the request.

## Operational reference

The byte-level channel contract, signature formula, header names and
verification commands are documented in
`security/policy/SUBSCRIPTIONS.md`. All of the offline checks run with
`python3 scripts/security/subscriptions_selftest.py`.
