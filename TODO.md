# General Bots 7.0 - Enhanced Multi-Agent Orchestration

**Target Release:** Q3 2026  
**Current Version:** 6.1.0  
**Priority:** Critical for enterprise adoption

---

## Phase 1: Enhanced Orchestration (Months 1-2) 🚀

### 1.1 ORCHESTRATE WORKFLOW Keyword
- [ ] **File:** `src/basic/keywords/orchestration.rs`
- [ ] Add `ORCHESTRATE WORKFLOW` keyword to BASIC interpreter
- [ ] Support STEP definitions with BOT calls
- [ ] Support PARALLEL branches execution
- [ ] Support conditional IF/THEN logic in workflows
- [ ] Variable passing between steps
- [ ] **Database:** Add `workflow_executions` table
- [ ] **Test:** Create workflow execution tests

### 1.2 Event Bus System
- [ ] **File:** `src/basic/keywords/events.rs`
- [ ] Add `ON EVENT` keyword for event handlers
- [ ] Add `PUBLISH EVENT` keyword for event emission
- [ ] Add `WAIT FOR EVENT` with timeout support
- [ ] **Integration:** Use existing Redis pub/sub
- [ ] **Database:** Add `workflow_events` table
- [ ] **Test:** Event-driven workflow tests

### 1.3 Bot Learning Enhancement
- [ ] **File:** `src/basic/keywords/bot_learning.rs`
- [ ] Add `BOT LEARN` keyword for pattern storage (extends existing `SET BOT MEMORY`)
- [ ] Add `BOT RECALL` keyword for pattern retrieval (extends existing bot memory)
- [ ] Add `BOT SHARE KNOWLEDGE` for cross-bot learning
- [ ] **Integration:** Use existing VectorDB (Qdrant) + existing bot_memories table
- [ ] **Write-back:** Store learned patterns in `.gbkb` folders for persistence
- [ ] **Test:** Bot learning and recall tests

**Note:** Difference between `SET BOT MEMORY` vs `BOT LEARN`:
- `SET BOT MEMORY`: Manual key-value storage (existing)
- `BOT LEARN`: Automatic pattern recognition from conversations

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

