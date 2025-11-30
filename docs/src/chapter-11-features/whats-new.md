# What's New: Multi-Agent Features

General Bots has been enhanced with powerful multi-agent orchestration capabilities. This document summarizes the new features, keywords, and configuration options.

## Overview

The multi-agent update introduces:

- **Agent-to-Agent (A2A) Protocol** - Bots communicate and delegate tasks
- **Cross-Session User Memory** - User data persists across bots and sessions
- **Dynamic Model Routing** - Switch LLM models based on task requirements
- **Hybrid RAG Search** - Combined semantic + keyword search with RRF
- **Code Sandbox** - Safe Python/JavaScript/Bash execution
- **Agent Reflection** - Self-analysis for continuous improvement
- **SSE Streaming** - Real-time response streaming

## New BASIC Keywords

### Multi-Agent Keywords

| Keyword | Description |
|---------|-------------|
| `ADD BOT` | Add a bot with triggers, tools, or schedules |
| `DELEGATE TO BOT` | Send task to another bot and get response |
| `BROADCAST TO BOTS` | Send message to all session bots |
| `TRANSFER CONVERSATION` | Hand off conversation to another bot |
| `BOT REFLECTION` | Enable agent self-analysis |
| `BOT REFLECTION INSIGHTS` | Get reflection analysis results |

### Memory Keywords

| Keyword | Description |
|---------|-------------|
| `SET USER MEMORY` | Store data at user level (cross-bot) |
| `GET USER MEMORY` | Retrieve user-level data |
| `SET USER FACT` | Store a fact about the user |
| `USER FACTS` | Get all stored user facts |

### Model Routing Keywords

| Keyword | Description |
|---------|-------------|
| `USE MODEL` | Switch LLM model (fast/quality/code/auto) |

### Code Execution Keywords

| Keyword | Description |
|---------|-------------|
| `RUN PYTHON` | Execute Python in sandbox |
| `RUN JAVASCRIPT` | Execute JavaScript in sandbox |
| `RUN BASH` | Execute Bash script in sandbox |
| `RUN ... WITH FILE` | Run script from file |

## Quick Examples

### Multi-Agent Routing

```basic
' Router bot directing queries to specialists
HEAR userquery

category = LLM "Classify into billing, technical, sales: " + userquery

SWITCH category
    CASE "billing"
        result = DELEGATE userquery TO BOT "billing-bot"
    CASE "technical"
        result = DELEGATE userquery TO BOT "tech-bot"
    CASE "sales"
        result = DELEGATE userquery TO BOT "sales-bot"
END SWITCH

TALK result
```

### Cross-Bot User Memory

```basic
' Store user preference (accessible from any bot)
SET USER MEMORY "language", "pt-BR"
SET USER MEMORY "timezone", "America/Sao_Paulo"

' In another bot - retrieve preference
language = GET USER MEMORY("language")
IF language = "pt-BR" THEN
    TALK "Olá! Como posso ajudar?"
END IF
```

### Dynamic Model Selection

```basic
' Use fast model for simple queries
USE MODEL "fast"
greeting = LLM "Say hello"

' Switch to quality model for complex analysis
USE MODEL "quality"
analysis = LLM "Analyze market trends and provide recommendations"

' Let system decide automatically
USE MODEL "auto"
```

### Code Sandbox

```basic
' Execute Python for data processing
code = "
import json
data = [1, 2, 3, 4, 5]
print(json.dumps({'sum': sum(data), 'avg': sum(data)/len(data)}))
"
result = RUN PYTHON code
TALK "Statistics: " + result
```

### Agent Reflection

```basic
' Enable self-analysis
BOT REFLECTION true
BOT REFLECTION ON "conversation_quality"

' Later, check insights
insights = BOT REFLECTION INSIGHTS()
IF insights.qualityScore < 0.7 THEN
    SEND MAIL admin, "Low Quality Alert", insights.summary
END IF
```

## New Configuration Options

Add these to your `config.csv`:

### Multi-Agent (A2A)

```csv
name,value
a2a-enabled,true
a2a-timeout,30
a2a-max-hops,5
a2a-retry-count,3
```

### User Memory

```csv
name,value
user-memory-enabled,true
user-memory-max-keys,1000
user-memory-default-ttl,0
```

### Model Routing

