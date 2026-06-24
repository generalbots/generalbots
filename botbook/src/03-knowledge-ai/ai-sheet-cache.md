# AI Sheet Cache 🟡 BETA

> **Intelligent caching for AI-powered spreadsheet formulas**

## Overview

The AI Sheet Cache provides an asynchronous evaluation pipeline for `=BOT_AI_PROMPT()` formulas inside spreadsheets. Each formula invocation is cached using a SHA256 hash of the prompt template and referenced cell values, preventing redundant LLM API calls and reducing costs.

## Architecture

```
User edits cell with =BOT_AI_PROMPT()
  │
  ▼
Formula Parser (botsheet-core)
  │
  ▼
  ├── SHA256 cache key (prompt + cell values)
  ├── Cache hit → return cached result immediately
  └── Cache miss →
       ├── Acquire Semaphore (max 10 concurrent)
       ├── Call LLM API
       ├── Store in cache
       └── Return result
```

## BOT_AI_PROMPT Formula

```excel
=BOT_AI_PROMPT("Analyze the tone of: ", A2)
=BOT_AI_PROMPT("Translate to Portuguese: ", B2)
=BOT_AI_PROMPT("Summarize in 3 bullet points: ", C2, D2)
```

The formula parser extracts all cell references as dependencies. When referenced cells change, the cached result is invalidated automatically.

## Cache Layer

| Property | Value |
|----------|-------|
| Key algorithm | SHA256 |
| Key components | Prompt template + referenced cell values + model name |
| Backend | Valkey (Redis-compatible) |
| TTL | Permanent until referenced cells change |
| Rate limiting | Tokio Semaphore (max 10 concurrent LLM calls) |

## Batch Evaluation

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/sheet/ai/evaluate` | POST | Evaluate a single AI formula |
| `/api/sheet/ai/batch-evaluate` | POST | Batch evaluate up to 100 formulas |

**Batch request format:**
```json
{
  "cells": [
    { "prompt": "Analyze: ", "values": ["Great product!"] },
    { "prompt": "Translate: ", "values": ["Hello world"] }
  ]
}
```

**Batch response:**
```json
{
  "results": [
    { "index": 0, "value": "Positive feedback", "cached": false },
    { "index": 1, "value": "Olá mundo", "cached": true }
  ]
}
```

## Cost Controls

- Maximum batch size: 100 formulas per request
- Concurrent LLM requests: 10 (configurable via Semaphore)
- Cache TTL: configurable per-bot via `config.csv`
- Error cells render `#VALUE!` or `#TIMEOUT!` — never crash

## UI Integration

The Sheet app (see [Sheet - Spreadsheets](../07-user-interface/apps/sheet.md)) renders a loading state (`...`) while AI formulas are being evaluated, then swaps the cell content via HTMX when the result is ready.

## Configuration

| config.csv key | Description | Default |
|----------------|-------------|---------|
| `sheet-ai-enabled` | Enable AI formulas | `true` |
| `sheet-ai-max-batch` | Max batch size | `100` |
| `sheet-ai-concurrency` | Max concurrent LLM calls | `10` |
| `sheet-ai-cache-ttl` | Cache TTL in seconds | `86400` |

## Feature Flag

Enable with `sheet` feature flag in Cargo.toml:
```toml
botserver = { features = ["sheet"] }
```
