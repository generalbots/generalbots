# Auto Task Architecture - LLM-to-BASIC Intent Compiler

## Overview

The Auto Task system is a revolutionary approach to task automation that translates natural language intents into executable BASIC programs. This document describes the complete architecture for the "Premium VIP Mode" intelligent task execution system.

## Core Components

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                           USER INTERFACE                                     │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │  Natural Language Intent Input                                        │   │
│  │  "Make a financial CRM for Deloitte with client tracking & reports"  │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                         INTENT COMPILER                                      │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ │
│  │ Entity        │→│ Plan          │→│ BASIC Code    │→│ Risk        │ │
│  │ Extraction    │  │ Generation    │  │ Generation    │  │ Assessment  │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                          SAFETY LAYER                                        │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ │
│  │ Constraint    │  │ Impact        │  │ Approval      │  │ Audit       │ │
│  │ Checker       │  │ Simulator     │  │ Workflow      │  │ Trail       │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                        EXECUTION ENGINE                                      │
│  ┌───────────────┐  ┌───────────────┐  ┌───────────────┐  ┌─────────────┐ │
│  │ BASIC         │  │ MCP Client    │  │ API Gateway   │  │ State       │ │
│  │ Interpreter   │  │ (MCP Servers) │  │ (External)    │  │ Manager     │ │
│  └───────────────┘  └───────────────┘  └───────────────┘  └─────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

## 1. Intent Compiler (`intent_compiler.rs`)

The Intent Compiler is the "magic" component that translates natural language into executable BASIC programs.

### Flow

1. **Entity Extraction** - Uses LLM to extract:
   - Action (create, update, delete, analyze)
   - Target (CRM, website, report, API)
   - Domain (financial, healthcare, retail)
   - Client name
   - Features requested
   - Constraints (budget, timeline, technology)
   - Required integrations

2. **Plan Generation** - Creates an execution plan with:
   - Ordered steps
   - Dependencies between steps
   - Resource estimates
   - Risk assessment per step
   - Required approvals

3. **BASIC Code Generation** - Generates executable code using:
   - Existing 80+ keywords
   - New safety keywords (REQUIRE_APPROVAL, SIMULATE_IMPACT)
   - New MCP keywords (USE_MCP, MCP_INVOKE)

### Example Transformation

**Input:**
```
"Make a financial CRM for Deloitte with client management and reporting"
```

**Generated BASIC Program:**
```basic
' =============================================================================
' AUTO-GENERATED BASIC PROGRAM
' Plan: Financial CRM for Deloitte
' Generated: 2024-01-15 10:30:00
' =============================================================================

PLAN_START "Financial CRM for Deloitte", "Client management and reporting system"
  STEP 1, "Create database schema", CRITICAL
  STEP 2, "Setup authentication", HIGH
  STEP 3, "Build client management module", HIGH
  STEP 4, "Build financial tracking", MEDIUM
  STEP 5, "Create reporting dashboard", MEDIUM
  STEP 6, "Deploy and test", LOW
PLAN_END

' Initialize context
SET action = "create"
SET target = "CRM"
SET client = "Deloitte"
SET domain = "financial"

SET CONTEXT "Building a financial CRM for Deloitte with client management and reporting"

' -----------------------------------------------------------------------------
' STEP 1: Create database schema
' -----------------------------------------------------------------------------
REQUIRE_APPROVAL "create-database", "Creating database will incur monthly costs"
IF approved THEN
  AUDIT_LOG "step-start", "step-1", "Create database schema"
  
  schema = LLM "Generate PostgreSQL schema for financial CRM with clients, contacts, deals, and reports tables"
  
  USE_MCP "database", "execute_sql", {"sql": schema}
  
  AUDIT_LOG "step-complete", "step-1", "Database created"
END IF

' -----------------------------------------------------------------------------
' STEP 2: Setup authentication
' -----------------------------------------------------------------------------
AUDIT_LOG "step-start", "step-2", "Setup authentication"

RUN_PYTHON "
from auth_setup import configure_oauth
configure_oauth(provider='azure_ad', tenant='deloitte.com')
"

AUDIT_LOG "step-complete", "step-2", "Authentication configured"

' Continue with remaining steps...

TALK "Financial CRM for Deloitte has been created successfully!"
AUDIT_LOG "plan-complete", "financial-crm-deloitte", "success"
```

