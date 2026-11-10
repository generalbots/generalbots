# Inbound Webhook Signature Matrix 🟡 BETA

Each channel exposes its own inbound route and its own authenticity check. This
page states what the server actually verifies today, so an operator can tell
which endpoints are safe to expose and which need a proxy rule in front.

Authority: the channel crates under `botserver/crates/bot*` and the routes
mounted in `botserver/src/main_module/routes/feature_routers/make_saas_router.rs`.

## Matrix

| Channel | Inbound route | Verification | Behaviour when unconfigured |
|---|---|---|---|
| WhatsApp Cloud API | `GET/POST /api/whatsapp/webhook` | `X-Hub-Signature-256` HMAC-SHA256 over the **raw** body | Accepts unsigned deliveries |
| Telegram | `POST /webhook/telegram`, `POST /webhook/telegram/:bot_name` | `X-Telegram-Bot-Api-Secret-Token`, constant-time compare | Accepts unsigned deliveries |
| Microsoft Teams | `POST /api/msteams/messages` | **None** — no JWT, no shared secret | Always accepts |
| Instagram / Messenger | `GET/POST /api/instagram/webhook` | GET verify token equality only; POST has **no** HMAC | GET rejects on mismatch; POST always accepts |
| Stripe (billing) | `POST /api/billing/webhook` | `stripe-signature` HMAC-SHA256 with 300 s tolerance | Rejects when no webhook secret is configured |
| Outbound settings webhooks | delivered by the server, not received | `X-Webhook-Signature` HMAC-SHA256 over the payload | — |

## WhatsApp

`handle_webhook` reads the body as `Bytes` and only parses JSON **after** the
gate, because the digest binds the exact bytes Meta sent; deserialising into a
typed body first would re-serialise it and no digest would ever match.

```text
app_secret = vault("whatsapp_app_secret")
if app_secret is empty        → accept
else                          → require valid X-Hub-Signature-256
```

The comparison is constant-time (`constant_time_eq` in
`botwhatsapp/src/utils.rs`), the `sha256=` prefix is mandatory, and an uppercase
digest is rejected rather than normalised. The GET handshake compares
`hub.verify_token` against `whatsapp_verify_token` and echoes the challenge.

The residual risk is the documented default: a bot with no
`whatsapp_app_secret` accepts any POST that reaches the route, so the endpoint
is only as private as the network path to it.

## Telegram

`verify_webhook_secret_for_bot` reads the per-bot `telegram-webhook-secret`
configuration and compares it with the header using a constant-time comparison
(plain `==` on strings leaks the first differing byte through timing).

| Configured secret | Delivered header | Result |
|---|---|---|
| empty | anything | accepted |
| set | exact match | accepted |
| set | missing or different | `401 Unauthorized` |
| unreadable (DB error) | — | `500` — fails closed |

The comment in the source states the intent: a configuration read failure must
not turn into an open door while the database is down.

## Microsoft Teams

`handle_incoming` accepts a `TeamsActivity` JSON body and dispatches on
`activity.type`. There is no `Authorization` header check, no Bot Framework JWT
validation and no shared secret. Anyone who can reach `/api/msteams/messages`
can post an activity and have it routed into a session.

The outbound side does authenticate: `botmsteams/src/adapter.rs` sends
`Authorization: Bearer <token>` on its calls to the Bot Framework.

## Instagram / Messenger

- **GET** `/api/instagram/webhook` implements the Meta subscription handshake:
  `hub.mode` must be `subscribe` and `hub.verify_token` must equal the bot's
  configured `instagram-verify-token` (default `webhook_verify`).
- **POST** `/api/instagram/webhook` deserialises the payload and handles
  `entry[].messaging[].message.text` only. It does **not** verify
  `X-Hub-Signature-256`, so a forged POST is accepted.

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
| WhatsApp in production | Set `whatsapp_app_secret`; otherwise the route accepts anything |
| Telegram in production | Set `telegram-webhook-secret`; the default is unsigned |
| Teams | Terminate at the proxy and restrict the source, or add a verification layer; the server does not check |
| Instagram | Keep the route behind the proxy; POST authenticity is not verified |
| Any channel | The channel routes are public by design — they cannot require a user session |

## Related pages

- [WhatsApp Webhooks](./whatsapp-webhooks.md) — envelope, retries and payloads.
- [WhatsApp Channel Configuration](../10-configuration-deployment/whatsapp-channel.md)
  — Vault keys and app secret setup.
- [Telegram Channel](./telegram-channel.md) — bot registration.
- [Teams Channel](./teams-channel.md) — Bot Framework registration.
- [Security Features](../09-security/security-features.md) — the wider model.
