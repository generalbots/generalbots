# Knowledge Base 🟡 BETA

The Knowledge Base (KB) system enables semantic search and document retrieval for intelligent bot responses.

## Quick Overview

| Feature | Description |
|---------|-------------|
| **Storage** | S3-compatible drive + PostgreSQL metadata + Qdrant vectors |
| **Search** | Dense vector search, optionally fused with keyword matching — chosen per bot by `rag-mode` |
| **Formats** | PDF, DOCX, DOC, XLSX, XLS, ODS, PPTX, PPT, ODP, EPUB, ODT, TXT, MD, HTML, CSV, JSON, YAML, TOML, and 30+ text-based formats |
| **Integration** | Automatic context injection into LLM responses |

## Basic Usage

```basic
' Load knowledge base
USE KB "policies"

' Bot now answers questions using that knowledge
' No explicit search needed - it's automatic
```

## Key Capabilities

- **Semantic Search** - Find content by meaning, not just keywords
- **Multi-Collection** - Organize documents into focused collections
- **Auto-Indexing** - Documents indexed automatically when added
- **Hybrid Search** - Combines dense (semantic) retrieval with term matching, fused by Reciprocal Rank Fusion
- **Context Management** - Relevant chunks injected into LLM prompts
- **Structured Docs search** - Saved Docs documents are indexed in a separate, process-local structure-aware index. `GET /api/docs/kb/search?q=<text>&limit=<n>` returns citation identity (`document_id`, `file`, `version`, `source_ref`, `section`, `score`) for documents the caller may read, never document text. In chat, a question about a saved document appends up to three citation-only lines to the system message for an authenticated user; anonymous turns get none. See [Document Indexing](./indexing.md).

## Document Organization

```
bot.gbkb/
├── policies/      → USE KB "policies"
├── products/      → USE KB "products"
└── support/       → USE KB "support"
```

## Tenant Isolation

Knowledge-base rows are tenant-scoped. `knowledge_sources` carries an owner
`bot_id`; each `knowledge_chunks` row reaches its tenant through
`knowledge_chunks.source_id → knowledge_sources.id`. The KB HTTP handlers
resolve that owner bot from the caller, never from a global default.

Authority for this section (paths relative to the repository root):
`botserver/crates/botsecurity-core/src/tenant.rs`,
`botserver/crates/botsources/src/handlers/kb_handlers/resolve_bot_id.rs`,
`.../handle_query_knowledge_base.rs` and `.../handle_get_stats.rs`.

### Resolution contract

| Rule | Behaviour | Source |
|------|-----------|--------|
| Tenant from verified claims only | The caller's branch is read from the `branch_id` claim of a server-minted, HS256-signed JWT in the `Authorization` header (Bearer or Basic transport). A missing, unsigned, tampered, wrong-key, non-HS256 or expired token yields no claims. | `tenant.rs:65-90` |
| Owner bot from that branch | `resolve_bot_id_for_branch` selects `WHERE branch_id = $1 AND is_default_for_branch = TRUE`, then authorizes the bot against the caller branch; a bot owned by a different branch is denied. | `resolve_bot_id.rs:14-71` |
| Client branch headers ignored | No client-supplied branch header or query parameter is consulted; the only header used for tenant scope is `Authorization`. | `resolve_bot_id.rs:43`, `tenant.rs:70-84` |
| Fail closed | When the branch cannot be resolved, handlers deny the request and return no KB rows. No global default bot is substituted. | `resolve_bot_id.rs:36-38`, `handle_query_knowledge_base.rs:24-34` |

### Scope of the guarantee

| Path | Tenant predicate / behaviour on denial | Source |
|------|----------------------------------------|--------|
| Upload document | New `knowledge_sources` row is stamped with the resolved `bot_id`; unresolved branch refuses the upload before insert. | `resolve_bot_id.rs:184-215` |
| List sources | `WHERE bot_id = $1`; unresolved branch returns an empty "Tenant scope unavailable" fragment. | `resolve_bot_id.rs:297-321` |
| Query KB | Chunks are joined to their source with `ks.bot_id = $3`; unresolved branch returns an empty result set and a warning. | `handle_query_knowledge_base.rs:26-85` |
| Get stats | All counts use `WHERE bot_id = $1`; unresolved branch returns zero counts. | `handle_get_stats.rs:8-70` |

A legacy global default-bot resolver still exists in `resolve_bot_id.rs:73-96`
for callers outside this module, but its own contract marks it "MUST NOT be used
for new KB reads or writes". The get/delete/reindex-by-id handlers in
`handle_query_knowledge_base.rs` address rows by `id` and are outside the
claim-scoped paths listed above.

> **Verification status.** This contract is traced to source at commit
> `a7a34f054` (issue #1387). It is not a recorded test PASS: live UAT against a
> running deployment is still required to confirm cross-tenant reads are denied
> end to end.

## Configuration

Retrieval strategy is one per-bot setting:

```csv
name,value
rag-mode,hybrid
```

`rag-mode` accepts `standard`, `hybrid`, `corrective`, `graph`, `agentic` or
`multimodal`, and defaults to `standard`. It is read from the bot's configuration
row (environment fallback `RAG_MODE`), not from `config.csv`. The older
`rag-hybrid-enabled` / `rag-dense-weight` / `rag-sparse-weight` keys are read only
by an unconnected crate and have no effect — see
[Retrieval and RAG](./hybrid-search.md).

## Performance Tips

1. **Organize collections** by topic for precise activation
2. **Clear unused KBs** to free memory: `CLEAR KB "old-docs"`
3. **Enable caching** for repeated queries
4. **Tune weights** based on content type (technical vs conversational)

## Learn More

- **[KB System Architecture](../03-knowledge-ai/README.md)** - Technical deep dive
- **[Retrieval and RAG](../03-knowledge-ai/hybrid-search.md)** - How retrieval works
- **[Document Indexing](../03-knowledge-ai/indexing.md)** - Processing pipeline
- **[Retrieval and RAG](./hybrid-search.md)** - The `rag-mode` setting and the six retrieval modes
- **[USE KB Keyword](../04-basic-scripting/keyword-use-kb.md)** - Complete reference
- **[.gbkb Package](../02-architecture-packages/gbkb.md)** - Folder structure