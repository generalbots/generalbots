# Teams Channel 🟡 BETA

Microsoft Teams delivers messages as Bot Framework activities: the platform
posts each activity to `/api/msteams/messages`, the bot resolves the
conversation, routes it through the normal pipeline and answers with an
activity posted back to the service URL.

## Configuration

Store the credentials in the bot's `config.csv` (`.gbot` folder):

| Key | Required | Purpose |
|-----|----------|---------|
| `teams-app-id` | yes | Azure Bot registration (Microsoft App ID) |
| `teams-app-password` | yes | Client secret for that registration |
| `teams-tenant-id` | no | Single-tenant registrations; empty for multi-tenant |
| `teams-bot-id` | no | Bot identity as Teams sees it; defaults to `teams-app-id` |
| `teams-service-url` | no | Bot Framework endpoint; defaults to `https://smba.trafficmanager.net/teams` |

```csv
key,value
teams-app-id,00000000-0000-0000-0000-000000000000
teams-app-password,<client-secret>
teams-tenant-id,<tenant-guid>
```

When `teams-app-id` or `teams-app-password` is absent the adapter refuses to
send (`Teams adapter not configured`) and reports it in the log rather than
posting an unauthenticated call.

## Endpoints

| Method | Path | Auth | Purpose |
|--------|------|------|---------|
| `POST` | `/api/msteams/messages` | anonymous | Inbound Bot Framework activities |
| `POST` | `/api/msteams/send` | authenticated (`/api/msteams/**` permission) | Send an activity from a script or integration |

The inbound route is anonymous because Teams posts from its own infrastructure
without a token; it is also exempt from CSRF and registered as an anonymous
RBAC route for the same reason. Without those three registrations the activity
never reaches the handler — the same defect that made the Telegram channel
unreachable (#1327, #1330).

## How the credentials are resolved

All three channel adapters (Telegram, Teams, Instagram) read their credentials
through one shared reader, `channel_support::make_channel_config_reader`, which
goes through `ConfigManager::get_config`:

1. the bot's own scope (`config.csv` / per-bot Vault path),
2. the workspace scope,
3. the global fallback (`secret/gbo/llm` and friends).

A value that is genuinely absent resolves to *empty*, and the adapter reports
the missing key by name (for example
`Telegram adapter not configured. Please set telegram-bot-token in the bot
configuration database`). The reader never substitutes a placeholder — a
placeholder token would reach the provider and come back as an opaque
`Unauthorized`, which is exactly the failure mode of the former stub reader.
Instagram addresses the bot by string handle, so it uses
`make_channel_config_reader_by_handle`, which resolves a UUID directly and any
other handle by bot name.

## Attachments

Inbound activities carrying attachments are described to the bot; text
activities route as normal messages. Conversation state is keyed by the Teams
conversation id, mirroring the Telegram `chat_id` behaviour, and messages for a
session with an attendant assigned go to the attendant queue.

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `Teams adapter not configured` | `teams-app-id` / `teams-app-password` missing for that bot | Add both to the bot's `config.csv` |
| Activities never arrive | the inbound route is not reachable (registration missing after an upgrade) | `curl -i -X POST https://<host>/api/msteams/messages -H 'Content-Type: application/json' -d '{}'` must answer `422` (body validation) — `401` means the route lost its anonymous registration |
| `401 Unauthorized` from the Bot Framework | wrong/rotated client secret, or a single-tenant registration without `teams-tenant-id` | Reissue the secret, set the tenant id |
| Bot answers nothing in Teams | outbound call rejected (service URL or app id mismatch) | Check `teams-bot-id` / `teams-service-url` and the Bot Framework channel registration |

## See Also

- [Channels](./channels.md) — messaging platform setup
- [Telegram Channel](./telegram-channel.md) — the other BETA chat channel, with the same inbound-webhook requirements
- [Permissions Reference](../09-security/permissions-reference.md) — RBAC route permissions
