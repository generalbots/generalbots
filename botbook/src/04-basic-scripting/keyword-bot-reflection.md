# BOT REFLECTION 🟡 BETA

Agent self-analysis: the bot evaluates a session with the LLM and returns a
summary of what went wrong and what to improve.

| Form | Registered? | Where |
|---|---|---|
| `BOT REFLECTION …` | ❌ no | advertised by `get_all_keywords()` only |
| `SET BOT REFLECTION $expr$` | ✅ yes | `botserver/crates/botbasic_ai/src/keywords/agent_reflection/register_reflection_keywords.rs` |
| `REFLECT ON $expr$` | ✅ yes | same file |
| `GET REFLECTION INSIGHTS()` | ✅ yes | same file (`register_fn`) |

A script that writes `BOT REFLECTION enabled` does not run: there is no
registration for that form. The generated [keyword reference](./generated/index.md)
lists the advertised name under "Advertised but not registered".

## Syntax

```basic
SET BOT REFLECTION true          ' true | 1 | on enable reflection for the bot
summary = REFLECT ON "session"   ' runs the reflection engine, returns a summary string
insights = GET REFLECTION INSIGHTS()
```

## `SET BOT REFLECTION`

Stores the reflection flag for the bot. The argument is evaluated and lowered;
`true`, `1` and `on` enable it, anything else disables it. The write happens on
a worker thread, so the statement returns before the database write completes.

## `REFLECT ON`

Runs `ReflectionEngine::reflect(session_id, reflection_type)` for the current
session and returns `result.summary()` as a string. The type string is parsed by
`ReflectionType::from`; an unknown string falls back to the default type. The
call blocks the script for up to 60 seconds and returns
`Reflection failed: …` as a string when the engine errors.

## `GET REFLECTION INSIGHTS()`

Returns the stored reflection insights for the bot as an array.

## See also

- [Generated BASIC reference](./generated/index.md) — registration status and source line
- [USE MODEL](./keyword-use-model.md) — the model that performs the reflection
