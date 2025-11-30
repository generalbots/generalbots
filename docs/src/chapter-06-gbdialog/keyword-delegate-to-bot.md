# DELEGATE TO BOT

Delegates a task or message to another bot in a multi-agent system. This enables agent-to-agent communication using the A2A (Agent-to-Agent) protocol.

## Syntax

```basic
DELEGATE "message" TO BOT "botname"
DELEGATE "message" TO BOT "botname" TIMEOUT seconds
result = DELEGATE "message" TO BOT "botname"
```

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `message` | String | The task or message to send to the target bot |
| `botname` | String | Name of the target bot to delegate to |
| `seconds` | Number | Optional timeout in seconds (default: 30) |

## Description

`DELEGATE TO BOT` sends a message or task to another bot and optionally waits for a response. This is the core keyword for multi-agent orchestration, enabling:

- **Task specialization** - Route tasks to specialized bots
- **Agent collaboration** - Multiple bots working together
- **Workload distribution** - Spread tasks across agents
- **Expert consultation** - Query domain-specific bots

The delegation uses the A2A (Agent-to-Agent) protocol which handles:
- Message routing between agents
- Correlation IDs for request/response matching
- Timeout handling
- Error propagation

## Examples

### Basic Delegation

```basic
' Delegate a translation task to a specialized bot
DELEGATE "Translate 'Hello World' to Portuguese" TO BOT "translator-bot"
TALK "Translation request sent!"
```

### Get Response from Delegated Bot

```basic
' Ask the finance bot for a calculation
result = DELEGATE "Calculate ROI for investment of $10000 with 12% annual return over 5 years" TO BOT "finance-bot"
TALK "The finance expert says: " + result
```

### Delegation with Timeout

```basic
' Long-running task with extended timeout
result = DELEGATE "Analyze this quarterly report and provide insights" TO BOT "analyst-bot" TIMEOUT 120
TALK result
```

### Multi-Bot Workflow

```basic
' Customer support escalation workflow
issue = "Customer reports billing discrepancy"

' First, check with billing bot
billingInfo = DELEGATE "Check account status for customer " + customerid TO BOT "billing-bot" TIMEOUT 30

IF INSTR(billingInfo, "discrepancy") > 0 THEN
    ' Escalate to senior support
    resolution = DELEGATE "Priority: " + issue + " Details: " + billingInfo TO BOT "senior-support-bot" TIMEOUT 60
    TALK "A senior agent is handling your case: " + resolution
ELSE
    TALK "Your account looks fine: " + billingInfo
END IF
```

### Parallel Expert Consultation

```basic
' Get opinions from multiple specialized bots
question = "What's the best approach for this investment portfolio?"

' Delegate to multiple experts
stockAnalysis = DELEGATE question TO BOT "stock-analyst"
bondAnalysis = DELEGATE question TO BOT "bond-analyst"
riskAssessment = DELEGATE question TO BOT "risk-assessor"

' Combine insights
BEGIN TALK
**Investment Analysis Summary**

📈 **Stock Analysis:** {stockAnalysis}

📊 **Bond Analysis:** {bondAnalysis}

⚠️ **Risk Assessment:** {riskAssessment}
END TALK
```

### Conditional Routing

```basic
' Route to appropriate specialist based on query type
HEAR userquery

' Use LLM to classify the query
category = LLM "Classify this query into one of: billing, technical, sales, general. Query: " + userquery

SWITCH category
    CASE "billing"
        response = DELEGATE userquery TO BOT "billing-bot"
    CASE "technical"
        response = DELEGATE userquery TO BOT "tech-support-bot"
    CASE "sales"
        response = DELEGATE userquery TO BOT "sales-bot"
    CASE ELSE
        response = DELEGATE userquery TO BOT "general-assistant"
END SWITCH

TALK response
```

### Chain of Delegation

```basic
' Research assistant that coordinates multiple bots
topic = "renewable energy trends 2025"

' Step 1: Gather data
rawData = DELEGATE "Search for recent data on " + topic TO BOT "research-bot" TIMEOUT 60

' Step 2: Analyze data
analysis = DELEGATE "Analyze this data and identify key trends: " + rawData TO BOT "analyst-bot" TIMEOUT 45

' Step 3: Generate report
report = DELEGATE "Create an executive summary from this analysis: " + analysis TO BOT "writer-bot" TIMEOUT 30

TALK report
```

## A2A Protocol Details

When you use `DELEGATE TO BOT`, the system creates an A2A message with:

| Field | Description |
|-------|-------------|
| `from_agent` | The current bot's identifier |
| `to_agent` | The target bot name |
| `message_type` | `Delegate` for task delegation |
| `payload` | The message content |
| `correlation_id` | Unique ID to match response |
| `timestamp` | When the message was sent |

## Error Handling

```basic
' Handle delegation failures gracefully
ON ERROR RESUME NEXT

result = DELEGATE "Process payment" TO BOT "payment-bot" TIMEOUT 30

IF ERROR THEN
    TALK "I'm having trouble reaching our payment system. Please try again in a moment."
    ' Log the error
    PRINT "Delegation failed: " + ERROR_MESSAGE
ELSE
    TALK result
END IF
```

## Related Keywords

| Keyword | Description |
|---------|-------------|
| [`ADD BOT`](./keyword-add-bot.md) | Add a bot to the current session |
| [`BROADCAST TO BOTS`](./keyword-broadcast-to-bots.md) | Send message to all bots |
| [`TRANSFER CONVERSATION`](./keyword-transfer-conversation.md) | Hand off conversation to another bot |

## Config.csv Options

```csv
name,value
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-retry-count,3
```

| Option | Default | Description |
|--------|---------|-------------|
| `a2a-enabled` | `true` | Enable agent-to-agent communication |
| `a2a-timeout` | `30` | Default timeout in seconds |
| `a2a-max-hops` | `5` | Maximum delegation chain depth |
| `a2a-retry-count` | `3` | Number of retry attempts on failure |

## Best Practices

1. **Set appropriate timeouts** - Long tasks need longer timeouts
2. **Handle failures gracefully** - Always have a fallback
3. **Avoid circular delegation** - Bot A → Bot B → Bot A
4. **Keep delegation chains short** - Max 3-4 hops recommended
5. **Log delegations** - Helps with debugging multi-agent flows
6. **Use descriptive bot names** - `billing-bot` not `bot2`

## Limitations

- Maximum message size: 1MB
- Maximum timeout: 300 seconds (5 minutes)
- Maximum concurrent delegations: 10 per session
- Target bot must be registered and active

## See Also

- [Multi-Agent Orchestration](../chapter-11-features/multi-agent-orchestration.md) - Complete multi-agent guide
- [A2A Protocol](../chapter-11-features/a2a-protocol.md) - Technical protocol details
- [Bot Configuration](../chapter-08-config/parameters.md) - Bot setup