## 2. Safety Layer (`safety_layer.rs`)

The Safety Layer ensures all actions are validated before execution.

### Components

#### Constraint Checker
- **Budget constraints** - Check estimated costs against limits
- **Permission constraints** - Verify user has required access
- **Policy constraints** - Enforce organizational rules
- **Compliance constraints** - Ensure regulatory compliance
- **Technical constraints** - Validate system capabilities
- **Rate limits** - Prevent resource exhaustion

#### Impact Simulator
Performs dry-run execution to predict:
- Data changes (records created/modified/deleted)
- Cost impact (API calls, compute, storage)
- Time impact (execution duration, blocking)
- Security impact (credentials accessed, external systems)
- Side effects (unintended consequences)

#### Approval Workflow
Multi-level approval system:
- Plan-level approval
- Step-level approval for high-risk actions
- Override approval for constraint violations
- Timeout handling with configurable defaults

#### Audit Trail
Complete logging of:
- All task lifecycle events
- Step execution details
- Approval decisions
- Data modifications
- API calls
- Error conditions

## 3. Execution Engine

### BASIC Interpreter
Executes the generated BASIC programs with:
- State management across steps
- Error handling and recovery
- Rollback support for reversible actions
- Progress tracking and reporting

### MCP Client (`mcp_client.rs`)
Integrates with Model Context Protocol servers:

**Supported Server Types:**
- Database Server (PostgreSQL, MySQL, SQLite)
- Filesystem Server (local, cloud storage)
- Web Server (HTTP/REST APIs)
- Email Server (SMTP/IMAP)
- Slack/Teams Server
- Analytics Server
- Custom servers

**Example MCP Usage:**
```basic
' Query database via MCP
result = USE_MCP "database", "query", {"sql": "SELECT * FROM clients WHERE status = 'active'"}

' Send Slack message
USE_MCP "slack", "send_message", {"channel": "#sales", "text": "New client added!"}

' Upload to S3
USE_MCP "storage", "upload", {"bucket": "reports", "key": "q4-report.pdf", "file": pdf_data}
```

### API Gateway
Handles external API integrations:
- Authentication (API Key, OAuth2, Basic)
- Rate limiting and retry logic
- Response transformation
- Error handling

## 4. Decision Framework

When the Intent Compiler detects ambiguity, it generates options:

```
┌─────────────────────────────────────────────────────────────────┐
│                    DECISION REQUIRED                             │
│                                                                  │
│  Your intent could be interpreted multiple ways:                 │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ OPTION A: Full Custom CRM                      [RECOMMENDED]  │
│  │ Build from scratch with all requested features            │   │
│  │ ✅ Maximum flexibility                                    │   │
│  │ ✅ Exactly matches requirements                           │   │
│  │ ❌ Higher cost (~$500)                                    │   │
│  │ ❌ Longer timeline (~2 weeks)                             │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │ OPTION B: Extend Existing Template                        │   │
│  │ Use CRM template and customize                            │   │
│  │ ✅ Lower cost (~$100)                                     │   │
│  │ ✅ Faster delivery (~3 days)                              │   │
│  │ ❌ Some limitations on customization                      │   │
│  │ ❌ May not fit all requirements                           │   │
│  └─────────────────────────────────────────────────────────┘   │
│                                                                  │
│  [Select Option A]  [Select Option B]  [Provide More Details]   │
└─────────────────────────────────────────────────────────────────┘
```

## 5. Auto Task Lifecycle

```
┌──────────┐   ┌───────────┐   ┌────────────────┐   ┌───────────┐
│  DRAFT   │──▶│ COMPILING │──▶│ PENDING_APPROVAL│──▶│ SIMULATING│
└──────────┘   └───────────┘   └────────────────┘   └───────────┘
                                                          │
┌──────────┐   ┌───────────┐   ┌─────────┐              │
│COMPLETED │◀──│  RUNNING  │◀──│  READY  │◀─────────────┘
└──────────┘   └───────────┘   └─────────┘
      ▲              │               │
      │              ▼               ▼
      │        ┌──────────┐   ┌────────────────┐
      │        │  PAUSED  │   │WAITING_DECISION│
      │        └──────────┘   └────────────────┘
      │              │
      │              ▼
      │        ┌──────────┐   ┌───────────┐
      └────────│ BLOCKED  │   │  FAILED   │
               └──────────┘   └───────────┘
```

