# SET_CONTEXT Keyword

The **SET_CONTEXT** keyword defines the operational context for the bot’s current session.  
It allows scripts to switch between different logical modes or workflows, influencing how subsequent commands are interpreted.

---

## Syntax

```basic
SET_CONTEXT "context-name"
```

---

## Parameters

- `"context-name"` — A string representing the new context.  
  Common examples include `"sales_mode"`, `"support_mode"`, or `"training_mode"`.

---

## Description

`SET_CONTEXT` updates the bot’s internal state to reflect a specific operational context.  
Contexts are used to modify behavior dynamically — for example, changing which tools are active, which memory entries are prioritized, or which prompts are used for LLM responses.

When a context is set, the bot automatically adjusts its logic and available commands to match that mode.  
This enables modular dialog design and flexible automation workflows.

If the context name does not exist, the system creates a new one automatically and stores it in the session cache.

---

## Example

```basic
' Switch to sales mode
SET_CONTEXT "sales_mode"

' Perform a context-specific action
TALK "Welcome to the sales assistant. How can I help you today?"

' Later, switch to support mode
SET_CONTEXT "support_mode"
TALK "Support mode activated. Please describe your issue."
```

---

## Implementation Notes

- Implemented in Rust under `src/context/mod.rs` and `src/context/langcache.rs`.  
- The keyword interacts with the session manager and context cache to update the active context.  
- Contexts are stored in memory and optionally persisted in cache component or a local cache file.  
- Changing context may trigger automatic loading of associated tools or memory entries.

---

## Related Keywords

- [`SET BOT MEMORY`](keyword-set-bot-memory.md) — Stores persistent data for the bot or user.  
- [`GET BOT MEMORY`](keyword-get-bot-memory.md) — Retrieves stored memory entries.
- [`SET SCHEDULE`](keyword-set-schedule.md) — Defines scheduled tasks that may depend on context.

---

## Summary

`SET_CONTEXT` is a key command for managing dynamic behavior in GeneralBots.  
It enables flexible, modular workflows by allowing scripts to switch between operational modes seamlessly.
