# General Bots Roadmap 2018-2027 🟡 BETA

<style>
.roadmap-btn {
  display: inline-flex;
  align-items: center;
  gap: 8px;
  padding: 12px 24px;
  background: linear-gradient(135deg, #6366F1, #8B5CF6);
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: transform 0.2s, box-shadow 0.2s;
}
.roadmap-btn:hover {
  transform: translateY(-2px);
  box-shadow: 0 8px 20px rgba(99, 102, 241, 0.3);
}
.roadmap-overlay {
  display: none;
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.7);
  z-index: 9999;
  justify-content: center;
  align-items: center;
}
.roadmap-overlay.active {
  display: flex;
}
.roadmap-popup {
  background: #FFFFFF;
  border-radius: 16px;
  width: 95vw;
  height: 90vh;
  max-width: 1800px;
  overflow: hidden;
  box-shadow: 0 25px 50px -12px rgba(0, 0, 0, 0.5);
  display: flex;
  flex-direction: column;
}
@media (prefers-color-scheme: dark) {
  .roadmap-popup {
    background: #1E293B;
  }
}
.roadmap-popup-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 16px 24px;
  border-bottom: 1px solid #E2E8F0;
}
@media (prefers-color-scheme: dark) {
  .roadmap-popup-header {
    border-bottom-color: #334155;
  }
}
.roadmap-popup-title {
  font-size: 1.25rem;
  font-weight: 700;
  color: #1E1B4B;
  margin: 0;
}
@media (prefers-color-scheme: dark) {
  .roadmap-popup-title {
    color: #F1F5F9;
  }
}
.roadmap-popup-close {
  background: #F1F5F9;
  border: none;
  width: 36px;
  height: 36px;
  border-radius: 8px;
  font-size: 20px;
  cursor: pointer;
  color: #64748B;
  display: flex;
  align-items: center;
  justify-content: center;
}
.roadmap-popup-close:hover {
  background: #E2E8F0;
  color: #334155;
}
@media (prefers-color-scheme: dark) {
  .roadmap-popup-close {
    background: #334155;
    color: #94A3B8;
  }
  .roadmap-popup-close:hover {
    background: #475569;
    color: #E2E8F0;
  }
}
.roadmap-iframe {
  flex: 1;
  width: 100%;
  border: none;
}
</style>

<button class="roadmap-btn" onclick="openRoadmap()">
  View Interactive Roadmap (historical plan, 2018-2026)
</button>

<div class="roadmap-overlay" id="roadmap-overlay" onclick="closeRoadmap(event)">
  <div class="roadmap-popup" onclick="event.stopPropagation()">
    <div class="roadmap-popup-header">
      <h3 class="roadmap-popup-title">General Bots Roadmap 2018-2026 — historical plan</h3>
      <button class="roadmap-popup-close" onclick="closeRoadmap()">X</button>
    </div>
    <iframe class="roadmap-iframe" src="assets/roadmap.html" title="Roadmap"></iframe>
  </div>
</div>

<script>
function openRoadmap() {
  document.getElementById('roadmap-overlay').classList.add('active');
  document.body.style.overflow = 'hidden';
}
function closeRoadmap(event) {
  if (!event || event.target.id === 'roadmap-overlay') {
    document.getElementById('roadmap-overlay').classList.remove('active');
    document.body.style.overflow = '';
  }
}
document.addEventListener('keydown', function(e) {
  if (e.key === 'Escape') closeRoadmap();
});
</script>

---

> **Last verified September 2026.** This page states what has shipped, what is in progress and what is planned. Every "shipped" entry is traceable to code in this repository. The interactive view above is the **2018–2026 planning artifact** and is not maintained as a status source — this page is.

## Current release state

| Surface | Status |
|---|---|
| **Chat**, **Explorer** (Drive), **Vibe** | Stable |
| **Mail**, **Sheets** | Preview — advanced |
| **Docs**, **Slides** | Preview — in test |
| Everything else in the catalog | Preview |

Per-application state, with counts generated from the catalog, is in
[Suite Apps Status](./07-user-interface/apps/suite-apps-status.md).

## Timeline

"Delivered" counts items actually present in the code today, not items that were scoped.

