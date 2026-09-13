# What's New 🟡 BETA

What changed recently, with the date it landed. Every entry here is traceable to a
commit, a route or a page in the repository — this page is a changelog, not a
feature list. For what exists today rather than what is new, see the
[Roadmap](../ROADMAP.md) and [Suite Apps Status](../07-user-interface/apps/suite-apps-status.md).

> **Verified 2026-09-13** against the commit history. If an entry here disagrees
> with the code, the code is right.

## September 2026

| Date | Change | Where it lands |
|---|---|---|
| 2026-09-13 | **Applications in the menu can be launched again.** Nine catalog apps were unreachable behind a launch gate, which was removed | Launcher; [Suite Apps Status](../07-user-interface/apps/suite-apps-status.md) |
| 2026-09-13 | **Platform subdomains resolve to their bot**, and the proxy can ask for a certificate during the TLS handshake (`/api/domains/tls-ask`) | [Domain management](../12-ecosystem-reference/README.md) |
| 2026-09-13 | **The cloud sidebar shows the signed-in identity** — `/api/auth/me` resolves cloud JWTs rather than only chat tokens | Cloud console |
| 2026-09-13 | **Vibe website projects run on the proxy container**, not a development VM, and an unknown project type is rejected with an explicit error instead of failing quietly | [Vibe](../07-user-interface/apps/vibe.md) |
| 2026-09-13 | **Signup identities get a local user row and a working password** — account creation now imports the user through the directory instead of writing a row the directory did not know about | [Security](../09-security/README.md) |
| 2026-09-13 | **The documentation site is `docs.generalbots.org`**, replacing the older domain | This book |

## Earlier in 2026

The larger arcs — the Rust core, channels, Drive, the productivity suite, workflow
and CRM — are recorded with their state and evidence in the
[Roadmap](../ROADMAP.md), which separates **shipped** work from work that is
in progress or planned. That separation is deliberate: this page should never be
the place where unfinished work is announced as finished.

## See Also

- [Roadmap](../ROADMAP.md) — what is shipped, in progress and planned
- [Multi-Agent Orchestration](./multi-agent-orchestration.md) — agents, protocols and routing
- [Retrieval and RAG](./hybrid-search.md) — how knowledge is retrieved, and what is not implemented
- [Suite Apps Status](../07-user-interface/apps/suite-apps-status.md) — per-application state
