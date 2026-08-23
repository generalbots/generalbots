# Domains & Bot Subdomains

Every published bot is reachable on the web in two ways:

| Mode | URL | Requires |
|------|-----|----------|
| **Platform subdomain** (default) | `https://{botname}.generalbots.org` | Nothing — automatic |
| **URL path** | `https://{host}/{botname}` | Nothing — automatic |
| **Custom domain** | `https://chat.yourbrand.com` | `bot_domains` row + DNS |

Both the subdomain and the URL path resolve to the **same bot** — publishing a bot makes it instantly available as `{botname}.generalbots.org` with zero extra configuration. Users who have not purchased a custom domain never need one.

## Platform Subdomain Mode

The platform domain is parameterized:

```
GB_PLATFORM_DOMAIN=generalbots.org   # default
```

- Set it on **botserver** and **botui** when self-hosting under your own platform domain (e.g. `GB_PLATFORM_DOMAIN=bots.mycompany.com`).
- Any host `{botname}.{GB_PLATFORM_DOMAIN}` serves the bot named `{botname}`.
- The apex and reserved service hosts (`chat.`, `www.`, `api.`, `app.`, `docs.`, `store.`, `cloud.`, `login.`, `admin.`, `mail.`, …) serve the **default** bot instead.
- Bot names are matched case-insensitively against active bots.

### Self-hosting DNS setup

1. Point a **wildcard** DNS record at your server: `*.{platform} → server IP` (CoreDNS zone: `* IN A <ip>`). New bots then require **zero** DNS changes.
2. Add a reverse-proxy (Caddy) site block per exposed host, or use a wildcard certificate (DNS-01 challenge) for `*.{platform}`.
3. Set `GB_PLATFORM_DOMAIN` on both botserver and botui, restart, and publish bots — each one appears at `{botname}.{platform}` automatically.

## Custom Domain Mode

Users with their own domain (e.g. `chat.yourbrand.com`):

1. Add a mapping in the **Domain Manager** (`/domains` on the cloud UI, admin only) or via API — see below.
2. Point the domain's DNS `A` record at the server.
3. The proxy terminates TLS (Caddy issues the certificate automatically via HTTP-01) and routes to the suite; the UI resolves the bot from the `Host` header.

### Database

**Table:** `bot_domains`

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID PK | Unique identifier |
| `domain` | VARCHAR(255) UNIQUE | The hostname (e.g. `chat.yourbrand.com`) |
| `bot_id` | UUID FK → bots | Which bot this domain routes to |
| `org_id` | UUID FK → organizations (optional) | Org scope for multi-tenant |
| `branch_id` | UUID FK → branches (optional) | Branch scope for multi-tenant |
| `created_at` / `updated_at` | TIMESTAMPTZ | Timestamps |

### API

| Method | Endpoint | Auth | Description |
|--------|----------|------|-------------|
| `GET` | `/api/cloud/domains` | JWT + super admin | List all mappings |
| `POST` | `/api/cloud/domains` | JWT + super admin | Create mapping |
| `PUT` | `/api/cloud/domains/{id}` | JWT + super admin | Update mapping |
| `DELETE` | `/api/cloud/domains/{id}` | JWT + super admin | Delete mapping |
| `GET` | `/api/domains/resolve?host=` | Anonymous | Resolve hostname → bot (exact → platform subdomain → wildcard) |

## Resolution Order

`GET /api/domains/resolve?host=<host>` tries, in order:

1. **Exact match** — a `bot_domains` row for the full hostname.
2. **Platform subdomain** — host ends with `.{GB_PLATFORM_DOMAIN}` → bot by subdomain name (`match_type: "platform_subdomain"`).
3. **Wildcard** — a `*.domain` row whose pattern matches the host (`match_type: "wildcard"`).

The UI server (botui) resolves platform subdomains **locally** (no API call) and only calls the resolve API for custom domains, so subdomain publishing works even before a user signs in.