| Period | Focus | Delivered | State |
|--------|-------|-----------|-------|
| **2018–2024** | v1–v5, pre-LLM | 12 | ✅ Shipped |
| **2024** | v6 foundation — Rust core, PostgreSQL, Vault | 8 | ✅ Shipped |
| **2025 H1** | Rust migration — BASIC engine, channels, Drive (S3), email, REST, WhatsApp, Telegram, PDF | 10 | ✅ Shipped |
| **2025 H2** | Features and Autotask — tasks AI, knowledge base, vector search, tools, generators, multimodal, NVIDIA GPU/LXC, Paper, Research, Calendar, Meet | 19 | ✅ Shipped |
| **2026 Q1** | Autonomous tasks, cloud productivity connections | see below | 🟡 Partially shipped |
| **2026 Q2** | Collaboration and multi-agent | see below | 🟡 Partially shipped |
| **2026 Q3** | Workflow and CRM — Designer, CRM | 2 | ✅ Shipped |
| **2026 Q4** | Enterprise — mobile apps, enterprise SSO, white label, advanced monitoring | — | 🔵 In progress |
| **2027 H1** | Retrieval quality — evaluation harness, re-ranking, chunking | — | 📋 Planned |
| **2027 H2** | Retrieval depth — graph index, multi-hop verification | — | 📋 Planned |

## 2026 detail — what is actually there

| Item | State | Evidence |
|------|-------|----------|
| Multi-agent orchestration | ✅ Shipped | `botserver/src/core/bot/` agent pipeline, [docs](./03-knowledge-ai/multi-agent-orchestration.md) |
| Cloud productivity connections (Google, Outlook, OneDrive, Google Calendar) | ✅ Shipped as integration providers | `botserver/crates/botintegrations/src/providers/` |
| Messaging providers (Slack, Discord, Teams, Zoom) | ✅ Shipped as integration providers | same provider catalog |
| White label | ✅ Shipped | the `.product` configuration surface, [docs](./12-ecosystem-reference/README.md) |
| Enterprise SSO | ✅ Shipped | Zitadel-backed identity, `botcoredirectory` |
| Advanced monitoring | ✅ Shipped | Monitoring app, `botmonitoring` |
| App marketplace | ✅ Shipped | `botmarketplace`, App Store app |
| Workflow designer | ✅ Shipped | Designer app |
| CRM | ✅ Shipped | CRM, People, Sales and Campaigns apps |
| Mobile apps | 🔵 In progress | Tauri shell exists for desktop; mobile not verified |
| Fully autonomous task execution | ⚠️ Partially | Autotask runs, but "production autonomous" as originally scoped is not verifiable as complete — treat as in progress |

## 2027 horizon — planned

Ordered by what unlocks the most, not by what is most visible:

1. **Retrieval evaluation harness.** Without a golden set and a measured metric, every other retrieval decision is guesswork. This blocks the rest — see [#1379](https://github.com/generalbots/generalbots/issues/1379).
2. **Cross-encoder re-ranking.** The cheapest quality win available; the configuration surface already exists in an unconnected crate.
3. **Real chunking with overlap.** Documents are currently indexed whole, which costs precision on long documents.
4. **Graph retrieval with an actual graph index**, replacing entity expansion with traversal.
5. **Latency and cost budgets per retrieval mode**, so mode selection stops being a guess.

## Technology stack — verified

| Layer | Technology |
|-------|------------|
| Language | Rust |
| HTTP | **Axum** (with `axum-server`) |
| Async runtime | Tokio |
| Database | PostgreSQL with **Diesel** and `diesel_migrations` |
| Cache | Valkey |
| Object storage | MinIO |
| Vector search | Qdrant |
| Identity | Zitadel |
| UI | HTMX with Askama templates |
| Desktop | Tauri |

> **Correction.** Earlier revisions of this page listed **Actix-Web** and **SQLx**. Both were wrong: the repository contains no Actix dependency and no SQLx usage — the HTTP stack is Axum and the data layer is Diesel.

## Status legend

| Status | Meaning |
|--------|---------|
| ✅ Shipped | Present in the code and reachable by a user today |
| 🟡 Partially shipped | Some of the scope works; the rest does not |
| 🔵 In progress | Active work, not yet reachable end to end |
| 📋 Planned | Scoped, not started |
| ⚠️ Not verifiable | Claimed previously without evidence — do not rely on it |

## How this page is maintained

- A row only says "shipped" if someone traced it to code.
- Counts that can be derived from the repository are generated by scripts in `scripts/`, not typed by hand.
- When a plan changes, the old state is corrected rather than deleted, so the record stays honest.
