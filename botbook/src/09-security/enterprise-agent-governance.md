# Enterprise Agent Governance 🟡 BETA

> **Centralized monitoring, token auditing, and safety guards for AI agents**

## Overview

Enterprise Agent Governance provides administrators with a centralized control room to observe, audit, and secure all AI agents operating across the workspace. It combines real-time telemetry, dynamic security guards, script execution isolation, and a live dashboard with kill-switch capabilities.

## Architecture

```
┌─────────────────────────────────────────────────┐
│              Governance Dashboard (HTMX)         │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ Live     │ │ Token    │ │ Kill Switches    │ │
│  │ Metrics  │ │ Auditing │ │ (bot-level)      │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
└──────────────────────┬──────────────────────────┘
                       │ SSE / WebSocket
┌──────────────────────▼──────────────────────────┐
│              Metrics Ingestion Pipeline          │
│  ┌──────────┐ ┌──────────┐ ┌──────────────────┐ │
│  │ LLM Call │ │ Tool     │ │ Script           │ │
│  │ Intercept│ │ Execution│ │ Guard            │ │
│  └──────────┘ └──────────┘ └──────────────────┘ │
└──────────────────────────────────────────────────┘
```

## Telemetry Ingestion

A middleware intercepts all LLM calls and tool executions across the system, ingesting:

| Metric | Description |
|--------|-------------|
| Timestamp | Event time |
| Bot ID | Which bot triggered the event |
| Session ID | User session context |
| Model | LLM model used |
| Prompt tokens | Input token count |
| Completion tokens | Output token count |
| Tool name | Executed tool/script |
| Latency | Execution duration |

Metrics are persisted to the database and pushed to Valkey for live dashboard rendering via SSE.

## Dynamic Security Guards

The script guard (`script_guard.rs`) inspects Rhai BASIC scripts before execution, detecting 17+ dangerous patterns:

| Pattern | Detection |
|---------|-----------|
| Secret key leakage | Reads from `/tmp/vault-token-gb`, env vars |
| Shell injection | `Command::new()`, shell execution |
| File system escape | Writes outside `.gbdrive/` |
| Network probing | Internal IP/port scanning |
| Resource exhaustion | Infinite loops, excessive allocations |

Blocked events are logged to an immutable `security_audit_logs` table.

## Administrator Console

### Dashboard

The governance dashboard at `/suite/governance` provides:

- **Live Metrics**: Real-time WebSocket throughput and token consumption costs grouped by bot and department
- **Kill Switches**: Immediately disable specific bots or revoke external tool access
- **Security Incidents**: Real-time alarms via Server-Sent Events (SSE)
- **Audit Log**: Searchable history of all security events

### Kill Switch

When a bot is disabled via the kill switch:

1. WebSocket connections for that bot are rejected
2. Pending tool executions are cancelled
3. An incident is logged to the audit table
4. Administrators are notified via the dashboard

## Configuration

| config.csv key | Description | Default |
|----------------|-------------|---------|
| `governance-enabled` | Enable governance features | `true` |
| `governance-audit-log` | Enable immutable audit log | `true` |
| `governance-alert-email` | Email for critical alerts | (none) |

## Feature Flag

Enable with `monitoring` feature flag:
```toml
botserver = { features = ["monitoring"] }
```

## Security

- Metric ingestion failures never block bot conversation paths — background threads with bounded channels
- User message content is redacted from monitoring console logs unless detailed auditing is explicitly enabled
- Kill switch fail-safe: DB pool overflow falls back to local file logging
