# OpenClaw and General Bots 🟡 BETA

OpenClaw is the closest thing in the market to what General Bots is trying to do for individuals: an open-source personal agent that runs on your own machine and meets you in the chat applications you already use. This page explains where the two overlap and where they genuinely differ.

> **Verified 2026-09-13.** Sources: `openclaw.ai`, `github.com/openclaw/openclaw`, and the creator's announcement at `steipete.me/posts/2026/openclaw` (2026-02-14). Re-check before quoting pricing or version numbers.

## What OpenClaw is

| Aspect | Detail |
|---|---|
| What it is | An open-source personal AI assistant that runs on your own computer |
| Reach | Meets you in WhatsApp, Telegram, Discord, iMessage, Slack, Teams and other chat apps |
| Capability | Works with your inbox, email, calendar and local files; runs real-world tasks through skills and plugins |
| Models | Bring your own — a provider key, a local model, or an existing chat subscription |
| Install | `npm i -g openclaw` or the install script; desktop apps for macOS, Windows and Linux |
| Created by | Peter Steinberger |
| Status (2026) | OpenAI acquired the project in February 2026; the creator joined OpenAI to lead personal agents. OpenClaw is moving to a **foundation** to stay open and independent, with OpenAI sponsoring it. |

OpenClaw's growth has been extraordinary — it became one of the most-starred repositories on GitHub within months of release. That momentum, and the depth of its personal-agent UX, is real and worth respecting.

## Where they overlap

- **Local-first and self-hosted.** Both run on infrastructure you control, and both keep your data on it.
- **Bring your own model.** Neither locks you to a single LLM provider.
- **Multi-channel.** Both treat chat applications as first-class surfaces rather than a web widget afterthought.
- **Open source.** Both are open codebases you can modify.

## Where they differ

| Aspect | OpenClaw | General Bots |
|--------|----------|--------------|
| Primary user | An individual with their own machine | A team or organisation, on a server |
| Multi-tenancy | Single-user focus | Organisations, branches, workspaces and per-bot scoping |
| Productivity suite | Acts on your existing inbox, calendar and files | Ships its own mail, calendar, files, tasks, meetings and office apps |
| Channel coverage | WhatsApp, Telegram, Discord, iMessage, Slack, Teams, SMS via provider | Web chat, WhatsApp, Teams, Slack, Telegram, SMS, email and voice |
| Configuration model | Skills, plugins and prompts in TypeScript | BASIC scripting, plus conversational configuration |
| Governance | Device-level; personal use | RBAC via Zitadel, audit logging, per-app permissions, admin console |
| Desktop | Full desktop apps for macOS, Windows, Linux | Suite in the browser, plus an optional desktop wrapper |
| Maturity | Extremely widely deployed for a personal agent | Early preview; stable surface is Chat, Explorer and Vibe |

## When OpenClaw is the better fit

- You want a personal assistant for **your own** devices and accounts.
- You want to reach it from inside the messaging apps you already live in.
- You prefer configuring behaviour by writing skills and plugins rather than a scripting language.
- You need something running in minutes on a laptop, not a deployment for a team.

## When General Bots is the better fit

- You need **many users** with different roles, permissions and isolated data.
- You want the assistant and the productivity suite in one system, rather than an agent acting on tools hosted elsewhere.
- You need server-side deployment with audit logs, RBAC and an admin console.
- You want automation expressed in a language that non-developers on the team can read.

## Summary

OpenClaw and General Bots share a philosophy — own your data, choose your model, meet users where they are — but they aim at different scales. OpenClaw is an exceptional personal agent. General Bots is an organisational platform that includes an agent. Choosing between them is mostly a question of whether you are deploying for **one person** or **an organisation**.

## See Also

- [Platform Comparison Matrix](./comparison-matrix.md) — full capability-by-capability comparison
- [OpenAI Frontier](./openai-frontier.md) — the enterprise agent platform from the same vendor
- [Channels](../06-channels/channels.md) — how General Bots connects to messaging platforms