CREATE TABLE bot_knowledge (
  id UUID PRIMARY KEY,
  bot_id UUID REFERENCES bots(id),
  pattern TEXT,
  confidence FLOAT,
  learned_from UUID REFERENCES conversations(id),
  kb_file_path TEXT, -- Path to .gbkb file for persistence
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## Phase 2: Visual Workflow Designer (Months 3-4) 🎨

### 2.1 Drag-and-Drop Canvas
- [ ] **File:** `src/designer/workflow_canvas.rs`
- [ ] Extend existing designer with workflow nodes
- [ ] Add node types: BotAgent, HumanApproval, Condition, Loop, Parallel
- [ ] Drag-and-drop interface using existing HTMX
- [ ] **Frontend:** Add workflow canvas to existing designer UI
- [ ] **Output:** Generate BASIC code from visual design

### 2.2 Bot Templates
- [ ] **Directory:** `bottemplates/` (not templates/)
- [ ] Create pre-built workflow `.gbai` packages
- [ ] Customer support escalation template
- [ ] E-commerce order processing template
- [ ] Content moderation template
- [ ] **Integration:** Auto-discovery via existing package system

### 2.3 Visual Designer Enhancement
- [ ] **File:** `src/designer/mod.rs`
- [ ] Add workflow mode to existing designer
- [ ] Real-time BASIC code preview
- [ ] Workflow validation and error checking
- [ ] **Test:** Visual designer workflow tests

---

## Phase 3: Intelligence & Learning (Months 5-6) 🧠

### 3.1 Smart LLM Routing
- [ ] **File:** `src/llm/smart_router.rs`
- [ ] Extend existing `llm/observability.rs`
- [ ] Add cost/latency tracking per model
- [ ] Automatic model selection based on task type
- [ ] **BASIC:** Enhance LLM keyword with OPTIMIZE FOR parameter
- [ ] **Database:** Add `model_performance` table
- [ ] **Test:** LLM routing optimization tests

### 3.2 Bot Learning System
- [ ] **File:** `src/bots/learning.rs`
- [ ] Pattern recognition from conversation history
- [ ] Cross-bot knowledge sharing mechanisms
- [ ] Confidence scoring for learned patterns
- [ ] **Write-back to .gbkb:** Store learned patterns as knowledge base files
- [ ] **Integration:** Use existing conversation storage + VectorDB
- [ ] **Test:** Bot learning behavior tests

### 3.3 Enhanced BASIC Keywords
```basic
' New keywords to implement
result = LLM "Analyze data" WITH OPTIMIZE FOR "speed"
BOT LEARN PATTERN "customer prefers email" WITH CONFIDENCE 0.8
preferences = BOT RECALL "customer communication patterns"
BOT SHARE KNOWLEDGE WITH "support-bot-2"
```

---

## Phase 4: Plugin Ecosystem (Months 7-8) 🔌

### 4.1 Plugin Registry
- [ ] **File:** `src/plugins/registry.rs`
- [ ] **Database:** Add `plugins` table with metadata
- [ ] Plugin security scanning system
- [ ] Version management and updates
- [ ] **Integration:** Extend existing MCP support

### 4.2 Plugin Discovery Keywords
- [ ] **File:** `src/basic/keywords/plugins.rs`
- [ ] Add `SEARCH PLUGINS` keyword
- [ ] Add `INSTALL PLUGIN` keyword
- [ ] Add `LIST PLUGINS` keyword
- [ ] **Integration:** Auto-update `mcp.csv` on install
- [ ] **Test:** Plugin installation and discovery tests

### 4.3 Plugin Marketplace
- [ ] **Database Schema:**
```sql
CREATE TABLE plugins (
  id UUID PRIMARY KEY,
  name TEXT UNIQUE,
  description TEXT,
  mcp_server_url TEXT,
  permissions TEXT[],
  security_scan_result JSONB,
  downloads INTEGER DEFAULT 0,
  rating FLOAT,
  created_at TIMESTAMPTZ DEFAULT NOW()
);
```

---

## Implementation Guidelines

### Code Standards
- [ ] **No breaking changes** - all existing `.gbai` packages must work
- [ ] **Extend existing systems** - don't rebuild what works
- [ ] **BASIC-first design** - everything accessible via BASIC keywords
- [ ] **Use existing infrastructure** - PostgreSQL, Redis, Qdrant, LXC
- [ ] **Proper error handling** - no unwrap(), use SafeCommand wrapper

### Testing Requirements
- [ ] **Unit tests** for all new BASIC keywords
- [ ] **Integration tests** for workflow execution
- [ ] **Performance tests** for multi-agent coordination
- [ ] **Backward compatibility tests** for existing `.gbai` packages

### Documentation Updates
- [ ] **File:** `docs/reference/basic-language.md` - Add new keywords
- [ ] **File:** `docs/guides/workflows.md` - Workflow creation guide
- [ ] **File:** `docs/guides/multi-agent.md` - Multi-agent patterns
- [ ] **File:** `docs/api/workflow-api.md` - Workflow REST endpoints

---

## File Structure Changes

```
src/
├── basic/keywords/
│   ├── orchestration.rs          # NEW: ORCHESTRATE WORKFLOW
│   ├── events.rs                 # NEW: ON EVENT, PUBLISH EVENT
│   ├── agent_learning.rs         # NEW: AGENT LEARN/RECALL
│   └── plugins.rs                # NEW: SEARCH/INSTALL PLUGINS
├── designer/
│   ├── workflow_canvas.rs        # NEW: Visual workflow editor
│   └── mod.rs                    # EXTEND: Add workflow mode
├── llm/
│   └── smart_router.rs           # NEW: Intelligent model routing
├── agents/
│   └── learning.rs               # NEW: Agent learning system
└── plugins/
    └── registry.rs               # NEW: Plugin management

templates/
└── workflow-templates/           # NEW: Pre-built workflows
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
- [ ] **Backward compatibility:** 100% existing `.gbai` packages work
- [ ] **Performance:** Workflow execution <2s overhead
- [ ] **Reliability:** 99.9% workflow completion rate
- [ ] **Memory usage:** <10% increase from current baseline

### Business Metrics
- [ ] **Workflow creation time:** 50% reduction vs manual coordination
- [ ] **Training time:** 80% reduction for non-programmers
- [ ] **Enterprise adoption:** 10x faster implementation
- [ ] **Community plugins:** 100+ plugins in first 6 months

---

## Risk Mitigation

### Technical Risks
- [ ] **Context overflow:** Implement workflow state persistence
- [ ] **Bot coordination failures:** Add timeout and retry mechanisms
- [ ] **Performance degradation:** Implement workflow step caching
- [ ] **Memory leaks:** Proper cleanup of workflow sessions

### Business Risks
- [ ] **Breaking changes:** Comprehensive backward compatibility testing
- [ ] **Complexity creep:** Keep BASIC-first design principle
- [ ] **Performance impact:** Benchmark all new features
- [ ] **Security vulnerabilities:** Security review for all plugin systems

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
| **Phase 4** | 2 months | Plugin ecosystem | Phase 1 |

**Total Duration:** 8 months  
**Target Release:** General Bots 7.0 - Q3 2026

---

## Getting Started

### Immediate Next Steps
1. [ ] **Create feature branch:** `git checkout -b feature/orchestration-7.0`
2. [ ] **Set up development environment:** Ensure Rust 1.75+, PostgreSQL, Redis
3. [ ] **Start with Phase 1.1:** Implement `ORCHESTRATE WORKFLOW` keyword
4. [ ] **Create basic test:** Simple 2-step workflow execution
5. [ ] **Document progress:** Update this TODO.md as tasks complete

### Development Order
1. **Start with BASIC keywords** - Core functionality first
2. **Add database schema** - Persistence layer
3. **Implement workflow engine** - Execution logic
4. **Add visual designer** - User interface
5. **Enhance with intelligence** - AI improvements
6. **Build plugin system** - Extensibility

**Remember:** Build on existing systems, don't rebuild. Every new feature should extend what already works in General Bots.
