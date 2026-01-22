# Deep Feature Gating Plan: Schema & Migrations

## Objective
Reduce binary size and runtime overhead by strictly feature-gating Database Schema (`schema.rs`) and Migrations. 
The goal is that a "minimal" build (chat-only) should:
1. NOT compile code for `tasks`, `mail`, `calendar`, etc. tables.
2. NOT run database migrations for those features (preventing unused tables in DB).
3. NOT fail compilation due to missing table definitions in `joinable!` macros.

---

## 🚀 Phase 1: Split Diesel Schema (The Hard Part)

The current `schema.rs` is monolithic (3000+ lines). 
Diesel requires `joinable!` and `allow_tables_to_appear_in_same_query!` macros to know about relationship graphs.

### **1.1 Analysis & grouping**
Identify which tables belong to which feature and strictly define their relationships.
- **Core**: `users`, `bots`, `sessions`, `organizations`, `rbac_*`
- **Tasks**: `tasks`, `task_comments` (links to `users`, `projects`)
- **Mail**: `email_*` (links to `users`)
- **Drive**: (Uses S3, but maybe metadata tables?)
- **People/CRM**: `crm_*` (links to `users`, `organizations`)

### **1.2 Create Modular Schema Files**
Instead of one file, we will separate them and use `mod.rs` to aggregate.
- `src/core/shared/schema/mod.rs` (Aggregator)
- `src/core/shared/schema/core.rs` (Base tables)
- `src/core/shared/schema/tasks.rs` (Gated `#[cfg(feature="tasks")]`)
- `src/core/shared/schema/mail.rs` (Gated `#[cfg(feature="mail")]`)
...etc.

### **1.3 Solve the `allow_tables_to_appear_in_same_query!` Problem**
The single massive macro prevents gating. 
**Solution**: Break the "Universe" assumption.
1. Define a `core_tables!` macro.
2. Define `task_tables!` macro (gated).
3. We might lose the ability to write arbitrary joins between *any* two tables if they are in different clusters, unless we explicitly define `joinable!` in the aggregator.
4. **Strategy**: 
    - Keep `joinable!` definitions relative to the feature.
    - If `TableA` (Core) joins `TableB` (Tasks), the `joinable!` must be gated by `tasks`.
    - The `allow_tables...` output must be constructed dynamically or strict subsets defined.

### **1.4 Comprehensive Feature Mapping**
Based on `Cargo.toml`, we need to map all optional features to their schema requirements.

| Feature | Tables (Prefix/Name) | Migrations |
|---------|-----------------------|------------|
| (core) | `users`, `bots`, `sessions`, `orgs`, `rbac_*`, `system_automations`, `bot_memories`, `kb_*` | `migrations/core/` |
| `tasks` | `tasks`, `task_comments` | `migrations/tasks/` |
| `calendar`| `calendars`, `calendar_events`, `calendar_*` | `migrations/calendar/` |
| `mail` | `email_*`, `distribution_lists`, `global_email_signatures` | `migrations/mail/` |
| `people` | `crm_contacts`, `crm_accounts`, `crm_leads`, `crm_opportunities`, `crm_activities`, `crm_pipeline_*`, `people_*` | `migrations/people/` |
| `compliance`| `compliance_*`, `legal_*`, `cookie_consents`, `data_*` | `migrations/compliance/` |
| `meet` | `meet_*`, `meeting_*`, `whiteboard_*` | `migrations/meet/` |
| `attendant`| `attendant_*` | `migrations/attendant/` |
| `analytics`| `dashboards`, `dashboard_*` | `migrations/analytics/` |
| `drive` | (No direct tables? Check `files` or `drive_*`?) | `migrations/drive/` |
| `billing` | `billing_*`, `products`, `services`, `price_*`, `inventory_*` (See note) | `migrations/billing/` |
| `social` | `social_*` | `migrations/social/` |
| `canvas` | `canvases`, `canvas_*` | `migrations/canvas/` |
| `workspaces`| `workspaces`, `workspace_*` | `migrations/workspaces/` |
| `research` | `research_*` | `migrations/research/` |
| `goals` | `okr_*` | `migrations/goals/` |
| `feedback` | `support_tickets`, `ticket_*` | `migrations/tickets/` |

