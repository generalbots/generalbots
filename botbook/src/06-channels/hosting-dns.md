# Hosting, DNS, and MDA Integration 🟡 BETA

General Bots integrates with hosting providers, DNS services, and Mail Delivery Agents (MDA) for complete platform deployment.

---

## Overview

A complete General Bots deployment typically includes:

| Component | Purpose | Providers Supported |
|-----------|---------|---------------------|
| **Hosting** | Run botserver | Any VPS, LXC, bare metal |
| **DNS** | Domain registration + records | Porkbun, Cloudflare, Dynadot, Namecheap, deSEC |
| **MDA** | Email delivery | Stalwart, Postfix, external SMTP |
| **AI/LLM** | Language models | OpenAI, Anthropic, local models |

---

## Namecheap Integration

General Bots can automatically manage DNS records via the Namecheap API.

### Configuration

Add to your bot's `config.csv`:

```csv
name,value
namecheap-api-user,your-username
namecheap-api-key,stored-in-vault
namecheap-username,your-username
namecheap-client-ip,your-server-ip
```

> **Note**: API key is stored in Vault, not in config.csv. Only reference it by name.

### Automatic DNS Setup

When deploying a new bot instance, General Bots can:

1. Create A record pointing to your server
2. Create MX records for email
3. Create TXT records for SPF/DKIM/DMARC
4. Create CNAME for www subdomain

### BASIC Keywords for DNS

```basic
' Create DNS record
DNS SET "bot.example.com", "A", server_ip

' Create MX record for email
DNS SET "example.com", "MX", "mail.example.com", 10

' Create SPF record
DNS SET "example.com", "TXT", "v=spf1 mx a ip4:" + server_ip + " -all"

' List current records
records = DNS LIST "example.com"
```

### Registrar Adapters

Registrars are driven by the `botdomains` crate. An adapter can only earn
"records" if it can write **one DNS record at a time**, because the platform
writes A, AAAA, CAA and TXT records on every domain it provisions.

| Registrar | Adapter | API access | Per-record DNS writes | Whole-zone replace | DNSSEC |
|---|---|---|---|---|---|
| **Porkbun** | `porkbun` | open, no gate | ✅ | no | ❌ |
| **Cloudflare Registrar** | `cloudflare` | open | ✅ | no | ✅ |
| **Dynadot** | `dynadot` | open | ✅ | no | ✅ |
| **Namecheap** | `namecheap` | ⚠️ gated | ⚠️ zone replace only | **yes** | ❌ |
| **deSEC** | `desec` | open | ✅ | no | ✅ |
| Manual | — | via config | via config | — | — |

**Porkbun is the default**: open API, flat pricing with no renewal cliff, 400+
TLDs. Cloudflare Registrar is offered where Cloudflare DNS is already
authoritative — at-cost pricing with no markup, though only for the TLDs
Cloudflare resells and no ccTLDs. Dynadot covers breadth (805 TLDs). deSEC is
DNS-only: it manages DNSSEC as its core product but sells no names.

**There is no ACME client in this codebase.** TLS certificates are not issued by
the platform; terminate TLS at your own proxy.

### Namecheap's zone-replace hazard

`domains.dns.setHosts` is Namecheap's only DNS write endpoint and it **replaces
the entire record set**. There is no per-record create, update or delete.

Every single-record change is therefore read-all → modify-in-memory →
write-all, and any record the client fails to re-send is silently deleted from the
customer's zone. `botdomains` computes an explicit `ZoneDiff` before the write and
**refuses** a write that would drop anything, naming the records it would delete.
That guard is what makes the adapter safe to keep selectable.

Two further limits: production API access requires 20 domains in the account, or
$50 in balance, or $50 spent in the last two years; and Namecheap carries the
steepest renewal markup in the adapter group ($10.98 → $18.48 for `.com`).

### Registrar Configuration

Credentials are stored per organization in `cloud_organizations.config` under
`registrar_keys`, never in `config.csv`:

