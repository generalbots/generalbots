# BASIC Language Reference - Version 6.2.0

## New Workflow Orchestration Keywords

### ORCHESTRATE WORKFLOW
Creates multi-step workflows with bot coordination.

**Syntax:**
```basic
ORCHESTRATE WORKFLOW "workflow-name"
  STEP 1: BOT "bot-name" "action"
  STEP 2: HUMAN APPROVAL FROM "email@domain.com" TIMEOUT 1800
  STEP 3: PARALLEL
    BRANCH A: BOT "bot-a" "process"
    BRANCH B: BOT "bot-b" "process"
  END PARALLEL
END WORKFLOW
```

**Features:**
- Workflow state persists through server restarts
- Variables automatically passed between steps
- Human approval integration with timeouts
- Parallel processing support

### Event System

**ON EVENT**
```basic
ON EVENT "event-name" DO
  TALK "Event received"
END ON
```

**PUBLISH EVENT**
```basic
PUBLISH EVENT "event-name"
```

**WAIT FOR EVENT**
```basic
WAIT FOR EVENT "approval-received" TIMEOUT 3600
```

### Enhanced Memory

**BOT SHARE MEMORY**
```basic
BOT SHARE MEMORY "key" WITH "target-bot"
```

**BOT SYNC MEMORY**
```basic
BOT SYNC MEMORY FROM "source-bot"
```

### Enhanced LLM (Feature-gated)

**Optimized LLM Calls**
```basic
result = LLM "Analyze data" WITH OPTIMIZE FOR "speed"
result = LLM "Complex task" WITH MAX_COST 0.05 MAX_LATENCY 2000
```

## File Type Detection

The designer automatically detects:
- **Tools**: Simple input/output functions
- **Workflows**: Multi-step orchestration
- **Regular Bots**: Conversational interfaces

## Backward Compatibility

All existing BASIC keywords continue to work unchanged. New keywords extend functionality without breaking existing `.gbai` packages.
