# System Limits Reference

This document provides a comprehensive reference for all system limits, rate limits, package sizes, and configurable parameters in General Bots. Each limit includes the config.csv key, default value, and the source code location where it's enforced.

---

## Quick Reference

| Category | Limit | Default | Config Key |
|----------|-------|---------|------------|
| Package Size | Total package | 100 MB | `package-max-size` |
| Package Size | Single file | 10 MB | `user-file-limit` |
| Package Size | File count | 1,000 | `user-file-count` |
| Package Size | Script size | 1 MB | `script-max-size` |
| Session | Message history | 50 | `session-message-history` |
| Session | Variable storage | 1 MB | `session-variable-limit` |
| Session | Concurrent sessions | 1,000 | `session-max-concurrent` |
| Session | Rate limit | 60/min | `session-rate-limit` |
| Session | Timeout | 30 min | `session-timeout` |
| Knowledge Base | Collections | 50 | `kb-max-collections` |
| Knowledge Base | Document size | 50 MB | `kb-doc-max-size` |
| File Upload | Per file | 10 MB | `upload-max-size` |
| File Upload | Attachment | 25 MB | `attachment-max-size` |
| API | Rate limit | 100/min | `api-rate-limit` |
| Loop Safety | Max iterations | 100,000 | `loop-max-iterations` |
| GOTO Safety | Max iterations | 1,000,000 | `goto-max-iterations` |

---

## Package Size Limits

Controls the size and composition of `.gbai` packages.

### Total Package Size

