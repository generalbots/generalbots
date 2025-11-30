# Multi-Agent Keywords

This section covers keywords for building multi-agent systems where multiple specialized bots collaborate to handle complex tasks.

## Overview

Multi-agent orchestration enables:

- **Task specialization** - Each bot focuses on what it does best
- **Collaborative problem-solving** - Bots work together on complex tasks
- **Scalable architectures** - Add new specialists without modifying existing bots
- **Resilient systems** - Failures are isolated and handled gracefully

## Keyword Summary

| Keyword | Syntax | Description |
|---------|--------|-------------|
| `ADD BOT` | `ADD BOT "name" TRIGGER ON "keywords"` | Add bot to session with triggers |
| `DELEGATE TO BOT` | `result = DELEGATE "msg" TO BOT "name"` | Send task to another bot |
| `BROADCAST TO BOTS` | `BROADCAST "message" TO BOTS` | Send message to all bots |
| `TRANSFER CONVERSATION` | `TRANSFER CONVERSATION TO "botname"` | Hand off conversation |
| `BOT REFLECTION` | `BOT REFLECTION true` | Enable agent self-analysis |
| `BOT REFLECTION INSIGHTS` | `insights = BOT REFLECTION INSIGHTS()` | Get reflection results |

## ADD BOT

Adds a bot to the current session with optional triggers, tools, and schedules.

```basic
' Add bot with keyword triggers
ADD BOT "billing-bot" TRIGGER ON "billing,invoice,payment"

' Add bot with tool access
ADD BOT "analyst-bot" TOOLS "calculate,forecast,report"

' Add bot with scheduled execution
ADD BOT "monitor-bot" SCHEDULE "0 */1 * * *"

' Add bot with multiple configurations
ADD BOT "support-bot" TRIGGER ON "help,support" TOOLS "ticket,escalate"
```

### Trigger Types

| Type | Description | Example |
|------|-------------|---------|
| `TRIGGER ON` | Keyword-based activation | `TRIGGER ON "billing,payment"` |
| `TOOLS` | Tool-based activation | `TOOLS "calculate,search"` |
| `SCHEDULE` | Cron-based activation | `SCHEDULE "0 9 * * *"` |

## DELEGATE TO BOT

Sends a task to another bot and optionally waits for a response.

```basic
' Fire-and-forget delegation
DELEGATE "Process this order" TO BOT "order-processor"

' Get response from delegation
result = DELEGATE "Calculate ROI" TO BOT "finance-bot"
TALK "Result: " + result

' Delegation with timeout
result = DELEGATE "Analyze report" TO BOT "analyst-bot" TIMEOUT 60
```

### Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | String | Task or message to send |
| `botname` | String | Target bot name |
| `TIMEOUT` | Number | Optional timeout in seconds (default: 30) |

## BROADCAST TO BOTS

Sends a message to all bots in the current session.

```basic
' Notify all bots of an event
BROADCAST "New customer signup: " + customerid TO BOTS

' Emergency signal
BROADCAST "MAINTENANCE_MODE" TO BOTS

' Data update notification
BROADCAST "PRICE_UPDATE:" + JSON(prices) TO BOTS
```

## TRANSFER CONVERSATION

Hands off the entire conversation to another bot. The current bot exits.

```basic
' Simple transfer
TALK "Let me connect you with our billing specialist."
TRANSFER CONVERSATION TO "billing-bot"

' Transfer with context
SET CONTEXT "issue" AS "refund request"
SET CONTEXT "amount" AS "$150"
TRANSFER CONVERSATION TO "refunds-bot"

' Conditional transfer
IF issueType = "technical" THEN
    TRANSFER CONVERSATION TO "tech-support-bot"
ELSE
    TRANSFER CONVERSATION TO "general-support-bot"
END IF
```

## BOT REFLECTION

Enables agent self-analysis for continuous improvement.

