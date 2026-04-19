# BASIC Execution Modes: RUNTIME vs WORKFLOW

General Bots BASIC scripts run in one of two execution modes. The mode is selected by a pragma at the top of the `.bas` file.

## Quick Comparison

| Feature | RUNTIME mode (default) | WORKFLOW mode ⚗️ |
|---------|----------------------|-----------------|
| **Pragma** | _(none)_ | `#workflow` |
| **Engine** | Rhai AST | Step engine + PostgreSQL |
| **HEAR behavior** | Blocks a thread | Suspends to DB, zero threads |
| **Server restart** | Loses position | Resumes exact step |
| **Side-effect re-run** | ❌ possible on crash | ✅ never |
| **Multi-day flows** | ❌ (1h timeout) | ✅ unlimited |
| **FOR EACH loops** | ✅ | ❌ |
| **FUNCTION / SUB** | ✅ | ❌ |
| **USE WEBSITE** | ✅ | ❌ |
| **Startup time** | ~1ms | ~2ms |
| **RAM per session** | 1 thread (~64KB) | 0 threads |
| **Observability** | Logs only | DB rows (queryable) |
| **Best for** | Tools, short dialogs | Multi-step dialogs, tickets, approvals |

---

## RUNTIME Mode (default)

Every `.bas` file without `#workflow` runs in RUNTIME mode. The script compiles to a Rhai AST and executes in a `spawn_blocking` thread. `HEAR` blocks the thread until the user replies (up to `hear-timeout-secs`, default 3600).

```basic
' ticket.bas — RUNTIME mode (no pragma)
TALK "Describe the issue"
HEAR description          ' blocks thread, waits
SET ticket = CREATE(description)
TALK "Ticket #{ticket} created"
```

**When to use:** Tool scripts called by LLM, short dialogs (< 10 minutes), scripts using `FOR EACH`, `FUNCTION`, or `USE WEBSITE`.

---

## WORKFLOW Mode ⚗️

> **Status: Planned feature** — see `botserver/WORKFLOW_PLAN.md`

Add `#workflow` as the first line. The compiler produces a `Vec<Step>` instead of a Rhai AST. Each step is persisted to `workflow_executions` in PostgreSQL before execution. On `HEAR`, the engine saves state and returns — no thread held. On the next user message, execution resumes from the exact step.

```basic
#workflow
' ticket.bas — WORKFLOW mode
TALK "Describe the issue"
HEAR description          ' saves state, returns, zero threads
SET ticket = CREATE(description)
TALK "Ticket #{ticket} created"
```

**When to use:** Multi-step dialogs, ticket creation, approval flows, anything that may span minutes or days.

### Keyword compatibility in WORKFLOW mode

| Category | Keywords | WORKFLOW support |
|----------|----------|-----------------|
| **Dialog** | `TALK`, `HEAR`, `WAIT` | ✅ |
| **Data** | `SET`, `GET`, `FIND`, `SAVE`, `INSERT`, `UPDATE`, `DELETE` | ✅ |
| **Communication** | `SEND MAIL`, `SEND TEMPLATE`, `SMS` | ✅ |
| **AI** | `USE KB`, `USE TOOL`, `REMEMBER`, `THINK KB` | ✅ |
| **HTTP** | `GET` (http), `POST`, `PUT`, `PATCH`, `DELETE` (http) | ✅ |
| **Scheduling** | `SCHEDULE`, `BOOK`, `CREATE TASK` | ✅ |
| **Expressions** | `FORMAT`, math, datetime, string functions | ✅ (via Rhai eval) |
| **Control flow** | `IF/ELSE/END IF` | ✅ |
| **Loops** | `FOR EACH / NEXT` | ❌ use RUNTIME |
| **Procedures** | `FUNCTION`, `SUB`, `CALL` | ❌ use RUNTIME |
| **Browser** | `USE WEBSITE` | ❌ use RUNTIME |
| **Events** | `ON EMAIL`, `ON CHANGE`, `WEBHOOK` | ❌ use RUNTIME |

### How WORKFLOW compiles

The compiler does **not** use Rhai for workflow mode. It is a line-by-line parser:

```
TALK "Hello ${name}"   →  Step::Talk { template: "Hello ${name}" }
HEAR description        →  Step::Hear { var: "description", type: "any" }
SET x = score + 1       →  Step::Set  { var: "x", expr: "score + 1" }
IF score > 10 THEN      →  Step::If   { cond: "score > 10", then_steps, else_steps }
SEND MAIL to, s, body   →  Step::SendMail { to, subject, body }
```

Expressions (`score + 1`, `score > 10`) are stored as strings and evaluated at runtime using Rhai as a pure expression calculator — no custom syntax, no side effects.

### Observability

In WORKFLOW mode, every step is a DB row. You can query execution state directly:

```sql
SELECT script_path, current_step, state_json, status, updated_at
FROM workflow_executions
WHERE session_id = '<session-uuid>'
ORDER BY updated_at DESC;
```

---

## Choosing a Mode

```
Does the script use FOR EACH, FUNCTION, or USE WEBSITE?
  YES → RUNTIME (no pragma)

Does the script have HEAR and may run for > 1 hour?
  YES → WORKFLOW (#workflow)

Is it a tool script called by LLM (short, no HEAR)?
  YES → RUNTIME (no pragma)

Is it a multi-step dialog (ticket, approval, enrollment)?
  YES → WORKFLOW (#workflow)  ⚗️ when available
```
