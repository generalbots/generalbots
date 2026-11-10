# Model Routing and Fallback 🟡 BETA

There are three separate mechanisms in the code that could be called "model
routing". They are not layered; they are independent, and only one of them is
on the live chat path today. This page states what each one does and which is
actually wired.

| Mechanism | Crate / module | Wired to chat? |
|---|---|---|
| Boot-time single provider from Vault | `botserver/src/main_module/state_builder.rs` | **Yes** — the path every chat turn uses |
| Multi-profile router, circuit breaker, council | `botserver/crates/botllm/src/{profiles,router,breaker,council}.rs` | **No** — library API with no production caller |
| BASIC `USE MODEL` / `SET MODEL ROUTING` | `botserver/crates/botbasic_ai/src/keywords/model_routing/` | **No** — persists a session preference that the pipeline never reads |

## What actually answers a chat turn

At boot the server builds exactly one LLM provider:

```rust
// botserver/src/main_module/state_builder.rs
let base_llm_provider = crate::llm::provider_factory::create_llm_provider_from_url(/* url, model, key */);
```

The URL, model and key come from Vault (`secret/gbo/{org}/{branch}/{bot}` with
the global `secret/gbo/llm` fallback), and the provider type is derived from the
URL pattern. Every request in `botserver/src/core/bot/pipeline/llm/` uses that
provider. Per-bot model selection therefore happens at boot, not per turn.

If Vault has no LLM entry the server falls back to `LLM_URL`, `LLM_MODEL` and
`LLM_KEY` (or `OPENAI_API_KEY`). This is the same set of variables the legacy
profile reader uses, which is why the two configurations never contradict each
other on a single-provider deployment.

## Multi-profile routing (`botllm`)

### Profile resolution order

`profiles::load_profiles()` reads, in order, and stops at the first source that
yields at least one entry:

1. `GB_LLM_PROFILES` — a JSON array of profiles in the environment.
2. `GB_LLM_PROFILES_FILE` — a path to a JSON file with the same array.
3. The legacy single provider built from `LLM_URL` / `LLM_KEY` (or
   `OPENAI_API_KEY`) / `LLM_MODEL`, emitted as one profile with id `"default"`.

Entries whose `url` is empty are discarded. A malformed JSON array logs an error
and contributes nothing.

### Profile fields

| Field | Meaning |
|---|---|
| `id` | Stable identifier used in logs, breaker state and council attribution |
| `url` | OpenAI-compatible chat-completions base URL |
| `model` | Model identifier sent to the provider |
| `key` | Bearer key for this provider |
| `caps` | Capability tags used for filtering, e.g. `vision`, `tools` |
| `cost_per_1k` | Monetary cost per 1000 tokens, used by cheap preference |
| `priority` | Ordering weight; higher is attempted first |

Example file (values are placeholders, not defaults):

```json
[
  {"id": "primary", "url": "https://provider.example/v1", "model": "large",
   "caps": ["tools", "vision"], "priority": 100, "cost_per_1k": 0.01},
  {"id": "cheap", "url": "https://provider.example/v1", "model": "small",
   "caps": ["tools"], "priority": 10, "cost_per_1k": 0.0002}
]
```

### Decision path

```text
chat_with_fallback(system, user, params, caps)
        │
        ├─ route_candidates(caps, prefer_cheap)
        │     ├─ drop profiles whose breaker is open
        │     ├─ drop profiles missing a requested capability
        │     ├─ sort by effective_priority (cost-adjusted when prefer_cheap)
        │     └─ round-robin rotate the list
        │
        ├─ for each candidate (max 4 attempts):
        │     ├─ acquire that profile's semaphore (8 permits)
        │     ├─ generate via the existing provider client
        │     ├─ success → record_success(), return text + RouteMeta
        │     └─ failure → record_failure(5, 60s), try next candidate
        │
        └─ all failed → Err("all LLM profiles failed after N attempt(s): …")
```

Constants in `router.rs`: `MAX_CONCURRENCY_PER_PROFILE = 8`,
`FAILURE_THRESHOLD = 5`, `OPEN_SECS = 60`, `MAX_ATTEMPTS = 4`.
`RouteMeta` reports `profile_id`, `tokens_in`, `tokens_out` and `fallback_used`,
so a caller can tell that a fallback happened.

### Circuit breaker

`breaker.rs` counts **consecutive** failures per profile. When the count reaches
the threshold the breaker stores an `open_until` instant and `allow()` returns
`false` until it passes. Any success resets the counter and closes the circuit.
There is no half-open probe: the first request after expiry is the probe.

### Council mode

`council::council_answer(system, user, n, caps)` selects up to `n` **distinct**
profiles (deduplicated by `id`), asks them concurrently through the same
`attempt` helper, drops failures, then merges the surviving answers with a
second routed call whose system prompt is the arbiter instruction. The returned
`CouncilOutcome` lists `contributors` so the caller knows which profiles
answered. With fewer than `n` healthy profiles it uses all available ones; if
none answer it returns `all council contributors failed`.

### The gap

`chat_with_fallback` and `council_answer` have **no callers outside `botllm`**.
Grepping the workspace finds the definitions, the council's internal call, and
the tests. Setting `GB_LLM_PROFILES` therefore does not change how a chat turn
is answered; it only configures a router that nothing invokes. Treat the router
as an available library API, not as a deployed feature, until a caller is added
to `pipeline/llm/`.

## BASIC model keywords

`register_model_routing_keywords` registers four keywords. Their behaviour is
narrower than the reference page suggests:

| Keyword | What it does | What it does not do |
|---|---|---|
| `USE MODEL "name"` | Upserts `session_preferences (session_id, 'current_model')` | Does not change the provider used to generate the reply |
| `SET MODEL ROUTING "auto"` | Upserts `session_preferences (session_id, 'model_routing')` | Nothing reads the value on the chat path |
| `GET CURRENT MODEL` | Reads the preference back, falling back to the bot's configured model | — |
| `LIST MODELS` | Lists model entries built from `llm-model[-suffix]`, `llm-url[-suffix]`, `llm-key[-suffix]` Vault keys | — |

`ModelRouter::auto_route` implements a keyword heuristic (`code`, `quality`,
`fast` buckets) over the models named in the bot's `config.csv` model list, and
`load_balanced_route` returns the default model unchanged. Because no pipeline
code reads `current_model`, these keywords are a control surface without an
actuator today: they record intent in the database and report it back.

## Smart router

`botllm/src/smart_router.rs` implements a fourth mechanism: a performance cache
keyed by model name with `OptimizationGoal::{Speed, Cost, Quality, Balanced}`
and a weighted `Balanced` score. Its only intended entry point is the
`LLM "<prompt>"` BASIC keyword in
`botbasic_ai/src/keywords/enhanced_llm.rs`. That keyword is **not registered**:
`register_enhanced_llm_keyword` has no caller anywhere in `botserver/src`
(`botbasic_ai/src/keywords/mod.rs` notes that it needs `Arc<AppState>` and must
be registered at the botserver layer, but no such call exists). Even if it were
registered, the performance cache is populated only by `track_performance`,
which also has no callers, so `select_optimal_model` would return its hardcoded
fallback model name. Treat this module as unreachable.

## Related pages

- [AI and LLM Integration](../03-knowledge-ai/ai-llm.md) — Vault keys and
  provider configuration.
- [LLM Configuration](../10-configuration-deployment/llm-config.md) — the
  per-bot settings an operator edits.
- [USE MODEL](../04-basic-scripting/keyword-use-model.md) and
  [MODEL ROUTE Keywords](../04-basic-scripting/keyword-model-route.md) — the
  keyword surface.
