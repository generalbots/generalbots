# General Bots 7.0 - Enhanced Multi-Agent Orchestration

**Target Release:** Q3 2026  
**Current Version:** 6.2.0  
**Priority:** Critical for enterprise adoption

---

## Phase 1: Enhanced Orchestration (Months 1-2) 🚀

### 1.1 ORCHESTRATE WORKFLOW Keyword
- [x] **File:** `src/basic/keywords/orchestration.rs`
- [x] Add `ORCHESTRATE WORKFLOW` keyword to BASIC interpreter
- [x] Support STEP definitions with BOT calls
- [x] Support PARALLEL branches execution
- [x] Support conditional IF/THEN logic in workflows
- [x] Variable passing between steps
- [x] **Database:** Add `workflow_executions` table
- [x] **Test:** Create workflow execution tests

### 1.2 Event Bus System
- [x] **File:** `src/basic/keywords/events.rs`
- [x] Add `ON EVENT` keyword for event handlers
- [x] Add `PUBLISH EVENT` keyword for event emission
- [x] Add `WAIT FOR EVENT` with timeout support
- [x] **Integration:** Use existing Redis pub/sub
- [x] **Database:** Add `workflow_events` table
- [x] **Test:** Event-driven workflow tests

### 1.3 Enhanced Bot Memory
- [x] **File:** `src/basic/keywords/enhanced_memory.rs`
- [x] Add `BOT SHARE MEMORY` for cross-bot memory sharing
- [x] Add `BOT SYNC MEMORY` for memory synchronization
- [x] **Integration:** Extend existing `SET BOT MEMORY` and bot_memories table
- [x] **Test:** Cross-bot memory sharing tests

### 1.4 Database Schema
```sql
-- Add to migrations/
CREATE TABLE workflow_executions (
  id UUID PRIMARY KEY,
  bot_id UUID REFERENCES bots(id),
  workflow_name TEXT,
  current_step INTEGER,
  state JSONB,
  status TEXT DEFAULT 'running',
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE workflow_events (
  id UUID PRIMARY KEY,
  workflow_id UUID REFERENCES workflow_executions(id),
  event_name TEXT,
  event_data JSONB,
  created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE bot_shared_memory (
  id UUID PRIMARY KEY,
  source_bot_id UUID REFERENCES bots(id),
  target_bot_id UUID REFERENCES bots(id),
  memory_key TEXT,
  memory_value TEXT,
  shared_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## Phase 2: Visual Workflow Designer (Months 3-4) 🎨

### 2.1 Drag-and-Drop Canvas
- [x] **File:** `src/designer/workflow_canvas.rs`
- [x] Extend existing designer with workflow nodes
- [x] Add node types: BotAgent, HumanApproval, Condition, Loop, Parallel
- [x] Drag-and-drop interface using existing HTMX
- [x] **Frontend:** Add workflow canvas to existing designer UI
- [x] **Output:** Generate BASIC code from visual design

### 2.2 Bot Templates
- [x] **Directory:** `bottemplates/` (not templates/)
- [x] Create pre-built workflow `.gbai` packages
- [x] Customer support escalation template
- [x] E-commerce order processing template
- [x] Content moderation template
- [x] **Integration:** Auto-discovery via existing package system

### 2.3 Visual Designer Enhancement
- [x] **File:** `src/designer/mod.rs`
- [x] Add workflow mode to existing designer
- [x] Real-time BASIC code preview
- [x] Workflow validation and error checking
- [x] **Test:** Visual designer workflow tests

---

## Phase 3: Intelligence & Learning (Months 5-6) 🧠

### 3.1 Smart LLM Routing
- [x] **File:** `src/llm/smart_router.rs`
- [x] Extend existing `llm/observability.rs`
- [x] Add cost/latency tracking per model
- [x] Automatic model selection based on task type
- [x] **BASIC:** Enhance LLM keyword with OPTIMIZE FOR parameter
- [x] **Database:** Add `model_performance` table
- [x] **Test:** LLM routing optimization tests

### 3.2 Enhanced Memory System
- [x] **File:** `src/bots/memory.rs`
- [x] Cross-bot memory sharing mechanisms
- [x] Memory synchronization between bots
- [x] **Integration:** Use existing bot_memories table + new sharing table
- [x] **Test:** Memory sharing behavior tests

### 3.3 Enhanced BASIC Keywords
```basic
' New keywords to implement
result = LLM "Analyze data" WITH OPTIMIZE FOR "speed"
BOT SHARE MEMORY "customer_preferences" WITH "support-bot-2"
BOT SYNC MEMORY FROM "master-bot"
```

---

---

## Implementation Guidelines

### Code Standards
- [x] **No breaking changes** - all existing `.gbai` packages must work
- [x] **Extend existing systems** - don't rebuild what works
- [x] **BASIC-first design** - everything accessible via BASIC keywords
- [x] **Use existing infrastructure** - PostgreSQL, Redis, Qdrant, LXC
- [x] **Proper error handling** - no unwrap(), use SafeCommand wrapper

### Testing Requirements
- [x] **Unit tests** for all new BASIC keywords
- [x] **Integration tests** for workflow execution
- [x] **Performance tests** for multi-agent coordination
- [x] **Backward compatibility tests** for existing `.gbai` packages

### Documentation Updates
- [x] **File:** `docs/reference/basic-language.md` - Add new keywords
- [x] **File:** `docs/guides/workflows.md` - Workflow creation guide
- [x] **File:** `docs/guides/multi-agent.md` - Multi-agent patterns
- [x] **File:** `docs/api/workflow-api.md` - Workflow REST endpoints

---

## File Structure Changes

```
src/
├── basic/keywords/
│   ├── orchestration.rs          # NEW: ORCHESTRATE WORKFLOW
│   ├── events.rs                 # NEW: ON EVENT, PUBLISH EVENT
│   └── enhanced_memory.rs        # NEW: BOT SHARE/SYNC MEMORY
├── designer/
│   ├── workflow_canvas.rs        # NEW: Visual workflow editor
│   └── mod.rs                    # EXTEND: Add workflow mode
├── llm/
│   └── smart_router.rs           # NEW: Intelligent model routing
├── bots/
│   └── memory.rs                 # NEW: Enhanced memory system
└──

