# Multi-Agent Orchestration

General Bots supports sophisticated multi-agent systems where multiple specialized bots collaborate to handle complex tasks. This chapter covers the architecture, keywords, and best practices for building multi-agent solutions.

## Overview

Multi-agent orchestration enables:

- **Task specialization** - Each bot focuses on what it does best
- **Collaborative problem-solving** - Bots work together on complex tasks
- **Scalable architectures** - Add new specialists without modifying existing bots
- **Resilient systems** - Failures are isolated and handled gracefully

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Multi-Agent System                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐    A2A Protocol    ┌──────────┐               │
│  │          │◄──────────────────►│          │               │
│  │ Sales    │                    │ Support  │               │
│  │ Bot      │    ┌──────────┐    │ Bot      │               │
│  │          │◄──►│          │◄──►│          │               │
│  └──────────┘    │ Billing  │    └──────────┘               │
│                  │ Bot      │                                │
│  ┌──────────┐    │          │    ┌──────────┐               │
│  │          │◄──►└──────────┘◄──►│          │               │
│  │ Research │                    │ Analytics│               │
│  │ Bot      │                    │ Bot      │               │
│  │          │                    │          │               │
│  └──────────┘                    └──────────┘               │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Core Keywords

### ADD BOT

Adds a bot to the current session with optional triggers and tools.

```basic
' Add a bot with keyword triggers
ADD BOT "billing-bot" TRIGGER ON "billing,invoice,payment"

' Add a bot with tool access
ADD BOT "analyst-bot" TOOLS "calculate,forecast,report"

' Add a bot with scheduled tasks
ADD BOT "monitor-bot" SCHEDULE "0 */1 * * *"
```

### DELEGATE TO BOT

Sends a task to another bot and optionally waits for response.

```basic
' Fire-and-forget delegation
DELEGATE "Process this order" TO BOT "order-processor"

' Get response from delegation
result = DELEGATE "Calculate total for items" TO BOT "calculator-bot"
TALK "Total: " + result

' Delegation with timeout
result = DELEGATE "Analyze report" TO BOT "analyst-bot" TIMEOUT 60
```

### BROADCAST TO BOTS

Sends a message to all bots in the session.

```basic
' Notify all bots of an event
BROADCAST "New customer signup: " + customerid TO BOTS

' Emergency shutdown signal
BROADCAST "SHUTDOWN" TO BOTS
```

### TRANSFER CONVERSATION

Hands off the entire conversation to another bot.

```basic
' Transfer to specialist
TALK "Let me connect you with our billing specialist."
TRANSFER CONVERSATION TO "billing-bot"

' Transfer with context
SET CONTEXT "issue" AS "refund request"
SET CONTEXT "amount" AS "$150"
TRANSFER CONVERSATION TO "refunds-bot"
```

## A2A Protocol

The Agent-to-Agent (A2A) protocol handles all inter-bot communication.

### Message Types

| Type | Description | Use Case |
|------|-------------|----------|
| `Request` | Ask bot to perform task | Task delegation |
| `Response` | Reply to a request | Return results |
| `Broadcast` | Message to all bots | Notifications |
| `Delegate` | Hand off task | Specialization |
| `Collaborate` | Joint task | Team work |

### Message Structure

```basic
' A2A messages contain:
' - from_agent: Source bot ID
' - to_agent: Target bot ID  
' - message_type: Request, Response, etc.
' - payload: The actual content
' - correlation_id: Links request/response
' - timestamp: When sent
```

### Configuration

```csv
name,value
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-retry-count,3
a2a-queue-size,100
```

| Option | Default | Description |
|--------|---------|-------------|
| `a2a-enabled` | `true` | Enable A2A communication |
| `a2a-timeout` | `30` | Default timeout (seconds) |
| `a2a-max-hops` | `5` | Maximum delegation chain depth |
| `a2a-retry-count` | `3` | Retries on failure |
| `a2a-queue-size` | `100` | Max pending messages |

## Memory Management

### User Memory (Cross-Bot)

User memory is accessible across all bots, enabling seamless personalization.

```basic
' In any bot - store user preference
SET USER MEMORY "language", "pt-BR"
SET USER MEMORY "timezone", "America/Sao_Paulo"

' In any other bot - retrieve preference
language = GET USER MEMORY("language")
TALK "Olá!" IF language = "pt-BR"
```

### Bot Memory (Per-Bot)

Bot memory is isolated to each bot for bot-specific state.

```basic
' In sales-bot
SET BOT MEMORY "deals_closed", dealscount

' In support-bot (different memory space)
SET BOT MEMORY "tickets_resolved", ticketcount
```

### Session Memory (Temporary)

Session memory is shared within a conversation session.

```basic
' Store in session
SET "current_topic", "billing"

' Available to all bots in session
topic = GET "current_topic"
```

