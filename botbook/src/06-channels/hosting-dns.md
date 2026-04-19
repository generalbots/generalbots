# Hosting, DNS, and MDA Integration

General Bots integrates with hosting providers, DNS services, and Mail Delivery Agents (MDA) for complete platform deployment.

---

## Overview

A complete General Bots deployment typically includes:

| Component | Purpose | Providers Supported |
|-----------|---------|---------------------|
| **Hosting** | Run botserver | Any VPS, LXC, bare metal |
| **DNS** | Domain management | Namecheap, Cloudflare, Route53 |
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

### Supported DNS Providers

| Provider | API Support | Auto-SSL |
|----------|-------------|----------|
| Namecheap | ✅ Full | ✅ Let's Encrypt |
| Cloudflare | ✅ Full | ✅ Native |
| Route53 | ✅ Full | ✅ ACM |
| DigitalOcean | ✅ Full | ✅ Let's Encrypt |
| Manual | Via config | Manual |

---

## Hosting Options

### VPS Providers

General Bots runs on any Linux VPS:

| Provider | Minimum Spec | Recommended |
|----------|--------------|-------------|
| DigitalOcean | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU |
| Linode | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU |
| Vultr | 2GB RAM, 1 vCPU | 4GB RAM, 2 vCPU |
| Hetzner | 2GB RAM, 2 vCPU | 4GB RAM, 2 vCPU |
| AWS EC2 | t3.small | t3.medium |
| GCP | e2-small | e2-medium |

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
wget https://github.com/GeneralBots/botserver/releases/latest/botserver
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

### 4. Configure SSL

```bash
# Auto-configured with Let's Encrypt
./botserver ssl enable mybot.example.com
```

### 5. Verify Email

```basic
' test-email.bas
SEND MAIL "test@gmail.com", "Test from General Bots", "Email is working!"
PRINT "Email sent successfully"
```

---

## Troubleshooting

### DNS Not Propagating

1. Check Namecheap API credentials
2. Verify client IP is whitelisted
3. Wait up to 48 hours for propagation
4. Use `dig` or `nslookup` to verify

### Email Marked as Spam

1. Verify SPF record is correct
2. Check DKIM signature is valid
3. Ensure DMARC policy is set
4. Check IP reputation at mxtoolbox.com

### SSL Certificate Errors

1. Verify DNS A record points to server
2. Check port 80 is accessible for ACME challenge
3. Review Let's Encrypt rate limits
4. Check certificate expiry

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