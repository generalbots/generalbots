# BOT REFLECTION

Enables agent self-analysis and improvement by using LLM to evaluate conversation quality, identify issues, and suggest improvements. This is a key feature for continuous agent optimization.

## Syntax

```basic
BOT REFLECTION enabled
BOT REFLECTION ON "metric"
insights = BOT REFLECTION INSIGHTS()
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `enabled` | Boolean | `true` to enable, `false` to disable reflection |
| `metric` | String | Specific metric to analyze (e.g., "conversation_quality", "response_accuracy") |

## Description

`BOT REFLECTION` activates the agent self-improvement system, which periodically analyzes conversations and provides actionable insights. When enabled, the system:

- **Analyzes conversation quality** - Tone, clarity, helpfulness
- **Identifies issues** - Misunderstandings, incomplete answers, user frustration
- **Suggests improvements** - Better responses, missing information, tone adjustments
- **Tracks metrics over time** - Quality scores, resolution rates

This creates a continuous improvement loop where agents learn from their interactions.

## Examples

### Enable Basic Reflection

```basic
' Enable reflection for this bot session
BOT REFLECTION true

' Normal conversation proceeds
TALK "Hello! How can I help you today?"
HEAR userquery
response = LLM userquery
TALK response

' Reflection runs automatically in background
```

### Monitor Specific Metrics

```basic
' Enable reflection on conversation quality
BOT REFLECTION ON "conversation_quality"

' Enable reflection on response accuracy
BOT REFLECTION ON "response_accuracy"

' Enable reflection on user satisfaction
BOT REFLECTION ON "user_satisfaction"
```

### Retrieve Reflection Insights

```basic
' Get insights from reflection analysis
insights = BOT REFLECTION INSIGHTS()

IF insights <> "" THEN
    PRINT "Reflection Insights:"
    PRINT insights.summary
    PRINT "Quality Score: " + insights.qualityScore
    PRINT "Issues Found: " + insights.issuesCount
    
    FOR EACH suggestion IN insights.suggestions
        PRINT "Suggestion: " + suggestion
    NEXT suggestion
END IF
```

### Use Insights for Self-Improvement

```basic
' Periodic reflection check
BOT REFLECTION true

' After conversation ends, check insights
insights = BOT REFLECTION INSIGHTS()

IF insights.qualityScore < 0.7 THEN
    ' Log for review
    PRINT "Low quality conversation detected"
    PRINT "Issues: " + insights.issues
    
    ' Store for analysis
    SET BOT MEMORY "reflection_" + conversationid, insights
END IF
```

### Admin Dashboard Integration

```basic
' Script for admin to review bot performance
insights = BOT REFLECTION INSIGHTS()

BEGIN TALK
**Bot Performance Report**

📊 **Quality Score:** {insights.qualityScore}/1.0

📈 **Metrics:**
- Response Accuracy: {insights.responseAccuracy}%
- User Satisfaction: {insights.userSatisfaction}%
- Resolution Rate: {insights.resolutionRate}%

⚠️ **Issues Identified:**
{insights.issues}

💡 **Improvement Suggestions:**
{insights.suggestions}
END TALK
```

### Conditional Reflection

```basic
' Only reflect on complex conversations
messageCount = GET BOT MEMORY("messageCount")

IF messageCount > 5 THEN
    ' Enable reflection for longer conversations
    BOT REFLECTION true
    BOT REFLECTION ON "conversation_quality"
END IF
```

### Reflection with Alerts

```basic
' Enable reflection with alerting
BOT REFLECTION true

' Check for critical issues periodically
insights = BOT REFLECTION INSIGHTS()

IF insights.criticalIssues > 0 THEN
    ' Alert admin
    SEND MAIL admin, "Bot Alert: Critical Issues Detected", insights.summary
END IF
```

## Reflection Metrics

| Metric | Description | Score Range |
|--------|-------------|-------------|
| `conversation_quality` | Overall conversation effectiveness | 0.0 - 1.0 |
| `response_accuracy` | How accurate/correct responses are | 0.0 - 1.0 |
| `user_satisfaction` | Estimated user satisfaction | 0.0 - 1.0 |
| `tone_appropriateness` | Whether tone matches context | 0.0 - 1.0 |
| `resolution_rate` | Whether user issues were resolved | 0.0 - 1.0 |
| `response_time` | Average response latency | milliseconds |

## Insights Object Structure

```basic
insights = BOT REFLECTION INSIGHTS()

' Available properties:
insights.qualityScore       ' Overall quality (0-1)
insights.summary           ' Text summary of analysis
insights.issues            ' Array of identified issues
insights.issuesCount       ' Number of issues found
insights.suggestions       ' Array of improvement suggestions
insights.metrics           ' Object with detailed metrics
insights.criticalIssues    ' Count of critical problems
insights.conversationId    ' ID of analyzed conversation
insights.timestamp         ' When analysis was performed
```

## Config.csv Options

```csv
name,value
reflection-enabled,true
reflection-interval,10
reflection-min-messages,3
reflection-model,quality
reflection-store-insights,true
```

| Option | Default | Description |
|--------|---------|-------------|
| `reflection-enabled` | `true` | Enable/disable reflection globally |
| `reflection-interval` | `10` | Messages between reflection runs |
| `reflection-min-messages` | `3` | Minimum messages before reflecting |
| `reflection-model` | `quality` | LLM model for reflection analysis |
| `reflection-store-insights` | `true` | Store insights in database |

## How Reflection Works

1. **Collection** - Conversation history is collected
2. **Analysis** - LLM analyzes the conversation against metrics
3. **Scoring** - Quality scores are calculated
4. **Identification** - Issues and patterns are identified
5. **Suggestion** - Improvement suggestions are generated
6. **Storage** - Results stored for dashboards and trends

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`LLM`](./keyword-llm.md) | Query the language model |
| [`SET BOT MEMORY`](./keyword-set-bot-memory.md) | Store bot-level data |
| [`PRINT`](./keyword-print.md) | Debug output |

## Performance Considerations

- Reflection uses LLM calls (affects cost/latency)
- Run reflection periodically, not on every message
- Use smaller models for reflection when possible
- Consider async reflection for production

## Best Practices

1. **Enable for complex bots** - Most valuable for customer-facing agents
2. **Review insights regularly** - Use dashboards to spot trends
3. **Act on suggestions** - Update prompts and tools based on insights
4. **Set appropriate intervals** - Balance insight freshness vs cost
5. **Store for analysis** - Track improvements over time

## Limitations

- Reflection adds LLM cost per analysis
- Analysis quality depends on model capability
- Cannot analyze real-time user emotions
- Historical only (not predictive)

## See Also

- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Multi-agent systems
- [Observability](../chapter-11-features/observability.md) - Monitoring and metrics
- [LLM Configuration](../chapter-08-config/llm-config.md) - Model setup