### Memory Scope Comparison

| Memory Type | Scope | Persistence | Use Case |
|-------------|-------|-------------|----------|
| User Memory | Per user, all bots | Permanent | Preferences, profile |
| Bot Memory | Per bot, all users | Permanent | Bot state, counters |
| Session Memory | Per session | Session lifetime | Current context |

## Model Routing

Different bots can use different models optimized for their tasks.

### USE MODEL Keyword

```basic
' In customer service bot - use quality model
USE MODEL "quality"

' In quick-answer bot - use fast model
USE MODEL "fast"

' In code helper bot - use code model
USE MODEL "code"

' Let system decide
USE MODEL "auto"
```

### Model Routing Strategies

| Strategy | Description |
|----------|-------------|
| `manual` | Explicit model selection only |
| `auto` | System chooses based on query |
| `load-balanced` | Distribute for throughput |
| `fallback` | Try models in order |

### Configuration

```csv
name,value
model-routing-strategy,auto
model-default,fast
model-fast,DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
model-quality,gpt-4
model-code,codellama-7b.gguf
```

## Hybrid RAG Search

Multi-agent systems benefit from shared knowledge bases with advanced search.

### Configuration

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
```

### How It Works

1. **Dense Search** - Semantic/vector similarity (0.7 weight)
2. **Sparse Search** - BM25 keyword matching (0.3 weight)
3. **Fusion** - Reciprocal Rank Fusion combines results
4. **Reranking** - Optional LLM reranking for quality

```basic
' Hybrid search is automatic when enabled
USE KB "company-knowledge"
result = FIND "customer refund policy"
' Returns results using both semantic and keyword matching
```

## Code Sandbox

Bots can execute code in isolated sandboxes for data processing.

### Supported Languages

```basic
' Python for data science
result = RUN PYTHON "
import pandas as pd
df = pd.DataFrame({'a': [1,2,3]})
print(df.sum().to_json())
"

' JavaScript for JSON processing
result = RUN JAVASCRIPT "
const data = {items: [1,2,3]};
console.log(JSON.stringify(data.items.map(x => x * 2)));
"

' Bash for system tasks
result = RUN BASH "ls -la /data"
```

### Sandbox Configuration

```csv
name,value
sandbox-runtime,lxc
sandbox-timeout,30
sandbox-memory-mb,512
sandbox-cpu-percent,50
sandbox-network,false
```

### Runtimes

| Runtime | Security | Performance | Requirements |
|---------|----------|-------------|--------------|
| LXC | High | Excellent | LXC installed |
| Docker | High | Good | Docker daemon |
| Firecracker | Highest | Good | Firecracker |
| Process | Low | Best | None (fallback) |

## Agent Reflection

Bots can self-analyze and improve through reflection.

### Enable Reflection

```basic
' Enable self-reflection
BOT REFLECTION true

' Monitor specific metrics
BOT REFLECTION ON "conversation_quality"
BOT REFLECTION ON "response_accuracy"
```

### Get Insights

```basic
' Retrieve reflection analysis
insights = BOT REFLECTION INSIGHTS()

PRINT "Quality Score: " + insights.qualityScore
PRINT "Issues: " + insights.issuesCount

FOR EACH suggestion IN insights.suggestions
    PRINT "Suggestion: " + suggestion
NEXT suggestion
```

### Reflection Metrics

| Metric | Description |
|--------|-------------|
| `conversation_quality` | Overall conversation effectiveness |
| `response_accuracy` | Correctness of responses |
| `user_satisfaction` | Estimated user satisfaction |
| `tone_appropriateness` | Tone matches context |
| `resolution_rate` | Issues successfully resolved |

## SSE Streaming

Real-time streaming for responsive multi-agent UIs.

### Enable Streaming

```csv
name,value
sse-enabled,true
sse-heartbeat,30
sse-max-connections,1000
```

### Client Integration

```javascript
// Connect to SSE endpoint
const eventSource = new EventSource('/api/chat/stream?session=' + sessionId);

eventSource.onmessage = (event) => {
    const data = JSON.parse(event.data);
    
    if (data.type === 'token') {
        // Streaming token
        appendToMessage(data.content);
    } else if (data.type === 'bot_switch') {
        // Different bot responding
        showBotIndicator(data.botName);
    } else if (data.type === 'complete') {
        // Response complete
        finalizeMessage();
    }
};
```

## Patterns and Best Practices

### Router Pattern

A central router bot directs queries to specialists.

```basic
' router-bot/start.bas
HEAR userquery

' Classify the query
category = LLM "Classify into: billing, technical, sales, general. Query: " + userquery

