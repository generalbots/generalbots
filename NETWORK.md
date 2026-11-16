# NETWORK.md — Outbound Traffic Reference (Desktop Edition)

This document catalogs every **outbound network connection** that the General Bots
desktop edition (and its sidecar `botserver`) can initiate. It is intended for
network administrators, security reviewers, and offline deployments that must
whitelist or block egress.

> Scope: the desktop-local / desktop-chat-drive builds (`botserver --no-default-features
> --features desktop-local` or `desktop-chat-drive`). The full SaaS/cloud edition
> additionally talks to the General Bots control plane (see "Cloud/SaaS-only").

---

## 1. Conventions

- **Protocol**: HTTPS unless noted. All TLS is verified against the system trust store.
- **Ports**: listed as `host:port`. Default HTTPS = 443, HTTP = 80 when omitted.
- **Trigger**: what causes the connection (boot, on-demand, user action, feature).
- **Required**: whether the feature breaks without it (`required`) or degrades (`optional`).
- **Data sent**: short description of the payload (never secrets in logs).

---

## 2. Core / always-on (desktop builds)

| Host | Port | Protocol | Trigger | Required | Purpose |
|------|------|----------|---------|----------|---------|
| `api.kilo.ai` | 443 | HTTPS | boot + chat | required (default LLM) | Default free-tier LLM gateway (`/api/gateway/chat/completions`) |
| `api.openai.com` | 443 | HTTPS | chat (if configured) | optional | OpenAI chat completions / embeddings |
| `api.anthropic.com` | 443 | HTTPS | chat (if configured) | optional | Anthropic Claude |
| `api.groq.com` | 443 | HTTPS | chat (if configured) | optional | Groq inference |
| `generativelanguage.googleapis.com` | 443 | HTTPS | chat (if configured) | optional | Google Gemini |
| `api.mistral.ai` | 443 | HTTPS | chat (if configured) | optional | Mistral |
| `api.deepseek.com` | 443 | HTTPS | chat (if configured) | optional | DeepSeek |
| `api.x.ai` | 443 | HTTPS | chat (if configured) | optional | xAI Grok |
| `api.together.xyz` | 443 | HTTPS | chat (if configured) | optional | Together AI |
| `api.fireworks.ai` | 443 | HTTPS | chat (if configured) | optional | Fireworks |
| `api.cerebras.ai` | 443 | HTTPS | chat (if configured) | optional | Cerebras |
| `openrouter.ai` | 443 | HTTPS | chat (if configured) | optional | OpenRouter aggregator |
| `api.kilo.ai` | 443 | HTTPS | boot | required | Model catalog / gateway discovery |

---

## 3. LLM provider endpoints (selectable, user-configured)

Configured via Vault / settings. Only the chosen provider is contacted.

| Provider | Host(s) | Port | Path |
|----------|---------|------|------|
| OpenAI | `api.openai.com` | 443 | `/v1/chat/completions`, `/v1/embeddings` |
| Anthropic | `api.anthropic.com` | 443 | `/v1/messages` |
| Google (Gemini) | `generativelanguage.googleapis.com` | 443 | `/v1beta/...` |
| Google (Vertex) | `*-aiplatform.googleapis.com` | 443 | `/v1/projects/.../endpoints/openapi` |
| Azure OpenAI | `{resource}.openai.azure.com` | 443 | `/openai/deployments/...` |
| Groq | `api.groq.com` | 443 | `/openai/v1/chat/completions` |
| Mistral | `api.mistral.ai` | 443 | `/v1/chat/completions` |
| DeepSeek | `api.deepseek.com` | 443 | `/chat/completions` |
| xAI | `api.x.ai` | 443 | `/v1/chat/completions` |
| Ollama (local) | `localhost:11434` | 11434 | (no egress) |
| LM Studio (local) | `localhost:1234` | 1234 | (no egress) |
| llama.cpp (local) | `localhost:8080` | 8080 | (no egress) |

---

## 4. Local services (loopback only, no egress)

| Host | Port | Service |
|------|------|---------|
| `127.0.0.1` | 5432 | PostgreSQL |
| `127.0.0.1` | 6379 | Valkey/Redis cache |
| `127.0.0.1` | 8080 | botserver API (self) |
| `127.0.0.1` | 8081 | botmodels (llama.cpp) |
| `127.0.0.1` | 8200 | Vault |
| `127.0.0.1` | 9000 | MinIO (Drive) |
| `127.0.0.1` | 9100 | MinIO console |
| `127.0.0.1` | 6333 | Qdrant (vectordb) |
| `127.0.0.1` | 3000 | botui (dev) |
| `127.0.0.1` | 4000 | cloud UI (dev) |
| `127.0.0.1` | 5000 | login (dev) |

These never leave the machine.

---

