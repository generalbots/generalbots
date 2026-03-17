# BASIC Workflow Engine Plan

## Current State

- `workflow_executions` and `workflow_events` tables exist in DB
- `WorkflowExecution` / `WorkflowEvent` models exist in `core/shared/models/workflow_models.rs`
- `ORCHESTRATE WORKFLOW` keyword exists in `basic/keywords/orchestration.rs` (stub)
- `STEP` keyword registered but not durable
- Compiler (`basic/mod.rs`) produces Rhai AST and runs it in one shot via `engine.eval_ast_with_scope`
- `HEAR` currently blocks a thread (new) — works but not crash-safe

---

## Goal

BASIC scripts run as **durable step sequences**. Each keyword is a step. On crash/restart, execution resumes from the last completed step. No re-run. No Rhai for control flow.

```basic
' ticket.bas
TALK "Describe the issue"       ← Step 1
HEAR description                ← Step 2 (suspends, waits)
SET ticket = CREATE(description) ← Step 3
TALK "Ticket #{ticket} created" ← Step 4
```

---

## Two Execution Modes

The compiler serves both modes via a pragma at the top of the `.bas` file:

```basic
' Default: Rhai mode (current behavior, fast, no durability)
TALK "Hello"

' Workflow mode (durable, crash-safe)
#workflow
TALK "Hello"
HEAR name
```

`ScriptService::compile()` detects `#workflow` and returns either:
- `ExecutionPlan::Rhai(AST)` — current path, unchanged
- `ExecutionPlan::Workflow(Vec<Step>)` — new path

`ScriptService::run()` dispatches accordingly.

---

## Architecture

### 1. Compiler changes (`basic/mod.rs`, `basic/compiler/`)

Add `compile_to_steps(script: &str) -> Result<Vec<Step>>`:

```rust
pub enum Step {
    Talk { template: String },
    Hear { variable: String, input_type: String },
    Set  { variable: String, expression: String },
    If   { condition: String, then_steps: Vec<Step>, else_steps: Vec<Step> },
    Call { function: String, args: Vec<String>, result_var: Option<String> },
    // ... one variant per keyword
}
```

Expressions inside steps (`condition`, `expression`, `template`) are still evaluated by Rhai — but only as **pure expression evaluator**, no custom syntax, no side effects. This keeps Rhai as a math/string engine only.

### 2. WorkflowEngine (`basic/workflow/engine.rs`)

```rust
pub struct WorkflowEngine {
    state: Arc<AppState>,
    session: UserSession,
}

impl WorkflowEngine {
    /// Start a new workflow or resume existing one for this session+script
    pub async fn run(&self, script_path: &str, steps: Vec<Step>) -> Result<()>

    /// Execute one step, persist result, return next action
    async fn execute_step(&self, exec_id: Uuid, step: &Step, vars: &mut Variables) -> StepResult

    /// Load execution state from DB
    async fn load_state(&self, exec_id: Uuid) -> (usize, Variables)

    /// Persist step completion
    async fn save_state(&self, exec_id: Uuid, step_index: usize, vars: &Variables, status: &str)
}

pub enum StepResult {
    Continue,           // go to next step
    Suspend,            // HEAR — save state, return, wait for next message
    Done,               // script finished
}
```

### 3. HEAR in workflow mode

No thread blocking. Instead:

1. `execute_step(Hear)` saves state to `workflow_executions` with `status = "waiting"`, `current_step = N`
2. Returns `StepResult::Suspend` → engine returns to caller
3. Next user message → `stream_response` checks `workflow_executions` for `session_id` with `status = "waiting"`
4. Loads variables, sets `variables["description"] = user_input`, advances `current_step`, resumes

### 4. `stream_response` dispatch (`core/bot/mod.rs`)

```rust
// At top of stream_response, before LLM:
if let Some(exec) = WorkflowEngine::find_waiting(state, session_id).await {
    WorkflowEngine::resume(state, exec, message_content).await?;
    return Ok(());
}
```

### 5. DB schema (already exists, minor additions)

```sql
-- Already exists:
workflow_executions (id, bot_id, workflow_name, current_step, state_json, status, ...)

-- Add:
ALTER TABLE workflow_executions ADD COLUMN session_id UUID;
ALTER TABLE workflow_executions ADD COLUMN script_path TEXT;
-- state_json stores: { "variables": {...}, "step_index": N }
```

---

## Migration Path

### Phase 1 — Parallel mode (no breaking changes)
- Add `compile_to_steps()` alongside existing `compile()`
- Add `WorkflowEngine` as new struct
- `#workflow` pragma routes to new path
- All existing `.bas` files unchanged, run via Rhai as before

### Phase 2 — Keyword parity
Implement step variants for all keywords used in practice:
`TALK`, `HEAR`, `SET`, `IF/ELSE/END IF`, `CALL` (HTTP, LLM, tool), `SEND MAIL`, `SCHEDULE`