```basic
' Enable reflection
BOT REFLECTION true

' Disable reflection
BOT REFLECTION false

' Monitor specific metric
BOT REFLECTION ON "conversation_quality"
BOT REFLECTION ON "response_accuracy"
BOT REFLECTION ON "user_satisfaction"
```

### Reflection Metrics

| Metric | Description |
|--------|-------------|
| `conversation_quality` | Overall conversation effectiveness |
| `response_accuracy` | Correctness of responses |
| `user_satisfaction` | Estimated user satisfaction |
| `tone_appropriateness` | Whether tone matches context |
| `resolution_rate` | Whether issues were resolved |

## BOT REFLECTION INSIGHTS

Retrieves the results of reflection analysis.

```basic
' Get insights
insights = BOT REFLECTION INSIGHTS()

' Access properties
PRINT "Quality Score: " + insights.qualityScore
PRINT "Issues: " + insights.issuesCount

' Iterate suggestions
FOR EACH suggestion IN insights.suggestions
    PRINT "Suggestion: " + suggestion
NEXT suggestion

' Use for alerting
IF insights.qualityScore < 0.5 THEN
    SEND MAIL admin, "Low Quality Alert", insights.summary
END IF
```

### Insights Object

| Property | Type | Description |
|----------|------|-------------|
| `qualityScore` | Number | Overall quality (0-1) |
| `summary` | String | Text summary |
| `issues` | Array | Identified issues |
| `issuesCount` | Number | Count of issues |
| `suggestions` | Array | Improvement suggestions |
| `criticalIssues` | Number | Critical problem count |
| `timestamp` | DateTime | When analyzed |

## Common Patterns

### Router Pattern

A central bot routes queries to specialists.

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

Multiple bots provide perspectives.

```basic
question = "Should we expand into Europe?"

' Get multiple expert opinions
marketView = DELEGATE question TO BOT "market-analyst"
financeView = DELEGATE question TO BOT "finance-expert"
riskView = DELEGATE question TO BOT "risk-assessor"

' Synthesize
synthesis = LLM "Combine these expert views: " + marketView + financeView + riskView
TALK synthesis
```

### Escalation Pattern

Automatic escalation when confidence is low.

```basic
' First-line bot
confidence = LLM "Rate confidence (0-100) for: " + userquery

IF confidence < 50 THEN
    TALK "Let me connect you with a specialist."
    SET CONTEXT "escalation_reason" AS "low_confidence"
    TRANSFER CONVERSATION TO "senior-support-bot"
ELSE
    response = LLM userquery
    TALK response
END IF
```

## Configuration

### config.csv Options

```csv
name,value
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-retry-count,3
reflection-enabled,true
reflection-interval,10
reflection-min-messages,3
```

| Option | Default | Description |
|--------|---------|-------------|
| `a2a-enabled` | `true` | Enable agent-to-agent communication |
| `a2a-timeout` | `30` | Default delegation timeout (seconds) |
| `a2a-max-hops` | `5` | Maximum delegation chain depth |
| `a2a-retry-count` | `3` | Retry attempts on failure |
| `reflection-enabled` | `true` | Enable bot reflection |
| `reflection-interval` | `10` | Messages between reflections |

## Best Practices

1. **Use descriptive bot names** - `billing-bot` not `bot2`
2. **Set appropriate timeouts** - Long tasks need longer timeouts
3. **Handle failures gracefully** - Always have fallback paths
4. **Avoid circular delegation** - Bot A → Bot B → Bot A
5. **Keep chains short** - Max 3-4 delegation hops
6. **Log delegations** - Helps debug multi-agent flows
7. **Review reflection insights** - Act on improvement suggestions

## See Also

- [ADD BOT](./keyword-add-bot.md) - Detailed ADD BOT reference
- [DELEGATE TO BOT](./keyword-delegate-to-bot.md) - Delegation details
- [BOT REFLECTION](./keyword-bot-reflection.md) - Reflection details
- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Complete guide
- [A2A Protocol](../chapter-11-features/a2a-protocol.md) - Protocol details