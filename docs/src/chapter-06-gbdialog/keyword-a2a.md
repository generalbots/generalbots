# A2A Protocol Keywords

Agent-to-Agent (A2A) protocol enables communication between multiple bots in a session.

## Keywords

| Keyword | Purpose |
|---------|---------|
| `SEND TO BOT` | Send message to specific bot |
| `BROADCAST` | Send message to all bots |
| `COLLABORATE WITH` | Request collaboration on a task |
| `WAIT FOR BOT` | Wait for response from another bot |
| `DELEGATE CONVERSATION` | Hand off conversation to another bot |
| `GET A2A MESSAGES` | Retrieve pending messages |

## SEND TO BOT

```basic
result = SEND TO BOT "assistant-bot", "Please help with this query"
```

## BROADCAST

```basic
BROADCAST "New customer request received"
```

## COLLABORATE WITH

```basic
bots = ["research-bot", "writing-bot"]
result = COLLABORATE WITH bots, "Write a market analysis report"
```

## WAIT FOR BOT

```basic
SEND TO BOT "analysis-bot", "Analyze this data"
response = WAIT FOR BOT "analysis-bot", 30  ' 30 second timeout
```

## DELEGATE CONVERSATION

```basic
DELEGATE CONVERSATION TO "support-bot"
```

## Message Types

| Type | Description |
|------|-------------|
| `Request` | Request action from another agent |
| `Response` | Response to a request |
| `Broadcast` | Message to all agents |
| `Delegate` | Hand off conversation |
| `Collaborate` | Multi-agent collaboration |
| `Ack` | Acknowledgment |
| `Error` | Error response |

## Configuration

Add to `config.csv`:

```csv
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
```

## See Also

- [Multi-Agent Keywords](./keywords-multi-agent.md)
- [DELEGATE TO BOT](./keyword-delegate-to-bot.md)