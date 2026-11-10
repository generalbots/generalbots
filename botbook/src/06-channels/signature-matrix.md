# Inbound Webhook Signature Matrix 🟡 BETA

Each channel exposes its own inbound route and its own authenticity check. This
page states what the server actually verifies today, so an operator can tell
which endpoints are safe to expose and which need a proxy rule in front.

Authority: the channel crates under `botserver/crates/bot*` and the routes
mounted in `botserver/src/main_module/routes/feature_routers/make_saas_router.rs`.

## Matrix

| Channel | Inbound route | Verification | Behaviour when unconfigured |
|---|---|---|---|
| WhatsApp Cloud API | `GET/POST /api/whatsapp/webhook` | `X-Hub-Signature-256` HMAC-SHA256 over the **raw** body | Rejects every POST (`403`) |
| Telegram | `POST /webhook/telegram`, `POST /webhook/telegram/:bot_name` | `X-Telegram-Bot-Api-Secret-Token`, constant-time compare | Rejects every POST (`503`) |
| Microsoft Teams | `POST /api/msteams/messages` | `Authorization: Bearer` Bot Framework JWT, or `HMAC` outgoing-webhook signature | Rejects every POST (`401`) |
| Instagram / Messenger | `GET/POST /api/instagram/webhook` | `X-Hub-Signature-256` HMAC-SHA256 over the **raw** body (POST) + `hub.verify_token` (GET) | Rejects every POST and handshake (`503`) |
| Stripe (billing) | `POST /api/billing/webhook` | `stripe-signature` HMAC-SHA256 with 300 s tolerance | Rejects when no webhook secret is configured |
| Outbound settings webhooks | delivered by the server, not received | `X-Webhook-Signature` HMAC-SHA256 over the payload | — |

Channels without an inbound HTTP handler: Slack and Discord are outbound only,
Facebook declares an anonymous path but registers no handler, e-mail inbound is
an IMAP poller, and the Twilio SMS module is a payload parser with no route.
`scripts/check_webhook_auth.py --list` prints this enumeration from the code.

## The fails-closed rule

An anonymous inbound route that answers is worse than one that does not, so the
rule is: **an unconfigured credential rejects the delivery**. Every channel
follows it since #2142.

| Channel | Credential | Unconfigured | Status |
|---|---|---|---|
| WhatsApp | `whatsapp_app_secret` (Vault) / `whatsapp-app-secret` (config.csv) | every POST refused | `403` |
| Microsoft Teams | `teams-app-id` (Bot Framework JWT) and/or `teams-security-token` (HMAC) | every POST refused when neither is set | `401` |
| Stripe | webhook secret | every POST refused | `400` |
| Telegram | `telegram-webhook-secret` | every POST refused, one config error per bot | `503` |
| Instagram / Messenger | `instagram-app-secret` (POST) and `instagram-verify-token` (GET) | every POST and handshake refused | `503` |

The gate is executable: `scripts/check_webhook_auth.py` discovers every inbound
route, requires its handler (or the function it delegates to) to call a verifier,
and rejects an empty secret that returns accept. Its allowlist
(`scripts/webhook_auth_allowlist.txt`) is shrink-only.

Two details make the WhatsApp gate hard to bypass accidentally:

- the digest covers the **raw** bytes, so the body is read as `Bytes` and parsed
  only after the gate (re-serialising JSON changes key order and whitespace);
- the `sha256=` prefix is mandatory and an uppercase digest is rejected rather
  than normalised, because Meta always sends lowercase.

`DeliveryAuth` (`botwhatsapp/src/webhooks.rs`) keeps `Unconfigured` and
`Mismatch` as separate outcomes, so the log names the operator action
(`set whatsapp_app_secret`) instead of reporting a generic signature failure.

All four hardened paths are pinned by regression tests, which the readiness gates
`security-webhooks` and `webhook-auth` look for before they report `PASS`
(`botwhatsapp/src/utils_tests.rs`, `botmsteams/src/auth/tests.rs`,
`bottelegram/src/webhook_gate_tests.rs`, `botinstagram/src/auth_tests.rs` — all
four crates run in the `workspace-tests` CI matrix; see
[Production Readiness Gate](../10-configuration-deployment/production-readiness.md)).

## WhatsApp

`handle_webhook` reads the body as `Bytes` and only parses JSON **after** the
gate, because the digest binds the exact bytes Meta sent; deserialising into a
typed body first would re-serialise it and no digest would ever match.

```text
app_secret = vault("whatsapp_app_secret") ?? config("whatsapp-app-secret")
if app_secret is empty        → 403, the delivery is refused
else                          → require valid X-Hub-Signature-256, else 403
```

