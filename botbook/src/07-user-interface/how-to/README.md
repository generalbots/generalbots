# How To... Tutorials

> **Step-by-step guides for common tasks in the General Bots Suite.**

---

## About These Tutorials

Each tutorial follows the same format:

- **Objective** — what you will accomplish
- **Prerequisites** — what you need before starting
- **Steps** — numbered instructions
- **Troubleshooting** — common problems and solutions
- **Next Steps** — what to read next

---

## Getting Started

| Tutorial | Description |
|----------|-------------|
| [Create Your First Bot](./create-first-bot.md) | Set up a working bot from scratch |
| [Your First Conversation](../../01-getting-started/first-conversation.md) | Talk to your bot and understand responses |

---

## Knowledge Base

| Tutorial | Description |
|----------|-------------|
| [Add Documents to Knowledge Base](./add-kb-documents.md) | Teach your bot from files |

See also [Knowledge Base](../../03-knowledge-ai/knowledge-base.md) for collections, indexing and retrieval modes.

---

## BASIC Dialogs

| Tutorial | Description |
|----------|-------------|
| [Write Your First Dialog](./write-first-dialog.md) | Create a simple conversation script |

For the full picture, continue with [BASIC Scripting](../../04-basic-scripting/README.md), which documents every keyword.

---

## Messaging Channels

| Tutorial | Description |
|----------|-------------|
| [Connect WhatsApp](./connect-whatsapp.md) | Set up the WhatsApp Business integration |

For the other channels, see [Channels](../../06-channels/README.md).

---

## Analytics and Monitoring

| Tutorial | Description |
|----------|-------------|
| [Monitor Live Sessions](./monitor-sessions.md) | Watch conversations in real time |

---

## Keyboard Shortcuts

These are the shortcuts implemented in the Suite client. They are declared in the
frontend sources under `botui/ui/suite/js/`.

| Shortcut | Action | Source |
|----------|--------|--------|
| `Ctrl/⌘ + K` | Open the command palette | `command-palette.js`, `suite_app.js` |
| `Ctrl + L` | Open the control center | `control-center.js` |
| `Ctrl + Space` | Open spotlight search | `spotlight.js` |
| `Ctrl + ↑` | Toggle mission control | `mission-control.js` |
| `Ctrl + Shift + 1…9` | Switch virtual desktop | `virtual-desktops.js` |
| `Ctrl + Shift + ←/→` | Move between virtual desktops | `virtual-desktops.js` |
| `Ctrl + S` | Save (Editor) | `editor.js` |
| `Ctrl + Shift + L` | Toggle auto-filter (Sheets) | `modules-sheet-advanced/01_core.js` |
| `Ctrl + Enter` | Run query (Database) | `database.js` |
| `Alt` | Open the window menu | `suite_app.js`, `base.js` |

Desktop applications also carry their own shortcuts — see
[Suite Manual](../suite-manual.md) for the per-application tables.

---

## Getting Help

- The **Chat** application can answer questions about the Suite itself.
- Configuration and deployment questions are covered in
  [Configuration and Deployment](../../10-configuration-deployment/README.md).
- Bugs and feature requests are covered by
  [Instructions for Logging Issues](../../12-ecosystem-reference/contributing-guidelines.md).

---

## Tips for Following Tutorials

### Before you start

1. Keep the Suite open and follow along step by step.
2. Read an entire step before performing the action.

### If something goes wrong

1. Re-read the step — the value must match exactly before you customise it.
2. Check the **Troubleshooting** section at the end of each tutorial.
3. Ask the **Chat** application for assistance.

---

## Marker Legend

Throughout the book you will see these indicators:

| Marker | Meaning |
|--------|---------|
| 🟢 | **GA** — generally available |
| 🟡 | **PREVIEW** — usable, still changing |
| ℹ️ | **Note** — additional context |
| ⚠️ | **Warning** — a caution worth reading |

---

**Start with [Create Your First Bot](./create-first-bot.md).**