## 5. Optional integrations (user-enabled)

Only contacted when the user configures the integration. Each entry is an outbound
HTTPS call initiated by `botserver` on behalf of the user.

### 5.1 Email / mail providers
- `api.resend.com:443` (Resend)
- `api.postmarkapp.com:443` (Postmark)
- `api.mailgun.net` (Mailgun, via configured base URL)
- `smtp.*` user-configured (IMAP/SMTP) — port 993/465/587

### 5.2 Storage / cloud
- `s3.amazonaws.com`, `*.s3.*.amazonaws.com` (AWS S3)
- `storage.googleapis.com` (GCS)
- `api.dropboxapi.com` (Dropbox)
- `graph.microsoft.com` (OneDrive/SharePoint)

### 5.3 Messaging / social
- `api.slack.com`, `hooks.slack.com`
- `api.telegram.org`
- `graph.facebook.com`, `graph.threads.net`
- `api.twitter.com`, `upload.twitter.com`
- `api.discord.com` (discordapp CDN)
- `web.whatsapp.com` (WhatsApp Web bridge, browser-side)
- `api.zoom.us`
- `api.teams.microsoft.com`

### 5.4 Productivity / CRM
- `api.notion.com`, `api.hubapi.com`, `api.salesforce.com`,
  `api.zendesk.com`, `api.intercom.io`, `api.linear.app`, `api.trello.com`,
  `api.asana.com`, `api.clickup.com`, `api.monday.com`, `api.airtable.com`

### 5.5 Payments
- `api.stripe.com`, `api.paypal.com`, `api.paddle.com`, `api.square.com`

### 5.6 Calendar / scheduling
- `www.googleapis.com` (Google Calendar), `graph.microsoft.com` (Outlook),
  `api.calendly.com`, `api.cal.com`

### 5.7 Search / research
- `api.duckduckgo.com`, `api.search.tinyfish.ai`, `html.duckduckgo.com`
- `api.exchangerate.host`, `api.openweathermap.org`

### 5.8 AI media / generation
- `api.openai.com` (images/audio)
- `generativelanguage.googleapis.com` (Gemini image)
- BotModels gateway (local by default; remote if `BOTMODELS_HOST` set)

---

## 6. Desktop app (Tauri / botapp) egress

The desktop shell itself makes very few outbound calls:

| Host | Port | Purpose |
|------|------|---------|
| `github.com` | 443 | Update checks / release downloads (optional) |
| `raw.githubusercontent.com` | 443 | Asset fetches (optional) |

No telemetry is sent by default.

---

## 7. Web frontend (suite) outbound

When the suite UI runs in a browser (not desktop), it may contact:

- `fonts.googleapis.com`, `fonts.gstatic.com` — web fonts (optional, cached)
- `cdn.jsdelivr.net`, `cdnjs.cloudflare.com` — vendor libs (dev only; production bundles locally)
- The configured LLM provider (proxied via botserver `/api/*`, not direct)

The production build serves all assets locally; no CDN is required at runtime.

---

## 8. Cloud / SaaS-only (NOT in desktop edition)

These are contacted only by the full cloud build and are absent from desktop-local:

- `pragmatismo.com.br`, `cloud.pragmatismo.com.br`, `login.pragmatismo.com.br`
- `chat.pragmatismo.com.br` (bot API gateway)
- `alm.pragmatismo.com.br:4747` (Forgejo / CI)
- General Bots control plane / license servers
- Zitadel identity provider (self-hosted in prod)

---

## 9. Blocked / never contacted (security)

The following are explicitly blocked by the SSRF guard (`botsecurity-core`):

- `169.254.169.254` (cloud metadata)
- `metadata.google.internal`
- `localhost` / `127.0.0.0/8` from tool HTTP calls
- Private ranges `10.0.0.0/8`, `172.16.0.0/12`, `192.168.0.0/16` from tool HTTP calls

---

## 10. Offline mode

To run fully offline:

1. Set LLM provider to a local endpoint (`OLLAMA_HOST`, `LLM_URL=http://127.0.0.1:11434`).
2. Disable all integrations (no API keys configured).
3. The desktop edition makes **zero** outbound connections in this configuration.

---

## 11. How to audit

```bash
# Capture all outbound from the sidecar process:
sudo strace -e trace=network -p $(pgrep botserver) 2>&1 | grep connect
# Or with tcpdump:
sudo tcpdump -i any -n 'tcp[tcpflags] & tcp-syn != 0 and dst port != 5432 and dst port != 6379'
```

All HTTP clients in `botserver` use `reqwest` with the system proxy env
(`HTTP_PROXY`/`HTTPS_PROXY`) respected.

---

*Generated from source scan of `botserver/`, `botapp/`, `botui/`. Update when adding
a new integration or provider.*