The gate fails closed (#2113). An installation that has not configured the app
secret receives nothing: accepting unsigned POSTs let any caller drive a bot
session, which is worse than a channel that has to be configured before it
works.

The comparison is constant-time (`constant_time_eq` in
`botwhatsapp/src/utils.rs`), the `sha256=` prefix is mandatory, and an uppercase
digest is rejected rather than normalised. The GET handshake is unchanged: it
compares `hub.verify_token` against `whatsapp_verify_token` and echoes the
challenge.

## Telegram

`verify_webhook_secret_for_bot` reads the per-bot `telegram-webhook-secret`
configuration and compares it with the `X-Telegram-Bot-Api-Secret-Token` header
in constant time (`botsecurity_core::constant_time`, because `==` on strings
leaks the first differing byte through timing).

| Configured secret | Delivered header | Result |
|---|---|---|
| empty (unset) | anything | `503 Service Unavailable` + one configuration error per bot |
| set | exact match | accepted |
| set | missing or different | `401 Unauthorized` |
| unreadable (DB error) | — | `500` — fails closed |

The gate runs before the JSON body is parsed, so an unsigned POST never reaches
the update decoder. An unset secret is a deployment mistake, not a per-request
event: the configuration error is logged once per bot
(`webhook_gate::warn_unconfigured_once`) so a stranger cannot flood the log.

## Microsoft Teams

`handle_incoming` reads the body as `Bytes`, calls `auth::authorize` **before**
parsing the activity, and answers `401` to anything it cannot attribute. Two
credentials are accepted, one per way Teams reaches a bot:

| Header | Used by | Check |
|---|---|---|
| `Authorization: Bearer <jwt>` | Bot Framework channel | RS256 signature against the Bot Framework OpenID key set (`login.botframework.com`), plus `iss` = `https://api.botframework.com`, `aud` = `teams-app-id`, `exp` and `nbf` with 300 s skew |
| `Authorization: HMAC <sig>` | outgoing webhook | `base64(HMAC-SHA256(base64decode(teams-security-token), raw_body))` |

Implementation notes, all in `botmsteams/src/auth/`:

- the `kid` header selects the key; an unknown `kid` triggers one refresh of the
  cached key set so a Microsoft key rollover does not reject deliveries forever,
  and the cache expires after 24 h;
- `alg` must be RS256 — an HS256 token keyed with the (public) RSA key is the
  classic confusion attack and is refused before any key lookup;
- the HMAC is compared with `Mac::verify_slice`, which is constant-time;
- neither secret is logged; a rejection logs only the reason (`HMAC signature
  mismatch`, `Bot Framework token rejected`, …).

When neither `teams-app-id` nor `teams-security-token` is configured the route
rejects every POST. The outbound side authenticates separately:
`botmsteams/src/adapter.rs` sends `Authorization: Bearer <token>` on its calls
to the Bot Framework.

## Instagram / Messenger

- **GET** `/api/instagram/webhook` implements the Meta subscription handshake:
  `hub.mode` must be `subscribe` and `hub.verify_token` must equal the bot's
  configured `instagram-verify-token`. There is no default (#2142): the handler
  used to fall back to the literal `webhook_verify`, a published secret.
- **POST** `/api/instagram/webhook` verifies `X-Hub-Signature-256` —
  `sha256=hex(HMAC-SHA256(instagram-app-secret, raw body))` — before parsing the
  payload. The digest covers the raw bytes, the `sha256=` prefix is mandatory,
  an uppercase digest is rejected, and the comparison is constant time.

Both gates live in `botinstagram/src/auth.rs` and fail closed: an unconfigured
app secret or verify token refuses the request (`503`) and logs the operator
action once per process.

Two additional limitations in the current handler:

1. **Postbacks are dropped.** `botinstagram/src/adapter.rs` can parse a
   `postback.payload`, but the webhook handler never looks at
   `messaging[].postback`, so a button press produces no turn.
2. **Attachments are ignored.** Only `message.text` is read; media messages
   arrive with no text and are silently skipped.

## Stripe

`handle_stripe_webhook` requires the `stripe-signature` header, splits it into
`t` (timestamp) and `v1` (signature), signs `"{t}.{payload}"` with the configured
webhook secret, and rejects a mismatch. It then rejects timestamps older than
300 seconds, which bounds replay.

One detail worth knowing before treating this as a hardened path: the digest
comparison uses `expected_sig != *received_sig`, a short-circuiting string
comparison, unlike the WhatsApp and Telegram gates which compare in constant
time. The exposure is small (the attacker also needs a valid timestamp window
and the secret is high-entropy), but it is inconsistent with the other
channels.

## Outbound settings webhooks

The server signs its own deliveries (`botsettings/src/webhook_delivery.rs`):
`X-Webhook-Signature` carries an HMAC-SHA256 over the payload, keyed by a secret
stored in Vault under `secret/gbo/settings/webhooks/{webhook_id}`. The same
module validates destination URLs: HTTPS is required when `GB_ENV=production`,
and `localhost`, `127.0.0.1`, `0.0.0.0`, `::1`, `169.254.`, `10.` and `192.168.`
are refused. That blocklist does not cover `172.16.0.0/12`, IPv4-mapped IPv6
(`::ffff:10.0.0.1`) or internal DNS names — see
[Input Guardrails](../09-security/input-guardrails.md) for the same pattern.

## Operator guidance

| Situation | Reasonable posture |
|---|---|
| WhatsApp in production | `whatsapp-app-secret` is mandatory: without it every POST is refused |
| Telegram in production | Set `telegram-webhook-secret` **and** register the same value as the Bot API `setWebhook` `secret_token`; without it every POST is refused |
| Teams | Set `teams-app-id` (Bot Framework) or `teams-security-token` (outgoing webhook); without one every POST is refused |
| Instagram | Set `instagram-app-secret` (POST HMAC) and `instagram-verify-token` (handshake); without them every delivery and handshake is refused |
| Any channel | The channel routes are public by design — they cannot require a user session |

## Related pages

- [WhatsApp Webhooks](./whatsapp-webhooks.md) — envelope, retries and payloads.
- [WhatsApp Channel Configuration](../10-configuration-deployment/whatsapp-channel.md)
  — Vault keys and app secret setup.
- [Telegram Channel](./telegram-channel.md) — bot registration.
- [Teams Channel](./teams-channel.md) — Bot Framework registration.
- [Security Features](../09-security/security-features.md) — the wider model.
