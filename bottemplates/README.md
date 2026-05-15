# General Bots Templates

Pre-built bot packages for common business use cases. Templates are organized by type for easy discovery.

---

## Structure Overview

- **`bots/`**: Bot definition packages (.gbai) organized by category.
- **`apps/`**: Source code templates for various languages (Rust, Python, Node.js).
- **`sites/`**: Simple HTMX-based web site templates.

---

## 🤖 Bots (by Category)

### `/core`
Platform administration and core references.
| Template | Description |
|----------|-------------|
| `default.gbai` | Starter template |
| `template.gbai` | Reference for new templates |
| `ai-search.gbai` | Document search (RAG, Semantic) |
| `analytics-dashboard.gbai` | Platform analytics bot |

### `/ai`
Multiagent workflows and LLM examples.
| Template | Description |
|----------|-------------|
| `llm-server.gbai` | Model hosting |
| `customer-support-workflow.gbai` | Support workflow automation |

### `/communications`
Messaging and office automation tools.
| Template | Description |
|----------|-------------|
| `whatsapp.gbai` | WhatsApp Business |
| `broadcast.gbai` | Message broadcasting |

### `/sales`
Customer relationship and e-commerce.
| Template | Description |
|----------|-------------|
| `crm.gbai` | Full CRM system |
| `store.gbai` | E-commerce product catalog |

*(See individual folders for the complete list of 45+ templates)*

---

## 📱 Apps (Language Starters)

| Folder | Description | Tech Stack |
|--------|-------------|------------|
| `rust-starter` | Base Rust template | Tokio, Anyhow |
| `python-starter` | Base Python template | FastAPI, Uvicorn |
| `node-starter` | Base Node.js template | Express |

Each app template includes an `AGENTS.md` with LLM directives specific to that language.

---

## 🌐 Sites (HTMX Based)

| Folder | Description | Tech Stack |
|--------|-------------|------------|
| `htmx-starter` | Full HTMX application base | HTMX, Minimal CSS |

---

## Installation

### From Console

```bash
botserver --install-template bots/sales/crm
```

### Manual

Copy the template folder to your bot's packages directory:

```bash
cp -r templates/bots/sales/crm.gbai /path/to/your/bot/packages/
```

---

## License

All templates are licensed under AGPL-3.0 as part of General Bots.

---

**Pragmatismo** - General Bots Open Source Platform
