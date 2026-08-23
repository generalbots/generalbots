# Integrations OAuth Administration Runbook

Tenant-scoped integrations authenticate through the platform OAuth2
authorization-code broker (`/api/bots/:bot_id/integrations/oauth/:provider/start`
and `/callback`). The broker never embeds client secrets in code: each branch
supplies its own provider application credentials, stored strictly in Vault.
This runbook is the administrator checklist for enabling a provider family.

## 1. Register the platform application with the vendor

For every provider family below, create one developer application on the
vendor console. Register the callback exactly as shown, replacing
`chat.pragmatismo.com.br` when operating another domain.

| Family | Vendor console | Callback URL |
|---|---|---|
| HubSpot | app.hubspot.com → App Marketplace → Develop an app | `https://chat.pragmatismo.com.br/api/bots/*/integrations/oauth/hubspot/callback` (register per-bot exact path) |
| Intercom | app.intercom.io → Developer Hub | `.../integrations/oauth/intercom/callback` |
| Todoist | developer.todoist.com → Apps | `.../integrations/oauth/todoist/callback` |
| Zoom | marketplace.zoom.us → Server-to-server is NOT used here; build OAuth app | `.../integrations/oauth/zoom/callback` |
| Notion | notion.so/my-integrations → External OAuth | `.../integrations/oauth/notion/callback` |
| Google (Drive, Calendar, Sheets, Tasks, Photos, Forms, YouTube Analytics) | console.cloud.google.com → OAuth consent screen + OAuth client (Web) | One client may serve all: `.../integrations/oauth/<google_slug>/callback` for every slug in use; add scopes listed in §3 to the consent screen and mark the app production |
| Microsoft (Outlook, Outlook Calendar, OneDrive, SharePoint) | portal.azure.com → App registrations → Web redirect URI | `.../integrations/oauth/<ms_slug>/callback`; enable `offline_access` |

The botserver builds `redirect_uri` from the incoming request host, so stage
and production environments are registered separately.

## 2. Store client credentials in Vault (per branch)

Path contract:

```
gbo/{org_id}/{branch_id}/{bot_id}/integrations/oauth/{provider}
```

Keys: `client_id`, `client_secret`. Example:

```bash
vault kv put secret/gbo/<org>/<branch>/<bot>/integrations/oauth/hubspot \
    client_id=<id> client_secret=<secret>
```

Google/Microsoft families repeat the command once per concrete slug that the
workspace will authorize (`google_drive`, `outlook`, ...). A missing envelope
makes the Authorize button answer HTTP 412 with an explanatory message.

## 3. Scopes requested by the built-in flow

| Provider | Scope string |
|---|---|
| hubspot | `crm.objects.contacts.write crm.objects.deals.read` |
| todoist | `data:read_write` |
| google_drive | `https://www.googleapis.com/auth/drive.readonly` |
| google_calendar | `https://www.googleapis.com/auth/calendar.events` |
| google_photos | `https://www.googleapis.com/auth/photoslibrary.readonly` |
| google_forms | `https://www.googleapis.com/auth/forms.body` |
| youtube_analytics | `https://www.googleapis.com/auth/yt-analytics.readonly` |
| outlook / outlook_calendar / onedrive / sharepoint | `offline_access https://graph.microsoft.com/.default` |

Least privilege: request only what enabled actions need; rotate client
secrets on the vendor console and update Vault in the same window.

## 4. Token lifecycle

Access tokens, rotated refresh tokens and expiry are persisted automatically
(`token_refresh` worker runs hourly and refreshes anything expiring within two
hours). No administrator action is required after initial configuration other
than keeping the Vault envelope current when vendors rotate secrets.

## 5. Troubleshooting

- `412` at start: Vault envelope missing or lacks keys — recheck §2 path.
- `state signature mismatch`: INTERNAL_API_TOKEN differs between nodes.
- Vendor `invalid_grant`: refresh token revoked — disconnect and reconnect.
