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

## Status

The peer-network foundation (identity, heartbeat, status endpoint) ships in
`botserver/src/core/peer_network.rs` and
`botserver/src/main_module/routes/peer_network.rs`. Payloads (personal mesh, shared folder, drop box, ask-my-bot, peer backup,
calls, peer cache) now ship in `botserver/src/core/peer_payloads.rs`, and the
signed home-site generator ships in `botserver/src/core/home_site.rs`
(issue #1867).