**Note**: `billing` and `tickets` are currently ungated modules in `main.rs`. We should create features or group them. For now, assume they are part of admin/core or need gating.

---

## 🚀 Phase 2: Feature-Gated Migrations

Currently, `embed_migrations!("migrations")` embeds everything.

### **2.1 Split Migration Directories**
Refactor the flat `migrations/` folder into feature-specific directories (requires custom runner logic or Diesel configuration trickery).
Alternately (easier code-wise):
- Keep flat structure but **edit the migration SQL files** to be conditional? No, SQL doesn't support `#[cfg]`.
- **Better Approach**: Use multiple `embed_migrations!` calls.
    - `migrations/core/` -> Always run.
    - `migrations/tasks/` -> Run if `tasks` feature on.
    - `migrations/mail/` -> Run if `mail` feature on.

**Action Plan**:
1. Organize `migrations/` into subdirectories: `core/`, `tasks/`, `mail/`, `people/`.
2. Update `utils.rs` migration logic:
    ```rust
    pub fn run_migrations(conn: &mut PgConnection) {
        // Core
        const CORE: EmbeddedMigrations = embed_migrations!("migrations/core");
        conn.run_pending_migrations(CORE).unwrap();
        
        #[cfg(feature = "tasks")] {
            const TASKS: EmbeddedMigrations = embed_migrations!("migrations/tasks");
            conn.run_pending_migrations(TASKS).unwrap();
        }
        // ...
    }
    ```

### **2.2 Verify Migration Dependencies**
Ensure that `tasks` migrations (which might foreign-key to `users`) only run *after* `core` migrations are applied.

---

## 🚀 Phase 3: Fix Dependent Code (Strict Gating)

Once tables are gated, any code referencing `schema::tasks` will fail to compile if the feature is off.
(We have already done most of this in Models/Logic, but Schema was the safety net).

### **3.1 Verify Models imports**
Ensure `models/tasks.rs` uses `crate::schema::tasks` inside the same cfg gate.

### **3.2 Fix Cross-Feature Joins**
If Core code joins `users` with `tasks` (e.g. "get all user tasks"), that code chunk MUST be gated.

---

## Execution Checklist

### Schema
- [x] Create `src/core/shared/schema/` directory
- [x] Move core tables to `schema/core.rs`
- [x] Move task tables to `schema/tasks.rs` (gated)
- [x] Move mail tables to `schema/mail.rs` (gated)
- [x] Move people tables to `schema/people.rs` (gated)
- [x] Move tickets tables to `schema/tickets.rs` (gated)
- [x] Move billing tables to `schema/billing.rs` (gated)
- [x] Move attendant tables to `schema/attendant.rs` (gated)
- [x] Move calendar tables to `schema/calendar.rs` (gated)
- [x] Move goals tables to `schema/goals.rs` (gated)
- [x] Move canvas tables to `schema/canvas.rs` (gated)
- [x] Move workspaces tables to `schema/workspaces.rs` (gated)
- [x] Move social tables to `schema/social.rs` (gated)
- [x] Move analytics tables to `schema/analytics.rs` (gated)
- [x] Move compliance tables to `schema/compliance.rs` (gated)
- [x] Move meet tables to `schema/meet.rs` (gated)
- [x] Move research tables to `schema/research.rs` (gated)
- [x] Refactor `schema/mod.rs` to export modules conditionally
- [x] Split `joinable!` declarations into relevant files
- [x] Refactor `allow_tables_to_appear_in_same_query!` (Skipped/Implicit)

### Migrations
- [x] Audit `migrations/` folder
- [x] Create folder structure `migrations/{core,tasks,mail,people...}`
- [x] Move SQL files to appropriate folders
- [x] Update `run_migrations` in `utils.rs` to run feature-specific migration sets

### Validation
- [ ] `cargo check --no-default-features` (Run ONLY after all migration splits are verified)
- [ ] `cargo check --features tasks`
- [ ] `cargo check --all-features`
