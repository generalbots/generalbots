# Peer Network (P2P Mesh)

General Bots can run as a peer in an opt-in, privacy-preserving mesh. Every
desktop install is a potential node; joining is always explicit and can be
stopped at any time.

## Privacy model

- **Opt-in only.** The first-run wizard asks for a network mode
  (`Offline` or `P2P`). The default is `Offline`. A master peer switch in
  Settings → Shared Links can stop all P2P traffic.
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

Each desktop generates a local peer identity on first run. The identity is
stored in the local Vault and never transmitted in plaintext. Heartbeats are
sent only while P2P is enabled and only to the configured relay.

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
| `token` | `GB_PEER_TOKEN` | shared bearer token for machine callers; empty disables the check |
| `signing-key` | `GB_PEER_SIGNING_KEY` | home-site signing key; empty falls back to the node id |

`GET /api/peer/status` returns `config_sources`, the origin of each key
(`env`, `vault`, `prod-file`, `mode-file`, `default`), so an operator can prove
a node really reads Vault. Token and signing key are never printed — the boot
log reports them as `configured` or `absent`.

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

curl -s http://127.0.0.1:8080/api/peer/status | jq .config_sources
# { "enabled": "vault", "node-name": "vault", "token": "vault", ... }
```

Nothing peer-specific belongs in the systemd unit: only `VAULT_ADDR`,
`VAULT_TOKEN` and the CA path live there, and the values above are read from
Vault at boot.

### Token rotation

```bash
vault kv put secret/gbo/peer token="$(openssl rand -hex 32)"
sudo systemctl restart botserver   # configuration is resolved once, at boot
```

Rotate `signing-key` the same way; home sites regenerated after the restart carry
the new signature. The token is only checked on peer endpoints, so a restart is
the entire rollout — peers never need the previous value.

## Status

The peer-network foundation (identity, heartbeat, status endpoint) ships in
`botserver/src/core/peer_network.rs` and
`botserver/src/main_module/routes/peer_network.rs`. Payloads (personal mesh, shared folder, drop box, ask-my-bot, peer backup,
calls, peer cache) now ship in `botserver/src/core/peer_payloads.rs`, and the
signed home-site generator ships in `botserver/src/core/home_site.rs`
(issue #1867).
