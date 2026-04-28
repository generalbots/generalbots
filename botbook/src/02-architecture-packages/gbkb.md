# .gbkb Knowledge Base

The `.gbkb` package contains your bot's domain knowledge - documents that the AI uses to answer questions accurately about your specific organization, products, or services.

## What It Does

When you place documents in a `.gbkb` folder, the system automatically:

1. **Extracts text** from your files (PDF, DOCX, XLSX, PPTX, EPUB, TXT, MD, HTML, CSV, JSON, YAML, TOML, and more)
2. **Creates searchable indexes** using vector embeddings
3. **Enables semantic search** so users can ask questions naturally

This means your bot answers based on YOUR documents, not just general AI knowledge.

## Folder Structure

```
mybot.gbai/
└── mybot.gbkb/
    ├── policies/           ← Collection: "policies"
    │   ├── vacation.pdf
    │   └── handbook.docx
    ├── products/           ← Collection: "products"
    │   ├── catalog.pdf
    │   └── specs.xlsx
    └── support/            ← Collection: "support"
        └── faq.md
```

Each subfolder becomes a **collection** you can activate independently.

## Using in BASIC Scripts

```basic
' Activate collections for this conversation
USE KB "policies"
USE KB "products"

' Now the AI automatically searches these when answering
TALK "How can I help you today?"

' Later, clear when done
CLEAR KB "policies"
```

## Supported File Types

| Category | Formats | Extensions |
|----------|---------|------------|
| **Documents** | PDF, Word | `.pdf`, `.docx`, `.doc` |
| **Spreadsheets** | Excel, OpenDocument | `.xlsx`, `.xls`, `.ods` |
| **Presentations** | PowerPoint, OpenDocument | `.pptx`, `.ppt`, `.odp` |
| **E-books** | EPUB, OpenDocument Text | `.epub`, `.odt` |
| **Text** | Plain, Markdown, reStructuredText, AsciiDoc | `.txt`, `.md`, `.rst`, `.adoc` |
| **Web** | HTML | `.html` |
| **Data** | CSV, JSON, JSONL, TSV | `.csv`, `.json`, `.jsonl`, `.tsv` |
| **Config** | YAML, TOML, INI, Properties | `.yaml`, `.yml`, `.toml`, `.ini`, `.conf`, `.cfg`, `.env`, `.properties` |
| **Code** | Python, Rust, JS/TS, Shell, SQL, GraphQL, Proto | `.py`, `.rs`, `.js`, `.ts`, `.sh`, `.sql`, `.graphql`, `.proto` |
| **Style** | CSS, SVG | `.css`, `.svg` |
| **Calendar** | iCalendar, vCard, Email | `.ics`, `.vcf`, `.eml` |
| **Logs** | Log files | `.log` |

## Key Points

- **Automatic indexing** - Just drop files in folders
- **Semantic search** - Users don't need exact keywords
- **Multiple collections** - Organize by topic, activate as needed
- **No code required** - The AI handles search automatically

## Learn More

- [Chapter 03: Knowledge Base System](../03-knowledge-ai/README.md) - Technical deep-dive on indexing, vectors, and search
- [USE KB Keyword](../04-basic-scripting/keyword-use-kb.md) - Complete keyword reference
- [CLEAR KB Keyword](../04-basic-scripting/keyword-clear-kb.md) - Managing active collections