## 6. New Keywords Added

### Safety Keywords
| Keyword | Description | Example |
|---------|-------------|---------|
| `REQUIRE_APPROVAL` | Request human approval | `REQUIRE_APPROVAL "action-id", "reason"` |
| `SIMULATE_IMPACT` | Simulate before execute | `result = SIMULATE_IMPACT "step-id"` |
| `CHECK_CONSTRAINTS` | Validate constraints | `CHECK_CONSTRAINTS "budget", 1000` |
| `AUDIT_LOG` | Log to audit trail | `AUDIT_LOG "event", "id", "details"` |

### MCP Keywords
| Keyword | Description | Example |
|---------|-------------|---------|
| `USE_MCP` | Invoke MCP server tool | `USE_MCP "server", "tool", {params}` |
| `MCP_LIST_TOOLS` | List available tools | `tools = MCP_LIST_TOOLS "server"` |
| `MCP_INVOKE` | Direct tool invocation | `MCP_INVOKE "server.tool", params` |

### Auto Task Keywords
| Keyword | Description | Example |
|---------|-------------|---------|
| `PLAN_START` | Begin plan declaration | `PLAN_START "name", "description"` |
| `PLAN_END` | End plan declaration | `PLAN_END` |
| `STEP` | Declare a step | `STEP 1, "name", PRIORITY` |
| `AUTO_TASK` | Create auto-executing task | `AUTO_TASK "intent"` |

### Decision Keywords
| Keyword | Description | Example |
|---------|-------------|---------|
| `OPTION_A_OR_B` | Present options | `OPTION_A_OR_B optA, optB, "question"` |
| `DECIDE` | Get decision result | `choice = DECIDE "decision-id"` |
| `ESCALATE` | Escalate to human | `ESCALATE "reason", assignee` |

## 7. Database Schema

```sql
-- Auto Tasks
CREATE TABLE auto_tasks (
    id UUID PRIMARY KEY,
    title TEXT NOT NULL,
    intent TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'draft',
    execution_mode TEXT NOT NULL DEFAULT 'semi-automatic',
    priority TEXT NOT NULL DEFAULT 'medium',
    plan_id UUID REFERENCES execution_plans(id),
    basic_program TEXT,
    current_step INTEGER DEFAULT 0,
    total_steps INTEGER DEFAULT 0,
    progress FLOAT DEFAULT 0,
    risk_level TEXT,
    session_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    created_by TEXT NOT NULL,
    assigned_to TEXT DEFAULT 'auto',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ
);

-- Execution Plans
CREATE TABLE execution_plans (
    id UUID PRIMARY KEY,
    name TEXT NOT NULL,
    description TEXT,
    steps JSONB NOT NULL,
    dependencies JSONB,
    estimated_duration_minutes INTEGER,
    requires_approval BOOLEAN DEFAULT FALSE,
    rollback_plan TEXT,
    compiled_intent_id UUID,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Step Results
CREATE TABLE step_results (
    id UUID PRIMARY KEY,
    task_id UUID REFERENCES auto_tasks(id),
    step_id TEXT NOT NULL,
    step_order INTEGER NOT NULL,
    status TEXT NOT NULL,
    output JSONB,
    error TEXT,
    started_at TIMESTAMPTZ,
    completed_at TIMESTAMPTZ,
    duration_ms BIGINT,
    rollback_data JSONB
);

-- Pending Decisions
CREATE TABLE pending_decisions (
    id UUID PRIMARY KEY,
    task_id UUID REFERENCES auto_tasks(id),
    decision_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    options JSONB NOT NULL,
    default_option TEXT,
    timeout_seconds INTEGER,
    context JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    resolved_at TIMESTAMPTZ,
    chosen_option TEXT
);

-- Pending Approvals
CREATE TABLE pending_approvals (
    id UUID PRIMARY KEY,
    task_id UUID REFERENCES auto_tasks(id),
    approval_type TEXT NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    risk_level TEXT,
    step_id TEXT,
    impact_summary TEXT,
    simulation_result JSONB,
    approver TEXT,
    timeout_seconds INTEGER DEFAULT 3600,
    default_action TEXT DEFAULT 'pause',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    expires_at TIMESTAMPTZ,
    decided_at TIMESTAMPTZ,
    decision TEXT,
    decided_by TEXT,
    comments TEXT
);

-- MCP Servers
CREATE TABLE mcp_servers (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    name TEXT NOT NULL,
    description TEXT,
    server_type TEXT NOT NULL,
    config JSONB NOT NULL,
    auth JSONB,
    tools JSONB,
    status TEXT DEFAULT 'inactive',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(bot_id, name)
);

-- Safety Constraints
CREATE TABLE safety_constraints (
    id UUID PRIMARY KEY,
    bot_id UUID NOT NULL,
    name TEXT NOT NULL,
    constraint_type TEXT NOT NULL,
    description TEXT,
    expression TEXT,
    threshold JSONB,
    severity TEXT DEFAULT 'warning',
    enabled BOOLEAN DEFAULT TRUE,
    applies_to TEXT[],
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Audit Log
CREATE TABLE audit_log (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ DEFAULT NOW(),
    event_type TEXT NOT NULL,
    actor_type TEXT NOT NULL,
    actor_id TEXT NOT NULL,
    action TEXT NOT NULL,
    target_type TEXT,
    target_id TEXT,
    outcome_success BOOLEAN,
    outcome_message TEXT,
    details JSONB,
    session_id UUID,
    bot_id UUID,
    task_id UUID,
    step_id TEXT,
    risk_level TEXT,
    auto_executed BOOLEAN DEFAULT FALSE
);

CREATE INDEX idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX idx_audit_log_task_id ON audit_log(task_id);
CREATE INDEX idx_audit_log_event_type ON audit_log(event_type);
```

