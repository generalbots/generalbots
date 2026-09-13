# Docs 🟡 PREVIEW — IN TEST

> **In test.** Docs is a preview application and is **not part of the supported surface**. It is functional and actively being tested, but behaviour and interface may change, and edge cases are not guaranteed. Decide accordingly before depending on it.

Docs is the suite's collaborative document editor: rich text, comments, tracked changes and citation support, with AI assistance.

## What the shipped editor exposes

Verified from the app's own toolbar and module layout (`botui/ui/suite/docs/`).

### Editing

| Capability | Notes |
|---|---|
| Formatting | Rich text with headings, quotes, lists, inline code and links |
| Find & Replace | Search within the document |
| Undo / Redo | Standard history |
| **Track Changes** | Record edits as suggestions rather than applying them silently |
| **Compare Documents** | Diff two documents against each other |
| **Version history** | Browse and restore earlier states |
| Activity log | What happened to the document, and when |

### Structure and content

| Capability | Notes |
|---|---|
| Table of Contents | Generated navigation for the document |
| Header & Footer | Page furniture |
| Insert Table / Image | Structured and visual content |
| Insert Equation | Mathematical notation |
| Insert Footnote / Endnote | Scholarly referencing |
| Insert Page Break | Page control |
| From Template | Start from an existing template |

### Collaboration

| Capability | Notes |
|---|---|
| Collaborators & follow | See who is in the document and follow their cursor |
| Comments | Discussions attached to a position in the text |
| Presence | Live indication of who is editing where |

### References and AI

| Capability | Notes |
|---|---|
| Citations & References | Source management inside the document |
| AI Assistant | Ask for edits and improvements in natural language |

Localisation ships with `en` and `pt-BR`.

## What changed in this page

Earlier revisions listed a precise feature matrix — font size ranges, alignment options, a fixed set of formatting buttons — presented as finished behaviour. Those specifics could not be verified against the shipped editor, so they have been replaced with the capability list above, which is taken from the actual toolbar and module layout.

## Opening it

Docs is a **preview** application and is additionally gated behind Preview mode as an in-test app. Turn on the **Preview** switch in the left sidebar, then open **Docs** from the app menu.

## When to use something else

| Need | Use |
|---|---|
| A supported rich document, today | [Paper](./paper.md) — also preview, but the simpler authoring surface |
| Tables of data rather than prose | [Sheets](./sheet.md), the most advanced preview app |
| Presentations | [Slides](./slides.md) — also in test |

## See Also

- [Paper](./paper.md) - Notes and documents
- [Slides](./slides.md) - Presentations (in test)
- [Explorer](./drive.md) - Where documents are stored
- [Apps overview](./README.md) - Stability classification for the whole suite