```csv
name,value
model-routing-strategy,auto
model-default,fast
model-fast,DeepSeek-R1-Distill-Qwen-1.5B-Q3_K_M.gguf
model-quality,gpt-4
model-code,codellama-7b.gguf
```

### Hybrid RAG Search

```csv
name,value
rag-hybrid-enabled,true
rag-dense-weight,0.7
rag-sparse-weight,0.3
rag-reranker-enabled,true
```

### Code Sandbox

```csv
name,value
sandbox-runtime,lxc
sandbox-timeout,30
sandbox-memory-mb,512
sandbox-cpu-percent,50
sandbox-network,false
sandbox-python-packages,numpy,pandas,pillow
```

### Bot Reflection

```csv
name,value
reflection-enabled,true
reflection-interval,10
reflection-min-messages,3
reflection-model,quality
```

### SSE Streaming

```csv
name,value
sse-enabled,true
sse-heartbeat,30
sse-max-connections,1000
```

## Database Migrations

Run migrations to create the new tables:

```bash
cd botserver
cargo run -- migrate
```

### New Tables

| Table | Purpose |
|-------|---------|
| `user_memory` | Cross-session user preferences and facts |
| `user_preferences` | Per-session user settings |
| `a2a_messages` | Agent-to-Agent protocol messages |
| `user_memory_extended` | Enhanced memory with TTL |
| `kg_relationships` | Knowledge graph relationships |
| `conversation_summaries` | Episodic memory summaries |
| `conversation_costs` | LLM cost tracking |
| `openapi_tools` | OpenAPI tool tracking |
| `agent_reflections` | Agent self-analysis results |
| `chat_history` | Conversation history |
| `session_bots` | Multi-agent session tracking |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                    Multi-Agent System                        │
├─────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌──────────┐    A2A Protocol    ┌──────────┐               │
│  │ Router   │◄──────────────────►│ Billing  │               │
│  │ Bot      │                    │ Bot      │               │
│  └────┬─────┘    ┌──────────┐    └──────────┘               │
│       │          │ Support  │                                │
│       └─────────►│ Bot      │◄──────────────────┐           │
│                  └──────────┘                    │           │
│                                                  │           │
│  ┌──────────────────────────────────────────────┼──────┐    │
│  │              Shared Resources                 │      │    │
│  │  ┌────────────┐  ┌────────────┐  ┌──────────┴─┐    │    │
│  │  │ User       │  │ Hybrid     │  │ Model      │    │    │
│  │  │ Memory     │  │ RAG Search │  │ Router     │    │    │
│  │  └────────────┘  └────────────┘  └────────────┘    │    │
│  └───────────────────────────────────────────────────────┘  │
│                                                              │
└─────────────────────────────────────────────────────────────┘
```

## Design Principles

These features follow General Bots' core principles:

1. **BASIC-First** - All features accessible via simple BASIC keywords
2. **KISS** - Simple syntax, predictable behavior
3. **Pragmatismo** - Real-world utility over theoretical purity
4. **No Lock-in** - Local deployment, own your data

## Performance Considerations

| Feature | Impact | Mitigation |
|---------|--------|------------|
| A2A Protocol | Adds network latency | Use timeouts, local bots |
| User Memory | Database queries | Caching, indexing |
| Hybrid Search | Dual search paths | Results cached |
| Code Sandbox | Container startup | Warm containers |
| Reflection | LLM calls | Run periodically, not per-message |
| SSE Streaming | Connection overhead | Connection pooling |

## Migration Guide

### From Single-Bot to Multi-Agent

1. **Identify Specializations** - What tasks need dedicated bots?
2. **Create Specialist Bots** - Each with focused config
3. **Build Router** - Central bot to direct traffic
4. **Share Memory** - Move shared data to User Memory
5. **Test Delegation** - Verify communication paths

### Upgrading Existing Bots

1. Run database migrations
2. Add new config options as needed
3. Existing keywords continue to work unchanged
4. Gradually adopt new features

## See Also

- [Multi-Agent Orchestration](./multi-agent-orchestration.md) - Complete guide
- [Memory Management](./memory-management.md) - Memory deep dive
- [Hybrid RAG Search](./hybrid-search.md) - Search configuration
- [Keywords Reference](../chapter-06-gbdialog/keywords.md) - All keywords
- [Configuration Parameters](../chapter-08-config/parameters.md) - All config options