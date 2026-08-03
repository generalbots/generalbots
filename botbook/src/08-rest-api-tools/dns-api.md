# DNS API 🟡 BETA

> **Dynamic DNS hostname registration and removal for service discovery.**

---

## Base URL

```
/api/dns
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Overview

The DNS API provides dynamic hostname registration for services running within the BotServer network. When a service starts, it can register a hostname that maps to its IP address, enabling other services to discover it via DNS resolution.

**Features:**
- Automatic hostname validation (RFC-compliant)
- Rate limiting per IP address (configurable max entries)
- Automatic zone file updates
- Periodic cleanup of stale entries
- TTL-based expiration

---

## Endpoints

### Register Hostname

**`POST /api/dns/register`**

Register a new hostname that maps to the requesting IP address (or a specified IP).

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| hostname | string | Yes | Hostname to register (alphanumeric and hyphens, max 63 chars) |
| ip | string | No | IP address to map (defaults to the requester's IP) |

**Request Body:** None (parameters via query string).

**Response:**
```json
{
  "success": true,
  "hostname": "my-service.botserver.local",
  "ip": "10.0.0.5",
  "ttl": 60
}
```

**Error Responses:**

| Status | Condition |
|--------|-----------|
| 400 | Invalid hostname format |
| 429 | Rate limit exceeded (too many entries for this IP) |
| 500 | Internal error updating zone file |

---

### Remove Hostname

**`POST /api/dns/remove`**

Remove a previously registered hostname.

**Query Parameters:**

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| hostname | string | Yes | Hostname to remove |
| ip | string | No | IP address (used for validation) |

**Request Body:** None (parameters via query string).

**Response:**
```json
{
  "status": "ok"
}
```

---

## Hostname Validation Rules

| Rule | Description |
|------|-------------|
| Length | 1–63 characters |
| Characters | Alphanumeric (`a-z`, `A-Z`, `0-9`) and hyphens (`-`) |
| No leading hyphen | Must not start with `-` |
| No trailing hyphen | Must not end with `-` |
| Case insensitive | Stored in lowercase |

---

## Rate Limiting

Each IP address is limited to a configurable number of hostname registrations (default: 5). When the limit is exceeded, the oldest entry is automatically removed when a new one is registered.

| Setting | Default | Description |
|---------|---------|-------------|
| `max_entries_per_ip` | 5 | Maximum hostnames per IP address |
| `ttl_seconds` | 60 | DNS record time-to-live |
| `cleanup_interval_hours` | 24 | How often stale entries are purged |

---

## Zone File Format

The DNS service generates a standard BIND-compatible zone file. The file includes:

- SOA record for the domain
- NS record pointing to `ns1.botserver.local`
- Static entries for built-in services (`api`, `auth`, `llm`, `mail`, `meet`)
- Dynamic entries registered via this API

**Example zone file output:**
```
$ORIGIN botserver.local.
$TTL 60
@       IN      SOA     ns1.botserver.local. admin.botserver.local. (
                        1705312800      ; Serial
                        3600            ; Refresh
                        1800            ; Retry
                        604800          ; Expire
                        60              ; Minimum TTL
                        )
        IN      NS      ns1.botserver.local.
ns1     IN      A       127.0.0.1

; Static service entries
api     IN      A       127.0.0.1
auth    IN      A       127.0.0.1
llm     IN      A       127.0.0.1
mail    IN      A       127.0.0.1
meet    IN      A       127.0.0.1

; Dynamic entries
my-service     IN      A       10.0.0.5
worker-01      IN      A       10.0.0.6
```

---

## Examples

### Register a Hostname

```bash
curl -X POST "http://localhost:8080/api/dns/register?hostname=my-service" \
  -H "Authorization: Bearer $TOKEN"
```

### Register with Specific IP

```bash
curl -X POST "http://localhost:8080/api/dns/register?hostname=worker-01&ip=10.0.0.6" \
  -H "Authorization: Bearer $TOKEN"
```

### Remove a Hostname

```bash
curl -X POST "http://localhost:8080/api/dns/remove?hostname=my-service" \
  -H "Authorization: Bearer $TOKEN"
```

### From a Service at Startup

```bash
#!/bin/bash
# Register this service's hostname on startup
MY_HOSTNAME="api-gateway"
MY_IP=$(hostname -I | awk '{print $1}')
curl -X POST "http://localhost:8080/api/dns/register?hostname=${MY_HOSTNAME}&ip=${MY_IP}" \
  -H "Authorization: Bearer ${SERVICE_TOKEN}"
```

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 400 | Bad Request (invalid hostname or IP) |
| 401 | Unauthorized |
| 429 | Too Many Requests (rate limit exceeded) |
| 500 | Internal Server Error |

---

## DNS Records (admin)

```http
GET  /api/dns/list                # table rows (HTML)
GET  /api/dns/search?q=           # filtered rows (HTML)
POST /api/dns/register            # form { hostname, record_type, target, ttl }
POST /api/dns/remove              # form { id }
GET  /api/dns/:id/edit            # edit form (HTML)
```

Records are stored in the `dns_records` table.

---

## See Also

- [Admin API](./admin-api.md) — System administration and configuration
- [Monitoring API](./monitoring-api.md) — Health checks and metrics
- [Storage API](./storage-api.md) — File storage operations