### Phase 3 — Default for new scripts
New `.bas` files default to workflow mode. Rhai mode kept for backwards compat and tool scripts (short-lived, no HEAR).

### Phase 4 — Rhai scope reduction
Remove Rhai custom syntax registrations. Keep Rhai only as expression evaluator:
```rust
engine.eval_expression::<Dynamic>(&expr, &scope)
```

---

## File Map

```
basic/
  mod.rs                    ← add compile_to_steps(), ExecutionPlan enum
  compiler/
    mod.rs                  ← existing Rhai compiler, unchanged
    step_compiler.rs        ← NEW: BASIC → Vec<Step>
  workflow/
    mod.rs                  ← NEW: WorkflowEngine
    engine.rs               ← NEW: execute_step, load/save state
    variables.rs            ← NEW: Variables (HashMap<String, Dynamic>)
    steps.rs                ← NEW: Step enum
  keywords/                 ← existing, unchanged in Phase 1
```

---

## Keyword Compatibility

### Category A — Workflow steps (implement as `Step` variants)
`TALK`, `HEAR`, `SET`, `IF/ELSE/END IF`, `SEND MAIL`, `SEND TEMPLATE`, `SCHEDULE`,
`SAVE`/`INSERT`/`UPDATE`, `GET`, `FIND`, `SEARCH`, `USE KB`, `USE TOOL`, `REMEMBER`,
`HTTP GET/POST/PUT/DELETE`, `WAIT`, `TRANSFER TO HUMAN`, `CREATE TASK`, `BOOK`, `SCORE LEAD`

### Category B — Pure expressions (Rhai as calculator, no step boundary)
`math/*`, `datetime/*`, `string_functions`, `arrays/*`, `core_functions`, `validation/*`, `FORMAT`
→ Stored as expression strings in Step, evaluated at runtime via `engine.eval_expression_with_scope()`

### Category C — Rhai-only (scripts using these stay in Rhai mode, no `#workflow`)
`code_sandbox`, `use_website`, `face_api`, `on_change`, `on_email`, `webhook`,
`procedures` (FUNCTION/SUB/CALL), `for_next` (FOR EACH loops), `switch_case`, `events`, `orchestration`

A script with any Category C keyword cannot use `#workflow`. The compiler detects this and errors early.

---

## How Compilation Works Without Rhai

Workflow compiler is a **line-by-line parser**, not a Rhai AST walk:

```
Input line              → Step variant
─────────────────────────────────────────────────────
TALK "Hello ${name}"   → Step::Talk { template }
HEAR description        → Step::Hear { var, input_type }
SET x = score + 1       → Step::Set  { var, expr: "score + 1" }
IF score > 10 THEN      → Step::If   { cond: "score > 10", then_steps, else_steps }
SEND MAIL to, s, b      → Step::SendMail { to, subject, body }
USE TOOL path           → Step::UseTool { path, args }
```

Expressions (`score + 1`, `score > 10`) are stored as **raw strings** in the Step struct.
At runtime, Rhai evaluates them as pure expressions — no custom syntax, no side effects:

```rust
let mut engine = Engine::new(); // no register_custom_syntax calls
let mut scope = Scope::new();
for (k, v) in &variables { scope.push_dynamic(k, v.clone()); }
let result = engine.eval_expression_with_scope::<Dynamic>(&mut scope, expr)?;
```

Rhai remains a dependency but is used only as a math/string expression evaluator (~5 lines of code at runtime). All custom keyword machinery is bypassed entirely.

---



| Engine | Lang | Latency | RAM | Rust SDK | Verdict |
|--------|------|---------|-----|----------|---------|
| **Custom (this plan)** | Rust | ~1ms | 0 extra | Native | ✅ Best fit |
| **[Restate](https://restate.dev)** | Rust server | ~5ms | ~50MB | ✅ official | Fallback option |
| **[Rhythm](https://github.com/maxnorth/rhythm)** | Rust | ~2ms | ~10MB | Native | Experimental |
| **Temporal** | Go+Java | ~20ms | ~500MB | ❌ | Too heavy |
| **Windmill** | Rust+TS | ~10ms | ~200MB | ❌ | Wrong abstraction |

**Why custom over Restate:** Restate requires its own server as a proxy between HTTP requests and handlers — adds a network hop and an extra process. The custom plan uses PostgreSQL already running in the stack, zero extra infrastructure.

**Escape hatch:** The `Step` enum in this plan maps 1:1 to Restate workflow steps. If the custom engine proves too complex to maintain, migration to Restate is mechanical — swap `WorkflowEngine::execute_step` internals, keep the compiler and Step enum unchanged.

---



- **No re-run ever.** Steps before current_step are skipped on resume.
- **Rhai never removed entirely** — used for expression eval only.
- **Backwards compatible** — no `#workflow` = Rhai mode, existing bots unaffected.
- **HEAR in workflow mode = zero threads held.** State in DB, not RAM.
- **Tool scripts** (called by LLM) stay in Rhai mode — they're short-lived, no HEAR needed.