```json
{
  "domain_registrar": "porkbun",
  "registrar_keys": {
    "porkbun": { "api_key": "...", "api_secret": "..." }
  },
  "domain_app_ip": "203.0.113.7",
  "domain_app_host": "app.example.net"
}
```

`domain_app_ip` is written as the apex `A` record and `domain_app_host` as the
`www` CNAME. A deployment also writes two `CAA` records pinning Let's Encrypt,
an SPF record, and a `_dmarc` TXT record, because WhatsApp and email onboarding
both fail without them.

`deSEC` is configured with `with_config(zone)`; Cloudflare with an account id;
Namecheap in production with `production(client_ip)`.

### Domain Provisioning

`botcloud::domain_provisioning` turns a catalogue SKU into a live registration:

1. Resolve the domain name — a base-36 hash of the organization id, so two
   tenants cannot collide and the tenant's UUID is not published in DNS.
2. Walk the SKU's registrar chain (`domain-com` → porkbun, cloudflare, dynadot,
   namecheap). A registrar without a credential, or that does not sell the TLD,
   is skipped; a purchase that fails outright stops the walk.
3. Register, then delegate to the configured nameservers.
4. Write the routing and onboarding records.
5. **Write the `bot_domains` row.** This is the payoff: without it the customer
   buys a domain that routes nowhere. Afterwards
   `GET /api/domains/resolve?host=` answers with the bot.

Renewal state is queryable from the stored `expires_at` and `auto_renew`, so an
expiring domain is visible before it lapses.

---

## Hosting Options

### VPS Providers

General Bots runs on any Linux VPS. The last column distinguishes hosts the
platform can **provision for you** from hosts it merely **runs on**.

| Provider | Minimum Spec | Recommended | Runs GB | Provisioned by the platform |
|----------|--------------|-------------|---------|------------------------------|
| Hetzner Cloud | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ✅ `hetzner` |
| DigitalOcean | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ✅ `digitalocean` |
| Vultr | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ✅ `vultr` |
| Oracle Cloud | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ✅ `oracle` |
| Contabo | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ✅ `contabo` |
| Linode | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU | ✅ | ❌ no adapter |
| AWS EC2 | t3.small | t3.medium | ✅ | ❌ no adapter |
| GCP | e2-small | e2-medium | ✅ | ❌ no adapter |

### GPU / AI Accelerator Providers

| Provider | Adapter | Dispatchable | GPU line | API |
|----------|---------|--------------|----------|-----|
| **Vast.ai** | `vast` | ✅ | RTX 3060–4090, A100, H100, custom | ✅ REST |
| **OVHcloud** | `ovh` | ✅ | A100, H100, H200, L40S, A10, RTX 6000 Ada | ✅ REST, HMAC-SHA1 signed |
| **RunPod** | `runpod` | ✅ | RTX 3060–4090, A100, H100 | ✅ REST |
| **Contabo** | `contabo` | ✅ | A100 80GB, H100, L40S, RTX 4090/3090 | ✅ REST, HMAC-SHA1 signed |
| **Vultr** | `vultr` | ✅ | RTX 3090, RTX 4090, A100, H100 | ✅ REST |

All six compute adapters — `vast`, `contabo`, `runpod`, `vultr`, `hetzner`,
`ovh`, `digitalocean`, `oracle` — implement `provision`, `terminate`,
`get_status` and `list_instances`, and all are compiled and dispatchable. There
is no longer a difference between "code exists behind a disabled feature flag"
and "reachable from botcloud": `botproviders/Cargo.toml` enables every adapter
in `default`.

**Serverless GPU endpoints are out of scope.** `ComputeProvider` models
instances with a lifecycle; RunPod Serverless needs a different abstraction and is
not provisioned by the platform.

#### Provisioning

`botcloud::compute_provisioning` owns the policy:

- Each SKU carries an ordered candidate chain, so `vps-large` is no longer
  unprovisionable the moment Contabo is out.