| Property | Value |
|----------|-------|
| **Config Key** | `package-max-size` |
| **Default** | 104,857,600 (100 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/core/package_manager/mod.rs` |

```csv
name,value
package-max-size,209715200
```

### Single Document Size

| Property | Value |
|----------|-------|
| **Config Key** | `user-file-limit` |
| **Default** | 10,485,760 (10 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/core/package_manager/mod.rs` |

### File Count Per Package

| Property | Value |
|----------|-------|
| **Config Key** | `user-file-count` |
| **Default** | 1,000 |
| **Unit** | Files |
| **Source** | `botserver/src/core/package_manager/mod.rs` |

### Script File Size

| Property | Value |
|----------|-------|
| **Config Key** | `script-max-size` |
| **Default** | 1,048,576 (1 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/basic/compiler/mod.rs` |

---

## Session Limits

Controls resource usage per user session.

### Message History

| Property | Value |
|----------|-------|
| **Config Key** | `session-message-history` |
| **Default** | 50 |
| **Unit** | Messages |
| **Source** | `botserver/src/core/session/mod.rs` |
| **Notes** | Messages kept in LLM context window |

### Variable Storage

| Property | Value |
|----------|-------|
| **Config Key** | `session-variable-limit` |
| **Default** | 1,048,576 (1 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/core/session/mod.rs` |
| **Notes** | Total size of all session variables |

### Concurrent Sessions

| Property | Value |
|----------|-------|
| **Config Key** | `session-max-concurrent` |
| **Default** | 1,000 |
| **Unit** | Sessions |
| **Source** | `botserver/src/core/session/mod.rs` |
| **Notes** | Per server instance |

### Session Rate Limit

| Property | Value |
|----------|-------|
| **Config Key** | `session-rate-limit` |
| **Default** | 60 |
| **Unit** | Messages per minute |
| **Source** | `botserver/src/core/session/mod.rs` |

### Session Timeout

| Property | Value |
|----------|-------|
| **Config Key** | `session-timeout` |
| **Default** | 1,800 (30 minutes) |
| **Unit** | Seconds |
| **Source** | `botserver/src/core/session/mod.rs` |

---

## Knowledge Base Limits

Controls document ingestion and vector storage.

### Maximum Collections

| Property | Value |
|----------|-------|
| **Config Key** | `kb-max-collections` |
| **Default** | 50 |
| **Unit** | Collections |
| **Source** | `botserver/src/basic/keywords/kb.rs` |

### Document Size by Type

| File Type | Max Size | Config Key |
|-----------|----------|------------|
| PDF | 50 MB | `kb-pdf-max-size` |
| Word (.docx) | 25 MB | `kb-word-max-size` |
| Excel (.xlsx) | 25 MB | `kb-excel-max-size` |
| Text/Markdown | 10 MB | `kb-text-max-size` |
| Images | 10 MB | `kb-image-max-size` |

### RAG Parameters

| Config Key | Default | Description |
|------------|---------|-------------|
| `rag-top-k` | 10 | Number of chunks to retrieve |
| `rag-chunk-size` | 512 | Tokens per chunk |
| `rag-chunk-overlap` | 50 | Overlap between chunks |
| `rag-hybrid-enabled` | true | Enable hybrid search |
| `rag-rerank-enabled` | false | Enable reranking |

---

## File Upload Limits

Controls file upload operations.

### Standard Upload

| Property | Value |
|----------|-------|
| **Config Key** | `upload-max-size` |
| **Default** | 10,485,760 (10 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/api/upload.rs` |

### Email Attachment

| Property | Value |
|----------|-------|
| **Config Key** | `attachment-max-size` |
| **Default** | 26,214,400 (25 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/basic/keywords/send_mail.rs` |

### Archive Extraction

| Limit | Default | Config Key |
|-------|---------|------------|
| Archive size | 100 MB | `extract-archive-max-size` |
| Extracted size | 500 MB | `extract-output-max-size` |
| Files in archive | 10,000 | `extract-max-files` |
| Path depth | 10 | `extract-max-depth` |

---

## API Rate Limits

Controls API request rates.

### General API

| Property | Value |
|----------|-------|
| **Config Key** | `api-rate-limit` |
| **Default** | 100 |
| **Unit** | Requests per minute |
| **Source** | `botserver/src/api/middleware/rate_limit.rs` |

### Endpoint-Specific Limits

| Endpoint Category | Limit | Config Key |
|-------------------|-------|------------|
| Standard endpoints | 100/min | `api-rate-limit` |
| Compliance scans | 5/hour | `api-scan-rate-limit` |
| Report generation | 10/hour | `api-report-rate-limit` |
| LLM inference | 20/min | `llm-rate-limit` |
| Embedding | 100/min | `embedding-rate-limit` |

### Rate Limit Headers

All API responses include rate limit headers:

| Header | Description |
|--------|-------------|
| `X-RateLimit-Limit` | Maximum requests allowed |
| `X-RateLimit-Remaining` | Requests remaining in window |
| `X-RateLimit-Reset` | Unix timestamp when limit resets |

---

## Loop Safety Limits

Prevents infinite loops in BASIC scripts.

### WHILE/DO Loops

| Property | Value |
|----------|-------|
| **Config Key** | `loop-max-iterations` |
| **Default** | 100,000 |
| **Unit** | Iterations |
| **Source** | `botserver/src/basic/keywords/procedures.rs` |

### GOTO State Machine

| Property | Value |
|----------|-------|
| **Config Key** | `goto-max-iterations` |
| **Default** | 1,000,000 |
| **Unit** | Iterations |
| **Source** | `botserver/src/basic/compiler/goto_transform.rs` |

---

## Sandbox Limits

Controls code execution sandbox resources.

### Memory Limit

| Property | Value |
|----------|-------|
| **Config Key** | `sandbox-memory-mb` |
| **Default** | 256 |
| **Unit** | Megabytes |
| **Source** | `botserver/src/basic/keywords/code_sandbox.rs` |

### CPU Limit

| Property | Value |
|----------|-------|
| **Config Key** | `sandbox-cpu-percent` |
| **Default** | 50 |
| **Unit** | Percent |
| **Source** | `botserver/src/basic/keywords/code_sandbox.rs` |

### Execution Timeout

| Property | Value |
|----------|-------|
| **Config Key** | `sandbox-timeout` |
| **Default** | 30 |
| **Unit** | Seconds |
| **Source** | `botserver/src/basic/keywords/code_sandbox.rs` |

---

## Communication Limits

### WhatsApp

| Limit | Default | Config Key |
|-------|---------|------------|
| Messages per second | 10 | `whatsapp-rate-limit` |
| Broadcast recipients | 1,000 | `whatsapp-broadcast-max` |
| Template message size | 1,024 | `whatsapp-template-max-size` |

### Email

| Limit | Default | Config Key |
|-------|---------|------------|
| Recipients per email | 50 | `email-max-recipients` |
| Emails per hour | 100 | `email-rate-limit` |
| Attachment size | 25 MB | `email-attachment-max-size` |

### Delegate to Bot

| Property | Value |
|----------|-------|
| **Config Key** | `delegate-message-max-size` |
| **Default** | 1,048,576 (1 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/basic/keywords/delegate_to_bot.rs` |

| Property | Value |
|----------|-------|
| **Config Key** | `delegate-timeout` |
| **Default** | 300 |
| **Unit** | Seconds |
| **Source** | `botserver/src/basic/keywords/delegate_to_bot.rs` |

---

## Storage Limits

### User Storage Quota

| Property | Value |
|----------|-------|
| **Config Key** | `user-storage-quota` |
| **Default** | 104,857,600 (100 MB) |
| **Unit** | Bytes |
| **Source** | `botserver/src/basic/keywords/drive.rs` |

### Download Link Expiry

| Property | Value |
|----------|-------|
| **Config Key** | `download-link-expiry` |
| **Default** | 86,400 (24 hours) |
| **Unit** | Seconds |
| **Source** | `botserver/src/basic/keywords/download.rs` |

---

## LLM Limits

### Token Limits

| Config Key | Default | Description |
|------------|---------|-------------|
| `llm-max-tokens` | 4,096 | Max output tokens |
| `llm-context-window` | 8,192 | Context window size |
| `llm-temperature` | 0.7 | Default temperature |

### Tokens Per Minute (TPM)

| Property | Value |
|----------|-------|
| **Config Key** | `llm-tpm-limit` |
| **Default** | 20,000 |
| **Unit** | Tokens per minute |
| **Source** | `botcoder/src/main.rs` |
| **Env Var** | `LLM_TPM` |

---

## A2A Protocol Limits

### Maximum Hops

| Property | Value |
|----------|-------|
| **Config Key** | `a2a-max-hops` |
| **Default** | 5 |
| **Unit** | Hops |
| **Source** | `botserver/src/basic/keywords/a2a_protocol.rs` |
| **Notes** | Prevents infinite delegation chains |

---

## Video/Audio Limits

### Player Limits

| Config Key | Default | Description |
|------------|---------|-------------|
| `player-max-file-size-mb` | 100 | Max video file size |
| `player-default-volume` | 80 | Default volume (0-100) |
| `player-preload` | metadata | Preload strategy |

---

## Configuring Limits

### Via config.csv

Add entries to your bot's `config.csv` file:

```csv
name,value
package-max-size,209715200
session-rate-limit,120
api-rate-limit,200
llm-max-tokens,8192
```

### Via Environment Variables

Some limits can be set via environment variables (overrides config.csv):

| Environment Variable | Config Key |
|---------------------|------------|
| `LLM_TPM` | `llm-tpm-limit` |
| `SESSION_TIMEOUT` | `session-timeout` |
| `API_RATE_LIMIT` | `api-rate-limit` |

### Via API

Update limits programmatically:

```basic
SET CONFIG "session-rate-limit" TO "120"
SET CONFIG "api-rate-limit" TO "200"
```

---

## Monitoring Limits

### Viewing Current Limits

```basic
config = GET CONFIG "api-rate-limit"
TALK "Current API rate limit: " + config
```

### Rate Limit Errors

When limits are exceeded, the system returns:

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 429 | `RATE_LIMITED` | Too many requests |
| 413 | `PAYLOAD_TOO_LARGE` | File/request too large |
| 507 | `INSUFFICIENT_STORAGE` | Storage quota exceeded |

---

## Best Practices

1. **Start Conservative**: Begin with default limits and increase as needed
2. **Monitor Usage**: Track rate limit headers to understand usage patterns
3. **Plan for Scale**: Increase limits gradually as traffic grows
4. **Document Changes**: Track limit changes in your bot's changelog
5. **Test Limits**: Verify your application handles limit errors gracefully

---

## Related Documentation

- [Session Management](../01-introduction/sessions.md)
- [Package Structure](../02-templates/gbai.md)
- [Knowledge Base](../03-knowledge-base/README.md)
- [API Reference](../08-rest-api-tools/README.md)