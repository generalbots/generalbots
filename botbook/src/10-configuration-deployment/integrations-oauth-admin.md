# Integrations OAuth Administration Runbook

Integrations authenticate through the platform OAuth2 authorization-code
broker (`/api/bots/:bot_id/integrations/oauth/:provider/start` and
`/api/integrations/oauth/:provider/callback`). Client secrets never live in
code: each deployment supplies its own provider application credentials,
stored strictly in Vault.

## 0. One app per deployment — why not a single public client id

General Bots is open source: every installation runs on someone else's machine
and domain. Shipping one maintainer-owned Google/Microsoft client id inside the
product does not work, and it is not what the vendors allow:

- **Exact redirect URIs.** Google and Microsoft require every redirect URI to
  be registered verbatim — no wildcards — so a single client cannot cover
  `https://<any-host>/api/integrations/oauth/gmail/callback`.
- **Restricted scopes.** `gmail.readonly`, `drive.readonly` and Microsoft
  `Mail.Read` are sensitive/restricted scopes. A client distributed to unknown
  third parties must pass vendor verification (and, for Google restricted
  scopes, an annual security assessment), and the maintainers would become the
 data controller for every installer's mailbox.
- **Shared quota and abuse.** One client means one quota, one consent screen
  and one suspension risk: a single abusive deployment can break integrations
  for everyone.
- **Trust.** A self-hoster's users should see *that operator's* application
  name and legal terms on the consent screen, not the upstream project's.

So: **each deployment registers its own app once** (about five minutes), and
the operator configures it in Vault. Users never see a client id — they only
click *Connect* and sign in.

Two supported shapes:

| Shape | When | What is stored |
|---|---|---|
| Web client (**recommended**) | server install with a public hostname | `client_id` + `client_secret` |
| Public/desktop client | local install, loopback or device redirect, no secret | `client_id` only (PKCE is always sent, and the flow works without a secret) |

The local/desktop edition may ship a client id for its own loopback redirect
(`http://127.0.0.1:<port>/...`) — that is the one case where a bundled,
non-secret client id is legitimate; the vendor treats it as an installed app.

## 1. Register the application with the vendor

Create **one** developer application per provider family on the vendor console.
The callback is the same for every bot in the deployment:

```
https://{your-host}/api/integrations/oauth/{provider}/callback
```

| Family | Vendor console | Callback |
|---|---|---|
| Google (Gmail, Drive, Calendar, Photos, Forms) | console.cloud.google.com → OAuth consent screen + OAuth client (Web) | `/api/integrations/oauth/gmail/callback`, `/api/integrations/oauth/drive/callback`, … |
| Microsoft (Outlook, Outlook Calendar, OneDrive, SharePoint) | portal.azure.com → App registrations → Web redirect URI | `/api/integrations/oauth/onedrive/callback`, … |
| HubSpot | app.hubspot.com → App Marketplace | `/api/integrations/oauth/hubspot/callback` |
| Intercom | app.intercom.io → Developer Hub | `/api/integrations/oauth/intercom/callback` |
| Todoist | developer.todoist.com → Apps | `/api/integrations/oauth/todoist/callback` |
| Zoom | marketplace.zoom.us → OAuth app | `/api/integrations/oauth/zoom/callback` |
| Notion | notion.so/my-integrations → External OAuth | `/api/integrations/oauth/notion/callback` |
| Meta (Instagram, Facebook Pages) | developers.facebook.com → Facebook Login | `/api/integrations/oauth/instagram/callback` |

The bot id travels inside the HMAC-signed `state`, which is what lets one
callback serve every bot. Stage and production are registered separately
because the host differs.

Google requires the app to be **published** (or the user added as a test user)
before anyone outside the project can consent; Gmail/Drive also need their API
enabled in the project and the scopes added to the consent screen.

## 2. Store the client in Vault

Resolution order, first match wins:

```
gbo/{org_id}/{branch_id}/{bot_id}/integrations/oauth/{provider}   # per bot
gbo/{org_id}/{branch_id}/integrations/oauth/{provider}            # per branch
integrations/oauth/{provider}                                     # platform-wide
```

For a normal install, configure the platform-wide entry once:

```bash
vault kv put secret/integrations/oauth/gmail \
    client_id=<id> client_secret=<secret>

vault kv put secret/integrations/oauth/onedrive \
    client_id=<id> client_secret=<secret>
```

Describe a branch or a bot only when a tenant wants its own application. A
missing entry makes *Authorize* answer HTTP 412 with the path it looked for.

**Google needs offline consent** for a durable connection: the flow already
sends `access_type=offline&prompt=consent`, which is what makes Google return a
refresh token instead of a one-hour access token.

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
| instagram | `instagram_basic,instagram_content_publish,pages_show_list,pages_read_engagement,business_management` |
| facebook_pages | `pages_show_list,pages_read_engagement,pages_manage_posts` |

Least privilege: request only what enabled actions need; rotate client
secrets on the vendor console and update Vault in the same window.

## 4. Token lifecycle

Access tokens, rotated refresh tokens and expiry are persisted automatically
(`token_refresh` worker runs hourly and refreshes anything expiring within two
hours). No administrator action is required after initial configuration other
than keeping the Vault envelope current when vendors rotate secrets.

### Meta (Instagram, Facebook Pages)

The Meta code exchange answers with a user token that expires in about one
hour, which is far shorter than any publishing workflow. The callback upgrades
it in place with `grant_type=fb_exchange_token` and stores the resulting
long-lived token (about 60 days); if the upgrade call fails the short-lived
token is kept and the connection simply needs re-authorizing sooner.

Instagram publishing additionally requires a Facebook Page token rather than
the user token. The adapter resolves it at call time from
`/me/accounts?fields=instagram_business_account,access_token`, so:

- the authorized user must manage the Page the Instagram account is linked to;
- the Instagram account must be a Business or Creator account (personal
  accounts cannot publish through the Graph API);
- a system-user token, or an envelope carrying `ig_user_id` together with
  `page_access_token`, bypasses the discovery call.

## 5. Troubleshooting

- `412` at start: Vault envelope missing or lacks keys — recheck §2 path.
- `state signature mismatch`: INTERNAL_API_TOKEN differs between nodes.
- Vendor `invalid_grant`: refresh token revoked — disconnect and reconnect.
