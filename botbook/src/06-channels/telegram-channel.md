# Telegram Channel 🟡 BETA

Telegram sends messages to the bot through a webhook: the platform posts each
update to `/webhook/telegram`, the bot resolves the conversation, stores any
attachment in Drive and routes the content through the normal bot pipeline.

## Configuration

Store the credentials in the bot's `config.csv` (`.gbot` folder) — never in a
script:

| Key | Required | Purpose |
|-----|----------|---------|
| `telegram-bot-token` | yes | Bot token issued by [@BotFather](https://t.me/BotFather). Used for `getFile`, downloads and all outbound calls. |
| `telegram-webhook-secret` | no | Secret token that Telegram echoes in the `X-Telegram-Bot-Api-Secret-Token` header. When set, unsigned deliveries are rejected with `401`. |

```csv
key,value
telegram-bot-token,123456:ABC-DEF...
telegram-webhook-secret,a-long-random-string
```

Missing `telegram-bot-token` is reported as
`Telegram bot token not configured` in the log, and outbound calls fail — the
token is read per bot, so a shared deployment keeps one token per bot.

## Registering the webhook

Point Telegram at the bot server and include the secret token when one is
configured:

```bash
curl -X POST "https://api.telegram.org/bot<TOKEN>/setWebhook" \
  -H "Content-Type: application/json" \
  -d '{"url": "https://your-host/webhook/telegram", "secret_token": "<SECRET>"}'
```

Verify and inspect:

```bash
curl -s "https://api.telegram.org/bot<TOKEN>/getWebhookInfo" | python3 -m json.tool
curl -s "https://api.telegram.org/bot<TOKEN>/deleteWebhook"
```

The endpoint accepts the Bot API update payload directly (`message`,
`edited_message` and `callback_query`) and always answers `200` once the
delivery is accepted, so Telegram does not retry.

## Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `POST` | `/webhook/telegram` | anonymous + secret token (when configured) | Inbound updates from Telegram |
| `POST` | `/api/telegram/send` | authenticated (`/api/telegram/**` permission) | Send a message from a script or integration |

The inbound path is anonymous because Telegram holds no token — it is a
transport gate, and the authenticity check happens in the handler. Each
inbound route is also exempt from CSRF for the same reason; the outbound
`/api/telegram/send` route keeps its RBAC permission.

## Media ingestion (#1329)

Photos, documents, voice notes, audio tracks and videos are downloaded through
the Bot API and stored in the bot's Drive before the bot sees the message:

```
{bot}.gbai/{bot}.gbdrive/inbox/{generated-name}.{ext}
```

The conversation then carries a marker instead of a placeholder, so a BASIC
script can act on the real path:

| Update type | Marker |
|-------------|--------|
| `photo` | `[image] inbox/9f3c...jpg` |
| `document` | `[document] inbox/report.pdf` |
| `voice` | `[voice] inbox/4a1d...ogg` |
| `audio` | `[audio] inbox/song.mp3` |
| `video` | `[video] inbox/clip.mp4` |

A caption is appended on the following lines. When the download or the Drive
write fails, the marker is still produced with the reason, so the conversation
never becomes empty without explanation:

```
[image] (not stored: getFile failed: Telegram bot token not configured)
```

Storage rules:

- Telegram refuses `getFile` above 20 MB; such a file is skipped with an
  explicit reason.
- Image names are opaque (no user-controlled file name); document names keep
  their original stem with a sanitized name, and a traversing name is rejected.
- Media lands in `inbox/` first — a filing tool (for example the
  `classify_media` template) decides the final folder.

Because the bot receives a Drive path, `CLASSIFY IMAGE "inbox/9f3c.jpg"`,
`GET "inbox/report.pdf"` and any filing keyword work without extra glue. See
[Multimodal](./multimodal.md) for the classification keywords.

## Sessions and routing

Each Telegram `chat_id` maps to one session (`telegram:{chat_id}`), created on
first contact and reused afterwards. While a session has an `assigned_to`
value, messages go to the attendant queue instead of the bot; otherwise they
run through the normal pipeline — suggestions, tools and knowledge base
included. Inline keyboard callbacks are routed back to the bot as message text.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| Telegram reports `401` / delivery failing | `telegram-webhook-secret` is set but the webhook was registered without `secret_token`, or the values differ | Re-run `setWebhook` with the same secret, or clear the key |
| `Telegram bot token not configured` | `telegram-bot-token` missing for that bot | Add it to the bot's `config.csv` |
| Outbound messages fail | revoke/reissued token, or the bot was not started with the `telegram` feature | Update `telegram-bot-token`; confirm the build enables `telegram` (it is part of the default feature set) |
| `[image] (not stored: ...)` in the conversation | download from Telegram failed (token, size limit, network) | Read the reason in the marker; check `getWebhookInfo` and the bot token |
| Attachment missing from Drive | the `inbox/` write failed | Confirm the bot's Drive workspace is mounted for the bot id resolved by the channel |

## See Also

- [Channels](./channels.md) — messaging platform setup
- [Multimodal](./multimodal.md) — image/audio/video classification keywords
- [Teams Channel](./teams-channel.md) — the other BETA chat channel
- [Permissions Reference](../09-security/permissions-reference.md) — RBAC route permissions
