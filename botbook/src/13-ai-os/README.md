# AI OS 🟡 BETA

The platform is evolving from a conversational bot server into an autonomous
personal operating system. This chapter documents the AI OS capability set:
always-on agent workspaces, the tabbed suite shell, adaptive scheduled agents,
the skills marketplace, durable memory, real-time voice, consented desktop
automation, the hardened browsing control plane and enterprise connectors.

Every capability ships behind a feature flag and preserves legacy behavior
when disabled. Backend entry points are additive namespaces (`/api/agent/*`,
`/api/automations/*`, `/api/marketplace/*`, `/api/memory/*`, `/api/browser/*`,
`/api/consent/*`, `/api/connectors/*`, `/api/v1/sandbox`).

## In this chapter

- [Agent Mode & Snapshots](./agent-mode.md)
- [Workspace Tabs](./workspace-tabs.md)
- [Adaptive Automations](./automations.md)
- [Skills Marketplace](./marketplace.md)
- [Memory OS](./memory-os.md)
- [Consent System](./consent.md)

## Feature flags (botserver Cargo features)

| Feature | Crate | Surface |
|---|---|---|
| `agent-vm` | `botagent` | per-session VMs, snapshots, sandbox exec, org API keys |
| `automations` | `botautomation` | NL schedules, run engine, delivery loop, dashboard API |
| `marketplace` | `botmarketplace` | public catalog, publish/install |
| `consent` | `botconsent` | permission matrix, prompt cards, audit |
| `memory-os` | `botmemory` | recall injection, extraction hook, memory API |
| `connectors` | `botconnectors` | third-party ingestion with ACL filtering |
| `browser-policy` | `botbrowserpolicy` | browsing policy, budgets, page facts |
| `channel-bindings` | `botchannelbindings` | default number/domain bindings per bot |

Model routing (multi-profile fallback, circuit breakers, council mode) lives in
the `botllm` crate and activates when `GB_LLM_PROFILES` (env or JSON file) is
present; single-provider deployments continue to work unchanged.
