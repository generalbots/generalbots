# System Limits 🟡 BETA

Limits the platform enforces, and where each one is defined. Every value on this
page is read from the source file named beside it — a limit that is not in the
code is not on this page.

> **Verified 2026-09-13.** The authority is `botlib/src/limits/types.rs`, which
> defines `SystemLimits` and its defaults. If a number here disagrees with that
> file, the file is right.

> **Correction.** An earlier revision of this page listed per-operation
> configuration keys — `session-timeout`, `llm-max-tokens`, `kb-pdf-max-size`,
> `upload-max-size`, `rag-chunk-size` and around forty others. **None of those
> keys exist in the code.** They have been removed rather than softened. The
> limits themselves are real, but they are compile-time defaults in
> `SystemLimits`, not per-bot settings, and most are not configurable.

## Where limits come from

| Layer | Defined in | Configurable |
|---|---|---|
| Platform limits (`SystemLimits`) | `botlib/src/limits/types.rs` | No — compile-time constants |
| HTTP request rate | `botserver/crates/botsecurity-auth/src/rate_limiter.rs` | No — three fixed profiles |
| Code sandbox | `botserver/crates/botbasic_ai/src/keywords/code_sandbox.rs` | Yes — per-bot bot config |
| Knowledge-base indexing | `botserver/crates/botqdrant/src/drive_vectordb.rs` | No — fixed cap |

## Platform limits

Defaults from `SystemLimits` (`botlib/src/limits/types.rs`).

### Execution

| Limit | Default | Constant |
|---|---|---|
| Loop iterations | 100,000 | `MAX_LOOP_ITERATIONS` |
| Recursion depth | 100 | `MAX_RECURSION_DEPTH` |
| Script execution time | 300 s | `MAX_SCRIPT_EXECUTION_SECONDS` |
| Pending tasks | 1,000 | `MAX_PENDING_TASKS` |
| Tools per bot | 500 | `MAX_TOOLS_PER_BOT` |

`MAX_LOOP_ITERATIONS` and `MAX_RECURSION_DEPTH` exist to stop a runaway script
from consuming the process. A script that hits either is terminated, not queued.

### Data and memory

| Limit | Default | Constant |
|---|---|---|
| String length | 10 MiB | `MAX_STRING_LENGTH` |
| Array length | 1,000,000 elements | `MAX_ARRAY_LENGTH` |
| Database query results | 10,000 rows | `MAX_DB_QUERY_RESULTS` |
| Database connections per tenant | 20 | `MAX_DB_CONNECTIONS_PER_TENANT` |

### Files and storage

| Limit | Default | Constant |
|---|---|---|
| Single file | 100 MiB | `MAX_FILE_SIZE_BYTES` |
| Upload | 50 MiB | `MAX_UPLOAD_SIZE_BYTES` |
| Request body | 10 MiB | `MAX_REQUEST_BODY_BYTES` |
| Drive storage per tenant | 10 GiB | `MAX_DRIVE_STORAGE_BYTES` |

### Knowledge base

| Limit | Default | Constant |
|---|---|---|
| Documents per bot | 100,000 | `MAX_KB_DOCUMENTS_PER_BOT` |
| Document size | 50 MiB | `MAX_KB_DOCUMENT_SIZE_BYTES` |

Separately, the indexer skips any file larger than **10 MiB** regardless of type:
`drive_vectordb.rs::should_index` returns `false` above that size. The type is
matched against an allow-list at the same point, so an unsupported type is
skipped rather than truncated.

### LLM

| Limit | Default | Constant |
|---|---|---|
| Tokens per request | 128,000 | `MAX_LLM_TOKENS_PER_REQUEST` |
| Requests per minute | 60 | `MAX_LLM_REQUESTS_PER_MINUTE` |

### Concurrency and sessions

| Limit | Default | Constant |
|---|---|---|
| Concurrent requests per user | 100 | `MAX_CONCURRENT_REQUESTS_PER_USER` |
| Concurrent requests (global) | 10,000 | `MAX_CONCURRENT_REQUESTS_GLOBAL` |
| WebSocket connections per user | 10 | `MAX_WEBSOCKET_CONNECTIONS_PER_USER` |
| WebSocket connections (global) | 50,000 | `MAX_WEBSOCKET_CONNECTIONS_GLOBAL` |
| Sessions per user | 10 | `MAX_SESSIONS_PER_USER` |
| Session idle timeout | 3,600 s | `MAX_SESSION_IDLE_SECONDS` |
| Bots per tenant | 100 | `MAX_BOTS_PER_TENANT` |

### API throughput

| Limit | Default | Constant |
|---|---|---|
| API calls per minute | 1,000 | `MAX_API_CALLS_PER_MINUTE` |
| API calls per hour | 10,000 | `MAX_API_CALLS_PER_HOUR` |

## HTTP rate limiting

Three fixed profiles in `botsecurity-auth/src/rate_limiter.rs`, expressed as
requests per second with a burst allowance. `CombinedRateLimiter` applies these
alongside the per-bot limits.

| Profile | Requests / second | Burst | Used by |
|---|---|---|---|
| `api()` | 100 | 150 | The API surface, applied at server start |
| `strict()` | 50 | 100 | Authentication-sensitive routes |
| `relaxed()` | 500 | 1,000 | High-volume read paths |

The window for the platform counters is 60 s with a burst multiplier of 1.5
(`RATE_LIMIT_WINDOW_SECONDS`, `RATE_LIMIT_BURST_MULTIPLIER`).

## Code sandbox

The only limits on this page that are configurable per bot. They are read from the
bot's configuration, not from `config.csv`:

| Key | Default | Meaning |
|---|---|---|
| `sandbox-enabled` | `true` | Whether sandbox execution is permitted |
| `sandbox-runtime` | — | Runtime selector |
| `sandbox-timeout` | 30 s | Wall-clock limit for one execution |
| `sandbox-memory-mb` (alias `sandbox-memory-limit`) | 256 MiB | Memory ceiling |
| `sandbox-cpu-percent` (alias `sandbox-cpu-limit`) | 50% | CPU ceiling |
| `sandbox-network` (alias `sandbox-network-enabled`) | — | Whether the sandbox may reach the network |
| `sandbox-python-packages` | — | Comma-separated packages made available |

Source: `botserver/crates/botbasic_ai/src/keywords/code_sandbox.rs`.

> An earlier revision cited this file as
> `botserver/src/basic/keywords/code_sandbox.rs`. That path no longer exists — the
> module moved into the `botbasic_ai` crate. All of the `sandbox-*` keys above are
> real; the path was not.

## Storage quota

Drive usage against the tenant quota is reported by `GET /api/files/quota`. The
quota ceiling itself is `MAX_DRIVE_STORAGE_BYTES` — it is not set per user or per
bot.

## What is not configurable

There is no supported way to raise or lower the `SystemLimits` values at runtime:
they are constants compiled into `botlib`. Changing one requires a code change and
a rebuild. If a workload needs a different ceiling, that is a change to
`botlib/src/limits/types.rs`, not a setting.

## See Also

- [Configuration Parameters](./parameters.md) — the keys that do exist
- [Retrieval and RAG](../03-knowledge-ai/hybrid-search.md) — what retrieval does, and its limits
- [Security](../09-security/README.md) — rate limiting and request guards