- A capacity or authentication failure moves to the next candidate; any other
  error stops the walk, because retrying the same bad request against six vendors
  helps nobody.
- A spec above a provider's largest plan is **refused**, never substituted with a
  smaller machine.
- Credentials are per organization and per provider
  (`cloud_organizations.config` → `provider_keys`); `GB_PROVIDER` remains the
  global default and `GB_COMPUTE_REGION` the default region.
- `ProvisionResult::hourly_cost` is reconciled against the catalogue price and a
  variance beyond 5% is logged and stored on the resource, so billing above the
  advertised price is visible rather than discovered in support.

```bash
# List an adapter's catalogue without provisioning anything
# (registry::provider_info_for_name("hetzner"))
```

#### RunPod

RunPod's adapter drives persistent GPU pods through `api.runpod.io/v2`:
provision a GPU request, poll its status, and **stop then delete** it — stopping
alone leaves the request reserving the GPU at full rate, which is why the adapter
performs both calls and reports a failure if the delete does not land.

#### Vast.ai

A GPU marketplace with low entry prices, suitable for batch processing and
experimentation:
- Marketplace pricing (rent from other users' hardware)
- Offer search picks the cheapest bundle satisfying the requested GPU count
- Docker-based deployment
- Automatic port forwarding and SSH access
- Weaker uptime SLAs than dedicated providers

#### Configuration

Add GPU provider credentials to `config.csv`:
```csv
runpod-api-key, your-key-here
vastai-api-key, your-key-here
vultr-api-key, your-key-here
```

### LXC Container Deployment

Recommended for production isolation:

```bash
# Create container
lxc launch ubuntu:22.04 botserver

# Configure resources
lxc config set botserver limits.memory 4GB
lxc config set botserver limits.cpu 2

# Forward ports
lxc config device add botserver http proxy listen=tcp:0.0.0.0:80 connect=tcp:127.0.0.1:8080
lxc config device add botserver https proxy listen=tcp:0.0.0.0:443 connect=tcp:127.0.0.1:8443

# Set environment for Vault
lxc config set botserver environment.VAULT_ADDR="http://vault:8200"

# Deploy
lxc exec botserver -- ./botserver
```

### Docker Deployment

```yaml
version: '3.8'
services:
  botserver:
    image: generalbots/botserver:latest
    ports:
      - "8080:8080"
    environment:
      - VAULT_ADDR=http://vault:8200
    volumes:
      - ./bots:/app/bots
      - ./botserver-stack:/app/botserver-stack
```

---

## MDA (Mail Delivery Agent) Integration

General Bots includes Stalwart mail server for complete email functionality.

### Built-in Stalwart

Stalwart is automatically configured during bootstrap:

| Feature | Status |
|---------|--------|
| IMAP | ✅ Enabled |
| SMTP | ✅ Enabled |
| JMAP | ✅ Enabled |
| Spam filtering | ✅ SpamAssassin |
| Virus scanning | ✅ ClamAV |
| DKIM signing | ✅ Auto-configured |

### Email Configuration

In `config.csv`:

```csv
name,value
email-domain,example.com
email-dkim-selector,mail
email-spam-threshold,5.0
email-max-size-mb,25
```

### DNS Records for Email

Required DNS records (auto-created with Namecheap integration):

| Record | Type | Value |
|--------|------|-------|
| `mail.example.com` | A | Your server IP |
| `example.com` | MX | `mail.example.com` (priority 10) |
| `example.com` | TXT | `v=spf1 mx a -all` |
| `mail._domainkey.example.com` | TXT | DKIM public key |
| `_dmarc.example.com` | TXT | `v=DMARC1; p=quarantine` |

### External SMTP

To use external email providers instead:

```csv
name,value
smtp-host,smtp.sendgrid.net
smtp-port,587
smtp-user,apikey
smtp-secure,tls
```

Credentials stored in Vault:

```bash
vault kv put secret/botserver/smtp password="your-api-key"
```

---

## AI/LLM Integration

### Supported Providers

| Provider | Models | Config Key |
|----------|--------|------------|
| OpenAI | GPT-5, o3 | `llm-url=https://api.openai.com/v1` |
| Anthropic | Claude Sonnet 4.5, Opus 4.5 | `llm-url=https://api.anthropic.com` |
| Groq | Llama 3.3, Mixtral | `llm-url=https://api.groq.com/openai/v1` |
| DeepSeek | DeepSeek-V3, R3 | `llm-url=https://api.deepseek.com` |
| Local | Any GGUF | `llm-url=http://localhost:8081` |

### Local LLM Setup

Run local models with BotModels:

```bash
# Install BotModels
./botserver install llm

# Download a model
./botserver model download llama-3-8b

# Configure in config.csv
```

```csv
name,value
llm-url,http://localhost:8081
llm-model,llama-3-8b.gguf
llm-context-size,8192
llm-gpu-layers,35
```

### AI Features

| Feature | Description |
|---------|-------------|
| **Conversation** | Natural language chat |
| **RAG** | Knowledge base search |
| **Tool Calling** | Automatic BASIC tool invocation |
| **Embeddings** | Document vectorization |
| **Vision** | Image analysis (multimodal models) |
| **Voice** | Speech-to-text, text-to-speech |

---

## Complete Deployment Example

### 1. Provision Server

```bash
# On your VPS
wget https://github.com/generalbots/generalbots/releases/latest/botserver
chmod +x botserver
```

### 2. Configure DNS (Namecheap)

```basic
' setup-dns.bas
domain = "mybot.example.com"
server_ip = "203.0.113.50"

DNS SET domain, "A", server_ip
DNS SET "mail." + domain, "A", server_ip
DNS SET domain, "MX", "mail." + domain, 10
DNS SET domain, "TXT", "v=spf1 mx a ip4:" + server_ip + " -all"

PRINT "DNS configured for " + domain
```

### 3. Start botserver

```bash
./botserver
```

### 4. Terminate TLS at your proxy

The platform issues no certificates: there is no ACME client and no
`botserver ssl` command. Terminate TLS at your own proxy — the Caddy
configuration on the `proxy` container is where certificates are managed. The
`CAA` records written at registration pin Let's Encrypt as the only issuer
allowed for the zone.

### 5. Verify Email

```basic
' test-email.bas
SEND MAIL "test@gmail.com", "Test from General Bots", "Email is working!"
PRINT "Email sent successfully"
```

---

## Troubleshooting

### DNS Not Propagating

1. Check the registrar credential in `cloud_organizations.config` →
   `registrar_keys`
2. On Namecheap, confirm the client IP is the one registered with the account —
   the API rejects calls from any other address
3. Check the resource's stored config for `dns_records_refused`: a record the
   registrar rejected will never propagate
4. Wait up to 48 hours, then verify with `dig` or `nslookup`

### Email Marked as Spam

1. Verify SPF record is correct
2. Check DKIM signature is valid
3. Ensure DMARC policy is set
4. Check IP reputation at mxtoolbox.com

### Certificate Errors

These are proxy-side, not botserver-side.

1. Verify the DNS `A` record points at the proxy
2. Check the proxy's own certificate store and renewal schedule
3. Confirm the `CAA` records do not pin an issuer the proxy does not use —
   registration writes `CAA 0 issue "letsencrypt.org"`

### LLM Connection Failed

1. Verify `llm-url` in config.csv
2. Check API key in Vault
3. Test endpoint with curl
4. Review botserver logs

---

## See Also

- [LLM Providers](./llm-providers.md) — Detailed LLM configuration
- [Storage](./storage.md) — S3-compatible storage setup
- [Directory](./directory.md) — User authentication
- [Channels](./channels.md) — WhatsApp, Telegram, etc.
- [Installation](../01-getting-started/installation.md) — Full installation guide