# OpenAI Frontier and General Bots 🟡 BETA

OpenAI Frontier is the enterprise platform OpenAI launched in February 2026 to build, deploy and govern AI agents across business systems. It occupies the enterprise end of the agent market, which is the same ground General Bots targets — from a very different starting point.

> **Verified 2026-09-13.** Sources: OpenAI's Frontier launch material (`openai.com/business/frontier`) and published coverage of the launch. Where a capability is not publicly documented, this page says so rather than guessing. Re-check before quoting.

## What OpenAI Frontier is

| Aspect | Detail |
|---|---|
| What it is | An enterprise platform for building, deploying and managing AI agents across business systems |
| Launched | 5 February 2026 |
| Core concepts | **Business Context** (a shared institutional-memory layer over data warehouses, CRM, internal apps and documents), **Agent Execution** (parallel, multi-step tasks with escalation to humans), **Open Platform** (manages agents built outside OpenAI too) |
| Governance | Explicit permissions, comprehensive auditing, and auditable actions for every agent interaction |
| Model policy | Not locked to OpenAI models; multi-vendor agent management is a deliberate design goal |
| Management model | Treats agents like employees: onboarding, feedback loops and improvement cycles |
| Availability | Enterprise contracts; not self-serve signup |
| Launch customers | HP, Intuit, Oracle, State Farm, Thermo Fisher and Uber, with BBVA, Cisco and T-Mobile in pilots |

Frontier is explicitly **not** a chat interface (that is ChatGPT), **not** a low-code builder, and **not** something you sign up for yourself. It is a management and governance layer.

## Where they overlap

- **Agent governance as a product.** Both treat permissions, auditing and accountability as core features rather than afterthoughts.
- **Multi-step agent execution.** Both run agents across systems rather than answering single questions.
- **Multi-vendor models.** Neither insists you use one model provider.
- **Systems of record.** Both connect to the CRM, databases and document stores you already run.

## Where they differ

| Aspect | OpenAI Frontier | General Bots |
|--------|-----------------|--------------|
| Deployment | Managed cloud, enterprise contract | Self-hosted on your own infrastructure |
| Licensing | Proprietary, enterprise agreement | MIT open source |
| Data location | OpenAI-hosted | Your servers; you choose residency |
| Self-serve | No — enterprise onboarding | Yes — install and run it yourself |
| Scope | Agent management layer | Agents plus a full productivity suite (mail, calendar, files, tasks, meetings, office apps) |
| Interface | Administration and governance surfaces | Web chat, messaging channels, voice and a desktop suite |
| Configuration | APIs and SDKs | BASIC scripting, plus conversational configuration |
| Cost model | Enterprise contract | Infrastructure plus your own LLM spend |
| Maturity | Backed by a major vendor, Fortune 500 deployments | Early preview; stable surface is Chat, Explorer and Vibe |

## When Frontier is the better fit

- You are a large enterprise that needs a **vendor-supported** agent management layer with contractual governance.
- Your agents must be centrally governed across many business units and systems of record.
- You want to manage agents built on several vendors' models from one control plane.
- You have no appetite to run infrastructure yourself.

## When General Bots is the better fit

- You need **data sovereignty** — the agents and the data they touch must stay on your own infrastructure.
- You want open source you can read, modify and audit, with no per-user licensing.
- You want the agent and the daily productivity tools in one deployable system, not an agent layer bolted onto tools hosted elsewhere.
- You need channels your customers actually use — WhatsApp, Telegram, SMS, email and voice — as first-class surfaces.

## An honest note on scale

Frontier launched with Fortune 500 customers and a major vendor behind it. General Bots is an early-preview project maintained by a small team. If you need enterprise support contracts today, Frontier is the safer choice; if you need to own the stack, General Bots is the one that lets you.

## Summary

Frontier answers "how does a large enterprise govern many agents across many systems?" General Bots answers "how does an organisation own its assistant and its data end to end?" Both are legitimate questions; they are not the same question.

## See Also

- [Platform Comparison Matrix](./comparison-matrix.md) — full capability-by-capability comparison
- [OpenClaw and General Bots](./openclaw.md) — the personal-agent counterpoint
- [Security Policy](../09-security/security-policy.md) — how General Bots handles governance
