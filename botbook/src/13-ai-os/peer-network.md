# Peer Network (P2P Mesh)

General Bots can run as a peer in an opt-in, privacy-preserving mesh. Every
desktop install is a potential node; joining is always explicit and can be
stopped at any time.

## What the network is — and is not

One binary plays every role; the role is what an operator configures, not a
different build.

| Role | Ships today | What it is |
|---|---|---|
| **Peer (desktop)** | ✅ `core/peer_network.rs` | A desktop install that optionally joins the mesh. Off by default; a Settings switch stops all traffic. |
| **Always-on node (VPS)** | ✅ same binary, configured from Vault | A node that never sleeps, so other peers can reach it without the desktop being online. The public `peer.generalbots.org` node is deployed this way (see *Enabling the public node* below). |
| **Volunteer node / relay** | 🟡 planned | A peer that forwards ciphertext for others. The payload data layer exists (`core/peer_payloads.rs`); there is no volunteer mode in `peer_network.rs` yet, so no reader should assume one can help route traffic today. |

What the network is **not**:

- **No central store of files, messages or contacts.** Payloads are
  end-to-end encrypted; a relay holds only an encrypted copy that expires
  (`core/peer_payloads.rs:5-8`). Registrations live in an in-memory table,
  keyed by node id, that expires after `heartbeat-ttl-secs` (default 90 s).
- **No telemetry.** No analytics, usage reports or crash beacons travel
  through the mesh (see `PRIVACY.md`).
- **Not required.** With the mesh off, every request stays on `127.0.0.1`.

## What actually travels

Registration and heartbeat are the only plaintext messages, and they carry
exactly the four fields of `RegisterRequest` / `HeartbeatRequest`
(`core/peer_network_types.rs:100-119`):

| Message | Fields | Retained by a helper |
|---|---|---|
| register | `node_id`, `name`, optional `address`, `capabilities` | until the TTL expires, in memory |
| heartbeat | `node_id`, optional `address`, optional `capabilities` | refreshes `last_heartbeat` |

Every other operation is a `PeerPayload` envelope
(`core/peer_payloads.rs:20-31`): `personal-mesh`, `shared-folder`, `drop-box`,
`ask-my-bot`, `peer-backup`, `calls`, `peer-cache`. The body is opaque to the
relay and only the recipient can decrypt it. What a helper can observe per
operation:

| Operation | What a relay sees | What it can retain |
|---|---|---|
| Chat between two directly reachable peers | nothing — no relay hop | nothing |
| Any relayed payload (the seven kinds above) | recipient id, ciphertext, timing and size | the encrypted envelope until `expires_at` |
| Home-site signature check | the signed site and its key id | the site, as any web server would |
| Registration / heartbeat | node id, name, address, capabilities | the row, until the TTL expires |

The relay never receives message text, file contents, contact lists or the
node's signing key.

## Privacy model

- **Opt-in only.** The first-run wizard asks once for a network mode
  (`Offline` or `P2P`); the default is `Offline`. The answer is a hard gate,
  not a preference: it is written to the stack `.env` as `GB_NETWORK_MODE`
  and the node honours it at boot. Settings → Network can flip it at any
  time — `POST /api/peer/disable` stops every peer surface immediately and
  rewrites the same key, so the choice survives a restart. Offline also
  removes Settings → Shared Links, because there is no reachable address to
  share.
- **No retention.** Peer nodes do not retain user content. Payloads are
  end-to-end encrypted and only the recipient can decrypt them.
- **No telemetry.** The mesh carries no analytics, usage reports, or
  crash beacons. See `PRIVACY.md`.
- **Loopback by default.** When P2P is disabled, all networking stays on
  `127.0.0.1`.

## Reachability tiers

Shared Links (Settings → Shared Links) expose three tiers:

1. **LAN** — `hostname.local:<port>`; reachable only on the local network.
2. **Own domain** — requires a configured `PUBLIC_BASE_URL` / `public_base_url`
   and DNS pointing at the host.
3. **Free General Bots address** — opt-in hosted relay (`<name>.generalbots.org`),
   available only after joining P2P.

## Node identity

Each desktop generates a local peer identity on first run: a stable node id
persisted in `node.json` under `data-dir`, plus a signing key kept in the local
Vault (`secret/gbo/peer`, `signing-key`). The node id and the display name are
what registration and heartbeat announce — the signing key is never transmitted.
Heartbeats are sent only while P2P is enabled and only to the configured relay.

## Configuration (Vault)

A deployed node is configured from Vault `secret/gbo/peer`. Precedence, highest
first:

1. process environment (`GB_PEER_*`) — dev, test and one-off overrides
2. Vault `secret/gbo/peer` — the canonical source for a deployed node
3. the legacy `~/.prod` file (only for `GB_PEER_TOKEN`)
4. built-in defaults (and the persisted `mode.json` for the on/off flag)

