# General Bots Roadmap 2018-2026

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
  View Interactive Roadmap
</button>

<div class="roadmap-overlay" id="roadmap-overlay" onclick="closeRoadmap(event)">
  <div class="roadmap-popup" onclick="event.stopPropagation()">
    <div class="roadmap-popup-header">
      <h3 class="roadmap-popup-title">General Bots Roadmap 2018-2026</h3>
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

## Timeline Overview

| Period | Focus | Features | Key Deliverables |
|--------|-------|----------|------------------|
| **2018-2024** | v1-v5 Pre-LLM | 12 ✅ | Package System, TALK/HEAR, NLP/BERT, GPT-3.5, QR CODE, SET SCHEDULE |
| **2024** | v6 Foundation | 8 ✅ | Rust Core, Migration v5→v6, New Architecture, PostgreSQL, Vault, Minimal Flow |
| **2025 H1** | Rust Migration | 10 ✅ | BASIC Engine, Channels, Drive (S3), Email, REST API, WhatsApp, Telegram, PDF |
| **2025 H2** | Features & AUTOTASK | 19 ✅ | Tasks AI, KB, Vector DB, Tools, Generators, Multimodal, NVIDIA GPU/LXC, Paper, Research, Calendar, Meet |
| **2026 Q1** | Tasks AI GO ⭐ | 12 📋 | Production Autonomous, Gmail, Outlook, Google Drive, OneDrive, Google/Outlook Calendars, Transfer to Human |
| **2026 Q2** | Collaboration | 10 📋 | Multi-Agent, Teams, Google Meet, Zoom, Slack, Discord, Docker, Compliance, Marketplace |
| **2026 Q3** | Workflow & CRM | 2 📋 | Workflow Designer, CRM Integration |
| **2026 Q4** | Enterprise | 4 📋 | Mobile Apps, Enterprise SSO, White Label, Advanced Monitoring |

**Total: 77 Features** (49 Complete ✅ • 28 Planned 📋)

---

## Feature Highlights

### Tasks (AI Autonomous) GO

The flagship feature enabling fully autonomous AI task execution:

- Human provides intent in natural language
- AI creates execution plan
- AI generates code/content
- AI deploys result
- Human reviews and approves

**Available in:** 2025 H2 (scaffolding), Q1 2026 (production)

### Generators

| Generator | Purpose |
|-----------|---------|
| BOT | Conversational bots |
| APP | Full applications |
| SITE | HTMX websites |
| GENERAL | General content |
| LANDPAGE | Landing pages |

---

## Technology Stack

**Backend:** Rust, Actix-Web, Tokio, SQLx  
**Database:** PostgreSQL  
**Storage:** MinIO (S3-compatible)  
**Cache:** Valkey (Redis alternative)  
**UI:** HTMX, Askama templates  
**Desktop/Mobile:** Tauri  

---

## Status Legend

| Status | Description |
|--------|-------------|
| Complete | Feature is fully implemented and available |
| In Progress | Feature is currently being developed |
| Planned | Feature is scheduled for future development |

Click the **View Interactive Roadmap** button above to explore all 77 features with detailed descriptions. Scroll horizontally to navigate the timeline from 2018 to 2026.