SWITCH category
    CASE "billing"
        result = DELEGATE userquery TO BOT "billing-bot"
    CASE "technical"
        result = DELEGATE userquery TO BOT "tech-bot"
    CASE "sales"
        result = DELEGATE userquery TO BOT "sales-bot"
    CASE ELSE
        result = LLM userquery
END SWITCH

TALK result
```

### Expert Panel Pattern

Multiple bots provide perspectives on complex questions.

```basic
' Get input from multiple experts
question = "Should we expand into the European market?"

marketAnalysis = DELEGATE question TO BOT "market-analyst"
financialView = DELEGATE question TO BOT "finance-expert"
riskAssessment = DELEGATE question TO BOT "risk-assessor"

' Synthesize responses
synthesis = LLM "Synthesize these expert opinions into a recommendation:
Market: " + marketAnalysis + "
Finance: " + financialView + "
Risk: " + riskAssessment

BEGIN TALK
**Expert Panel Summary**

📊 **Market Analysis:** {marketAnalysis}

💰 **Financial View:** {financialView}

⚠️ **Risk Assessment:** {riskAssessment}

📋 **Recommendation:** {synthesis}
END TALK
```

### Escalation Pattern

Automatic escalation when bot can't handle query.

```basic
' First-line support bot
confidence = LLM "Rate your confidence (0-100) in answering: " + userquery

IF confidence < 50 THEN
    ' Escalate to specialist
    TALK "Let me connect you with a specialist who can better help."
    SET CONTEXT "escalation_reason" AS "low_confidence"
    SET CONTEXT "original_query" AS userquery
    TRANSFER CONVERSATION TO "senior-support-bot"
ELSE
    ' Handle normally
    response = LLM userquery
    TALK response
END IF
```

### Supervisor Pattern

A supervisor bot monitors and coordinates workers.

```basic
' supervisor-bot/monitor.bas
SET SCHEDULE "*/5 * * * *"  ' Run every 5 minutes

' Check all worker bots
workers = ["processor-1", "processor-2", "processor-3"]

FOR EACH worker IN workers
    status = DELEGATE "HEALTH_CHECK" TO BOT worker TIMEOUT 10
    
    IF status = "" OR status = "ERROR" THEN
        ' Worker unresponsive
        SEND MAIL admin, "Bot Alert", worker + " is unresponsive"
        DELEGATE "RESTART" TO BOT "bot-manager"
    END IF
NEXT worker
```

## Database Schema

Multi-agent systems use several database tables:

### a2a_messages

Stores inter-agent communication.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Message ID |
| `from_agent` | VARCHAR | Sender bot ID |
| `to_agent` | VARCHAR | Recipient bot ID |
| `message_type` | VARCHAR | Request, Response, etc. |
| `payload` | JSONB | Message content |
| `correlation_id` | UUID | Links request/response |
| `status` | VARCHAR | pending, delivered, failed |
| `created_at` | TIMESTAMP | When created |

### user_memory

Stores cross-bot user data.

| Column | Type | Description |
|--------|------|-------------|
| `user_id` | UUID | User identifier |
| `key` | VARCHAR | Memory key |
| `value` | JSONB | Stored value |
| `memory_type` | VARCHAR | preference, fact, context |
| `ttl` | TIMESTAMP | Optional expiration |

### agent_reflections

Stores reflection analysis results.

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Reflection ID |
| `bot_id` | UUID | Bot that was analyzed |
| `conversation_id` | UUID | Analyzed conversation |
| `quality_score` | FLOAT | Overall quality (0-1) |
| `insights` | JSONB | Analysis details |
| `created_at` | TIMESTAMP | When analyzed |

## Troubleshooting

### Bot Not Responding to Delegation

1. Check bot is registered: `LIST BOTS`
2. Verify A2A is enabled: `a2a-enabled,true`
3. Check timeout is sufficient
4. Review bot logs for errors

### Memory Not Sharing Between Bots

1. Ensure using `SET USER MEMORY` not `SET BOT MEMORY`
2. Check `user-memory-enabled,true`
3. Verify same user identity across bots

### Circular Delegation Detected

1. Review delegation chains
2. Increase `a2a-max-hops` if legitimately deep
3. Add guards to prevent loops:

```basic
hops = GET "delegation_hops"
IF hops > 3 THEN
    TALK "I'll handle this directly."
    ' Don't delegate further
ELSE
    SET "delegation_hops", hops + 1
    DELEGATE task TO BOT "specialist"
END IF
```

## See Also

- [ADD BOT Keyword](../chapter-06-gbdialog/keyword-add-bot.md)
- [DELEGATE TO BOT Keyword](../chapter-06-gbdialog/keyword-delegate-to-bot.md)
- [Memory Management](./memory-management.md)
- [Model Routing](../chapter-08-config/llm-config.md)
- [Code Sandbox](../chapter-07-gbapp/containers.md)
- [SSE Streaming](./streaming.md)