| Key | Env override | Meaning |
|-----|--------------|---------|
| `enabled` | `GB_PEER_NETWORK` | `1`/`true`/`yes`/`on` joins the mesh |
| `node-id` | `GB_PEER_NODE_ID` | stable UUID; generated and persisted when empty |
| `node-name` | `GB_PEER_NODE_NAME` | display name; `general-bots-<id8>` when empty |
| `advertise-addr` | `GB_PEER_ADVERTISE_ADDR` | address relayed to peers (default `peer.generalbots.org`) |
| `data-dir` | `GB_PEER_DATA_DIR` | where `node.json` and `mode.json` live |
| `heartbeat-ttl-secs` | `GB_PEER_HEARTBEAT_TTL_SECS` | peer expiry, default 90 |
| `token` | `GB_PEER_TOKEN` | shared node token; **required** — with no token every `/api/peer/*` route answers 503 |
| `signing-key` | `GB_PEER_SIGNING_KEY` | home-site signing key; empty falls back to the node id |

`GET /api/peer/status` returns `config_sources`, the origin of each key
(`env`, `vault`, `prod-file`, `mode-file`, `default`), so an operator can prove
a node really reads Vault. It is not anonymous (#2141): call it with the shared
token (`X-Peer-Token`) or an admin session. Token and signing key are never
printed — the boot log reports them as `configured` or `absent`.

### Desktop client (#1969)

The desktop client never discovers a relay. `GET /api/product` publishes a
`peer` object — `{ "relay": "peer.generalbots.org", "relay_url":
"https://peer.generalbots.org", "enabled": <bool>, "node_id": <uuid|null> }` —
and the suite resolves peers and home sites through that host
(`window.GB_PEER_RELAY`). Settings → Network and Settings → Shared Links read
the value from there instead of hardcoding it. The object never carries the
token, the signing key or the node's advertise address.

`advertise-addr` is separate: it is the address this node tells peers to reach,
and it defaults to the same relay host.

### Enabling the public node (`peer.generalbots.org`)

```bash
vault kv put secret/gbo/peer \
  enabled=1 \
  node-name=peer \
  advertise-addr=peer.generalbots.org \
  token="$(openssl rand -hex 32)" \
  signing-key="$(openssl rand -hex 32)"
sudo systemctl restart botserver

curl -s -H "X-Peer-Token: $GB_PEER_TOKEN" \
  http://127.0.0.1:8080/api/peer/status | jq .config_sources
# { "enabled": "vault", "node-name": "vault", "token": "vault", ... }
```

Nothing peer-specific belongs in the systemd unit: only `VAULT_ADDR`,
`VAULT_TOKEN` and the CA path live there, and the values above are read from
Vault at boot.

### Authentication (required, #2141)

There is no anonymous peer call. `GB_PEER_TOKEN` (or Vault `token`) is required
for every route under `/api/peer/`:

| Route | Credential |
|-------|------------|
| `POST /api/peer/register`, `POST /api/peer/heartbeat` | shared token |
| `POST /api/peer/payloads`, `GET /api/peer/payloads` | shared token |
| `GET /api/peer/home-site`, `GET /api/peer/status` | shared token or an admin session |
| `POST /api/peer/enable`, `POST /api/peer/disable` | admin session (normal auth; not in the anonymous list) |

- The token is compared in constant time and accepted as
  `X-Peer-Token: <token>` or `Authorization: Bearer <token>`.
- When no token is configured the machine routes fail closed with
  `503 peer_network_unconfigured`; they never fall back to "local calls are
  allowed". Configure a token before enabling the network.
- `enable`/`disable` are no longer anonymous: they are ordinary admin endpoints
  reached by the authenticated suite, so a stolen token-less request cannot stop
  the network.

### Token rotation

```bash
vault kv put secret/gbo/peer token="$(openssl rand -hex 32)"
sudo systemctl restart botserver   # configuration is resolved once, at boot
```

Rotate `signing-key` the same way; home sites regenerated after the restart carry
the new signature. The token is only checked on peer endpoints, so a restart is
the entire rollout — peers never need the previous value. Rotate the token on
every node that should keep talking: a peer still holding the old value gets
`401 unauthorized` until it is updated.

## Status

The peer-network foundation (identity, heartbeat, status endpoint) ships in
`botserver/src/core/peer_network.rs` and
`botserver/src/main_module/routes/peer_network.rs`. Payloads (personal mesh, shared folder, drop box, ask-my-bot, peer backup,
calls, peer cache) now ship in `botserver/src/core/peer_payloads.rs`, and the
signed home-site generator ships in `botserver/src/core/home_site.rs`
(issue #1867).
