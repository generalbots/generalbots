# Multi-Agent Workflows Guide

## Creating Workflows

### Basic Workflow Structure
```basic
ORCHESTRATE WORKFLOW "workflow-name"
  STEP 1: BOT "analyzer" "process input"
  STEP 2: BOT "validator" "check results"
END WORKFLOW
```

### Human Approval Integration
```basic
STEP 3: HUMAN APPROVAL FROM "manager@company.com"
  TIMEOUT 1800  ' 30 minutes
  ON TIMEOUT: ESCALATE TO "director@company.com"
```

### Parallel Processing
```basic
STEP 4: PARALLEL
  BRANCH A: BOT "processor-1" "handle batch-a"
  BRANCH B: BOT "processor-2" "handle batch-b"
END PARALLEL
```

### Event-Driven Coordination
```basic
ON EVENT "data-ready" DO
  CONTINUE WORKFLOW AT STEP 5
END ON

PUBLISH EVENT "processing-complete"
```

### Cross-Bot Memory Sharing
```basic
BOT SHARE MEMORY "successful-patterns" WITH "learning-bot"
BOT SYNC MEMORY FROM "master-knowledge-bot"
```

## Best Practices

1. **Keep workflows focused** - Max 10 steps per workflow
2. **Use meaningful names** - Clear bot and step names
3. **Add timeouts** - Always set timeouts for human approvals
4. **Share knowledge** - Use memory sharing for bot learning
5. **Handle events** - Use event system for loose coupling

## Workflow Persistence

Workflows automatically survive server restarts. State is stored in PostgreSQL and recovered on startup.

## Visual Designer

Use the drag-and-drop designer at `/designer/workflow` to create workflows visually. The designer generates BASIC code automatically.
