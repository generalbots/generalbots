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

## Localization (i18n)

Templates ship **one language per template** — a default template must never mix
Portuguese and English in the same `start.bas` (suggestions, `TALK` body and
system prompt all in the same language). The English pack under `bots/` is the
source language; the Brazil-specific demos (`food`, `taxi`, `doctor`, …) ship in
pt-BR.

Every shipped prompt carries the **response-language contract** so a bot never
answers in a language the user did not write:

```
Always answer in the user's language. If the user writes in Portuguese, answer
in Portuguese; if Japanese, answer in Japanese. Keep code, commands,
identifiers and file paths unchanged.
```

Prompts are resolved per channel by the runtime, in this order
(`load_system_prompt_for_channel`):

1. `PROMPT-<CHANNEL>.md` (e.g. `PROMPT-WEB.md`, `PROMPT-WHATSAPP.md`)
2. `PROMPT.md` / `prompt.md` / `PROMPT.txt` / `prompt.txt`
3. The generic assistant fallback

The loader appends the contract to whatever it resolved (and skips it when the
authored prompt already states it), so a custom bot that never mentions language
still answers in the language the user wrote.

A localized variant is a **new file next to the base one** — never an edit of the
base prompt: `PROMPT-<CHANNEL>.md` stays English (source), and a translated copy
is named `PROMPT-<CHANNEL>.<locale>.md` (`PROMPT-WEB.pt-BR.md`). The same rule
applies to content: `start.<locale>.bas` next to `start.bas`. Today the deploy
endpoint copies the template tree as-is and the runtime reads only the base
names, so a variant file is inert until the locale-aware selection lands; ship
it alongside the base file so the copy is already in place when it does.

Suggestion and switcher labels are **catalogue keys**, not literals: the
runtime resolves `ADD SUGGESTION "scan" AS "bot-suggest-scan-a-qr-code"` for the
session locale (session `context_data.locale` → bot `locale` configuration →
`en`) through `botlib/locales/<locale>/bot-templates.ftl`. A string the
catalogue does not define is kept exactly as authored, so a custom bot keeps its
own wording. `bottemplates/**/start.bas` therefore carries `bot-suggest-*` keys
(en / pt-BR / es ship translations today; other locales fall back to en), and
the same template serves every locale. `TALK` bodies remain authored in the
template's language — translating business content is out of scope for #2095.

---

## License

All templates are licensed under MIT as part of General Bots.

---

**Pragmatismo** - General Bots Open Source Platform
