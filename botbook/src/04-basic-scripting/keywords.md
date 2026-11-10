# Keyword Reference 🟡 BETA

This section lists every BASIC keyword implemented in the GeneralBots engine. Each keyword page includes:

* **Syntax** – Exact command format
* **Parameters** – Expected arguments
* **Description** – What the keyword does
* **Example** – A short snippet showing usage

The source code for each keyword lives in `src/basic/keywords/`. Only the keywords listed here exist in the system.

## Important: Case Insensitivity

**All variables in General Bots BASIC are case-insensitive.** The preprocessor normalizes variable names to lowercase automatically.

```basic
' These all refer to the same variable
host = "https://api.example.com"
result = GET Host + "/endpoint"
TALK HOST
```

Keywords are also case-insensitive but conventionally written in UPPERCASE:

```basic
' Both work identically
TALK "Hello"
talk "Hello"
```

## Configuration Variables (param-*)

Variables defined with `param-` prefix in `config.csv` are automatically available in scripts without the prefix:

```csv
name,value
param-host,https://api.example.com
param-limit,100
param-pages,50
```

```basic
' Access directly (lowercase, no param- prefix)
result = GET host + "/items?limit=" + limit
```

See [Script Execution Flow](./script-execution-flow.md) for complete details.

---

## Keyword Lists Are Generated

Every keyword and function is documented in the
[generated BASIC reference](./generated/index.md), built from the
`register_custom_syntax` / `register_fn` calls in
`botserver/crates/botbasic_*/src/**` and `botserver/src/basic/keywords/**`.
Do not add keyword tables to this page: a hand-written list drifts from the
runtime, the generator cannot.

| Need | Page |
|---|---|
| Keyword, signature, status, source file, docs link | [Generated reference](./generated/index.md) |
| Block constructs, first-token dispatch, directives, rewrites | [Grammar summary](./generated/grammar-summary.md) |
| Long-form per-keyword guides | [Keyword pages](./keyword-talk.md) (`keyword-*.md`) |

```bash
python3 scripts/gen_basic_reference.py            # regenerate
python3 scripts/gen_basic_reference.py --check    # fail on drift
```

## Syntax Rules

### DO ✅

```basic
' Variable names (no underscores in names)
let ticketnumber = "TKT001"
let useremail = "user@example.com"

' SAVE with field names = variable names
SAVE "table.csv", ticketnumber, useremail, status

' Keywords with spaces
SET BOT MEMORY "last_ticket", ticketnumber
SET CONTEXT "name" AS "description"
ADD SUGGESTION "key" AS "Display text"
CLEAR SUGGESTIONS
USE KB "myknowledge"
USE TOOL "mytool"

' GET BOT MEMORY as function
let lastticket = GET BOT MEMORY("last_ticket")
```

### DON'T ❌

```basic
' NO: Complex object operations
SET object.field = value  ' WRONG
SAVE "table", object.id, object  ' WRONG

' NO: IF for input validation (use HEAR AS TYPE instead)
IF value = "" THEN  ' OK for logic, but for input use:
HEAR value AS STRING  ' Better - validates input type
```

---

## Prompt Blocks

Special multi-line blocks for AI configuration and formatted output:

| Block | Purpose | Documentation |
|-------|---------|---------------|
| `BEGIN SYSTEM PROMPT ... END SYSTEM PROMPT` | Define AI persona, rules, capabilities | [Prompt Blocks](./prompt-blocks.md) |
| `BEGIN TALK ... END TALK` | Formatted multi-line messages with Markdown | [Prompt Blocks](./prompt-blocks.md) |

```basic
BEGIN SYSTEM PROMPT
You are a helpful assistant for AcmeStore.
Rules:
1. Always be polite
2. Never discuss competitors
END SYSTEM PROMPT

BEGIN TALK
**Welcome!** 🎉

I can help you with:
• Orders
• Tracking
• Returns
END TALK
```

---

## Script Structure

### No MAIN Function

Scripts execute from line 1 - no `MAIN` or entry point needed:

```basic
' ✅ CORRECT - Start directly
TALK "Hello!"
ADD TOOL "my-tool"

' ❌ WRONG - Don't use MAIN
SUB MAIN()
    TALK "Hello"
END SUB
```

### SUB and FUNCTION for Reuse

Use for helper code within tools, not as entry points:

```basic
FUNCTION CalculateTotal(price, quantity)
    RETURN price * quantity
END FUNCTION

SUB NotifyAdmin(message)
    SEND EMAIL admin1, message
END SUB

' Execution starts here
total = CalculateTotal(19.99, 3)
CALL NotifyAdmin("Order processed")
```

See [Script Execution Flow](./script-execution-flow.md) for entry points and lifecycle.

---

## Notes

- Keywords are case-insensitive (TALK = talk = Talk)
- Variables are case-insensitive (host = HOST = Host)
- String parameters can use double quotes or single quotes
- Comments start with REM or '
- Line continuation uses underscore (_)
- Objects are created with `#{ key: value }` syntax
- Arrays use `[item1, item2, ...]` syntax
- param-* config values become global variables

---

## See Also

- [Script Execution Flow](./script-execution-flow.md) - Entry points and lifecycle
- [Prompt Blocks](./prompt-blocks.md) - BEGIN SYSTEM PROMPT & BEGIN TALK
- [Basics](./basics.md) - Core concepts
- [Examples](./examples-consolidated.md) - Real-world patterns