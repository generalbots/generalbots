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

## Key Invariants

- **No re-run ever.** Steps before current_step are skipped on resume.
- **Rhai never removed entirely** — used for expression eval only.
- **Backwards compatible** — no `#workflow` = Rhai mode, existing bots unaffected.
- **HEAR in workflow mode = zero threads held.** State in DB, not RAM.
- **Tool scripts** (called by LLM) stay in Rhai mode — they're short-lived, no HEAR needed.