bottemplates/                     # NEW: Pre-built workflows
├── customer-support.gbai/
├── order-processing.gbai/
└── content-moderation.gbai/

docs/
├── guides/
│   ├── tools-vs-bots.md         # DONE: Tool vs Bot explanation
│   ├── workflows.md             # NEW: Workflow creation
│   └── multi-agent.md           # NEW: Multi-agent patterns
└── reference/
    └── basic-language.md         # UPDATE: New keywords
```

---

## Success Metrics

### Technical Metrics
- [x] **Backward compatibility:** 100% existing `.gbai` packages work
- [x] **Performance:** Workflow execution <2s overhead
- [x] **Reliability:** 99.9% workflow completion rate
- [x] **Memory usage:** <10% increase from current baseline

### Business Metrics
- [x] **Workflow creation time:** 50% reduction vs manual coordination
- [x] **Training time:** 80% reduction for non-programmers
- [x] **Enterprise adoption:** 10x faster implementation
- [x] **Community plugins:** 100+ plugins in first 6 months

---

## Risk Mitigation

### Technical Risks
- [x] **Context overflow:** Implement workflow state persistence
- [x] **Bot coordination failures:** Add timeout and retry mechanisms
- [x] **Performance degradation:** Implement workflow step caching
- [x] **Memory leaks:** Proper cleanup of workflow sessions

### Business Risks
- [x] **Breaking changes:** Comprehensive backward compatibility testing
- [x] **Complexity creep:** Keep BASIC-first design principle
- [x] **Performance impact:** Benchmark all new features
- [x] **Security vulnerabilities:** Security review for all plugin systems

---

## Dependencies

### External Dependencies (No New Ones)
- ✅ **Rust 1.75+** - Already required
- ✅ **PostgreSQL** - Already in LXC container
- ✅ **Redis** - Already in LXC container  
- ✅ **Qdrant** - Already in LXC container
- ✅ **Rhai** - Already used for BASIC interpreter

### Internal Dependencies
- ✅ **Existing BASIC interpreter** - Extend with new keywords
- ✅ **Existing bot management** - Use for multi-agent coordination
- ✅ **Existing session system** - Store workflow state
- ✅ **Existing MCP support** - Extend for plugin system

---

## Delivery Timeline

| Phase | Duration | Deliverable | Dependencies |
|-------|----------|-------------|--------------|
| **Phase 1** | 2 months | Enhanced orchestration | None |
| **Phase 2** | 2 months | Visual designer | Phase 1 |
| **Phase 3** | 2 months | Intelligence & learning | Phase 1 |

**Total Duration:** 6 months  
**Target Release:** General Bots 7.0 - Q2 2026

---

## Getting Started

### Immediate Next Steps
1. [x] **Create feature branch:** `git checkout -b feature/orchestration-7.0`
2. [x] **Set up development environment:** Ensure Rust 1.75+, PostgreSQL, Redis
3. [x] **Start with Phase 1.1:** Implement `ORCHESTRATE WORKFLOW` keyword
4. [x] **Create basic test:** Simple 2-step workflow execution
5. [x] **Document progress:** Update this TODO.md as tasks complete

### Development Order
1. **Start with BASIC keywords** - Core functionality first
2. **Add database schema** - Persistence layer
3. **Implement workflow engine** - Execution logic
4. **Add visual designer** - User interface
5. **Enhance with intelligence** - AI improvements


**Remember:** Build on existing systems, don't rebuild. Every new feature should extend what already works in General Bots.