## 8. API Endpoints

### Intent Compilation
- `POST /api/autotask/compile` - Compile intent to plan
- `POST /api/autotask/simulate/:plan_id` - Simulate plan execution

### Task Management
- `GET /api/autotask/list` - List auto tasks
- `GET /api/autotask/stats` - Get task statistics
- `POST /api/autotask/execute` - Execute a compiled plan

### Task Actions
- `POST /api/autotask/:task_id/pause` - Pause task
- `POST /api/autotask/:task_id/resume` - Resume task
- `POST /api/autotask/:task_id/cancel` - Cancel task
- `POST /api/autotask/:task_id/simulate` - Simulate task

### Decisions & Approvals
- `GET /api/autotask/:task_id/decisions` - Get pending decisions
- `POST /api/autotask/:task_id/decide` - Submit decision
- `GET /api/autotask/:task_id/approvals` - Get pending approvals
- `POST /api/autotask/:task_id/approve` - Submit approval

## 9. Security Considerations

1. **Sandboxed Execution** - All code runs in isolated containers
2. **Credential Management** - No hardcoded secrets, use references
3. **Rate Limiting** - Prevent runaway executions
4. **Audit Trail** - Complete logging for compliance
5. **Approval Workflow** - Human oversight for high-risk actions
6. **Rollback Support** - Undo mechanisms where possible
7. **Circuit Breaker** - Stop on repeated failures

## 10. Future Enhancements

- [ ] Visual plan editor
- [ ] Real-time collaboration
- [ ] Plan templates marketplace
- [ ] Advanced scheduling (cron-like)
- [ ] Cost optimization suggestions
- [ ] Multi-tenant isolation
- [ ] Federated MCP servers
- [ ] AI-powered plan optimization
- [ ] Natural language debugging
- [ ] Integration with external task systems (Jira, Asana, etc.)

---

## File Structure

```
botserver/src/basic/keywords/
├── intent_compiler.rs      # LLM-to-BASIC translation
├── auto_task.rs           # Auto task data structures
├── autotask_api.rs        # HTTP API handlers
├── mcp_client.rs          # MCP server integration
├── safety_layer.rs        # Constraints, simulation, audit
└── mod.rs                 # Module exports & keyword list

botui/ui/suite/tasks/
├── autotask.html          # Auto task UI
├── autotask.css           # Styles
├── autotask.js            # Client-side logic
├── tasks.html             # Original task list
├── tasks.css
└── tasks.js
```

---

*This architecture enables the "Premium VIP Mode" where users can describe what they want in natural language and the system automatically generates, validates, and executes the required tasks with full safety controls and audit trail.*