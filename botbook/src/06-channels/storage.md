# Storage Services 🟡 BETA

Drive stores bot files on an S3-compatible endpoint. The default is the
self-hosted MinIO that ships with the stack; Backblaze B2, Cloudflare R2 and
Wasabi are selectable without a rebuild.

The backend, its region and an optional failover endpoint are read from the
`secret/gbo/drive` Vault secret. `botdrive::storage_backends` owns the catalog
and `botdrive::s3_repository` consumes it.

## Private by default

Drive is private. There is no public bucket and no direct-to-S3 URL. The only
public surface is a revocable share link:

- A link is minted only for a file the authenticated owner can already reach;
  the storage key is captured at creation time, so no scope or user input reaches
  the public path.
- The token is a random 128-bit UUID, never derived from the file name.
- The public endpoint resolves a token and nothing else. It cannot list, search
  or enumerate.
- A revoked or expired link answers exactly like an unknown one, so existence is
  never leaked.
- Files are served as an attachment with `nosniff` and `private, no-store`
  caching, so a revoked link stops working immediately and an HTML or SVG payload
  cannot execute in the browser's origin.

The handler is `public_link_download` in
`botserver/crates/botdrive/src/drive_handlers.rs`; the token is stored in the
`drive_public_links` table (migration `6.5.61-drive-public-links`).

**Why this matters for backend choice:** bytes reach a client by traversing
botserver. Egress is proxied, link-gated, and bounded by how many links tenants
actually mint — Drive is not a public CDN origin. Presigned direct-to-S3 URLs are
deliberately not offered, because they would bypass revocation.

## Backends

| Backend | Vault `backend` | Storage /TB/mo | Egress | Minimums | Runtime buckets |
|---|---|---|---|---|---|
| MinIO | `minio` | self-hosted | none | none | ✅ default |
| Backblaze B2 | `b2` | $6.95 | free to 3× stored, then $0.01/GB | none | ✅ |
| Cloudflare R2 | `r2` | $15.00 | **$0 unlimited** | none | ✅ |
| Wasabi | `wasabi` | $7.99 | $0, fair use 1:1 | **1 TB floor, 90-day term** | ⚠️ see below |

Prices are public list rates, mid-2026.

### Wasabi is not the Drive default

Wasabi's headline numbers are genuinely good — $7.99/TB with no egress fee and
no API request fee. Three specifics rule it out for `.gbdrive` and `.gbai`:

1. **1 TB minimum billing.** Store 50 GB, pay $7.99. A platform with many small
   organizations pays that floor per tenant, which is structurally wrong.
2. **90-day minimum retention.** Objects deleted inside the window are billed for
   the full window. Drive holds `.ast` recompiles and transient media whose
   lifecycle is far shorter.
3. **Fair-use egress at 1:1.** Acceptable for a proxied private Drive — and it
   removes the main reason to pick Wasabi in the first place.

Selecting `backend=wasabi` for a runtime bucket logs a warning naming the floor
and the retention term. It remains a reasonable **archive** tier for long-lived
data — knowledge-base originals, backup snapshots — where the 90-day term is
aligned with the data and the floor is met by aggregate volume.

### Recommendation

| Tier | Backend | Why |
|---|---|---|
| Hot runtime path | MinIO | unchanged; no egress charge, no external dependency |
| Cold / archive / KB originals | Backblaze B2 | cheapest at $6.95/TB, no floor, no retention term, 3× egress included |
| Egress-heavy share links | Cloudflare R2 | $0 egress, at $15/TB storage |

## Configuration

`secret/gbo/drive` keys:

| Key | Required | Meaning |
|---|---|---|
| `host`, `port` | for MinIO | endpoint components, used as `http://host:port` |
| `endpoint` | for B2/R2/Wasabi | full URL; wins over `host`/`port` |
| `backend` | no | `minio` (default), `b2`, `wasabi`, `r2` |
| `region` | no | signing region; defaults per backend (`us-west-004` for B2) |
| `fallback_endpoint` | no | secondary endpoint used for read failover |
| `accesskey`, `secret`, `bucket` | yes | credentials and default bucket |

Backblaze B2 and Cloudflare R2 both **require** an explicit region — signing
against the wrong one is what makes them unusable — so the backend supplies a
default when the key is absent.

Env-var equivalents, for deployments that skip Vault: `MINIO_ENDPOINT`,
`MINIO_REGION`, `MINIO_BACKEND`, `MINIO_FALLBACK_ENDPOINT`.

An unknown `backend` value falls back to MinIO rather than failing the boot; a
typo in a secret should not take Drive offline.

## Failover

A configured `fallback_endpoint` is used for **reads only**, and only after the
primary endpoint fails. A write is never silently redirected: two divergent
copies of a bot's source tree are worse than a failed upload, and the fallback
read is logged so an operator can act on a real outage.

Switching backends does not trigger a Drive resync storm — `drive_monitor`
reloads on ETag change, and a backend switch is a process-level change rather
than a Drive content change.

## Egress observability

`EgressMeter` counts bytes read and written per repository, and reports
`billable_egress` against the backend's allowance:

| Backend | Allowance |
|---|---|
| B2 | 3× the stored volume |
| Wasabi | 1:1, and a breach is flagged by the provider |
| R2 | unmetered |
| MinIO | unmetered |

Without this, B2's overage and Wasabi's fair-use limit arrive with the invoice.
Read it with `S3Repository::egress()`.

## Bucket layout

| Bucket | Layout | Contents |
|---|---|---|
| `{bot}.gbai` | layout 1 | a standalone bot's own prefix tree |
| `{org}.gborg` | layout 2 | an organization holding `{workspace}.gbai/` prefixes |

Under each workspace prefix: `{bot}.gbdrive/` for user files and media,
`{bot}.gbot/` for channel prompts, `{bot}.gbdialog/` for BASIC scripts and
`{bot}.gbkb/` for the knowledge base. Discovery is implemented in
`botserver/src/main_module/drive_monitors.rs`.

## Related

- [Storage and Data](../03-knowledge-ai/storage.md) — how the storage layers fit together
- [MinIO Configuration](../10-configuration-deployment/minio.md) — running the default backend
