# USE MCP

> **Invoke MCP Server Tools from BASIC**

---

## Overview

The `USE MCP` keyword allows you to invoke tools from Model Context Protocol (MCP) servers directly in your BASIC scripts. MCP servers extend your bot's capabilities by providing access to external systems like databases, filesystems, APIs, and more.

---

## Syntax

```bas
result = USE MCP "server_name", "tool_name", {parameters}
```

| Parameter | Description |
|-----------|-------------|
| `server_name` | Name of the MCP server (as defined in `mcp.csv`) |
| `tool_name` | Name of the tool to invoke |
| `parameters` | JSON object with tool parameters |

---

## The mcp.csv File

MCP servers are configured by adding entries to the `mcp.csv` file in your bot's `.gbai` folder:

```
mybot.gbai/
├── mybot.gbdialog/     # BASIC scripts
├── mybot.gbdrive/      # Files and documents
├── config.csv          # Bot configuration
├── attendant.csv       # Attendant configuration
└── mcp.csv             # MCP server definitions
```

When botserver starts, it reads the `mcp.csv` file and loads all server configurations. These servers become available to Tasks and can be invoked using the `USE MCP` keyword.

---

## mcp.csv Format

The CSV file has the following columns:

| Column | Required | Description |
|--------|----------|-------------|
| `name` | Yes | Unique server identifier (used in `USE MCP` calls) |
| `type` | Yes | Connection type: `stdio`, `http`, `websocket`, `tcp` |
| `command` | Yes | For stdio: command to run. For http/ws: URL |
| `args` | No | Command arguments (space-separated) or empty |
| `description` | No | Human-readable description |
| `enabled` | No | `true` or `false` (default: `true`) |
| `auth_type` | No | Authentication type: `none`, `api_key`, `bearer` |
| `auth_env` | No | Environment variable name for auth credential |
| `risk_level` | No | `safe`, `low`, `medium`, `high`, `critical` |
| `requires_approval` | No | `true` or `false` (default: `false`) |

### Example mcp.csv

```csv
name,type,command,args,description,enabled
# MCP Server Configuration
# Lines starting with # are comments
filesystem,stdio,npx,"-y @modelcontextprotocol/server-filesystem /data",Access local files,true
github,stdio,npx,"-y @modelcontextprotocol/server-github",GitHub API,true,bearer,GITHUB_TOKEN
postgres,stdio,npx,"-y @modelcontextprotocol/server-postgres",Database queries,false
slack,stdio,npx,"-y @modelcontextprotocol/server-slack",Slack messaging,true,bearer,SLACK_BOT_TOKEN
myapi,http,https://api.example.com/mcp,,Custom API,true,api_key,MY_API_KEY
```

---

## Connection Types

### stdio (Local Process)

For MCP servers that run as local processes via npx, node, python, etc:

```csv
filesystem,stdio,npx,"-y @modelcontextprotocol/server-filesystem /data",File access,true
```

The `command` is the executable, and `args` contains the arguments.

### http (REST API)

For HTTP-based MCP servers:

```csv
myapi,http,https://api.example.com/mcp,,REST API server,true
```

The `command` is the URL endpoint.

### websocket

For WebSocket connections:

```csv
realtime,websocket,wss://ws.example.com/mcp,,Real-time server,true
```

### tcp

For raw TCP connections:

```csv
legacy,tcp,localhost:9000,,Legacy TCP server,true
```

Format: `host:port` in the command column.

---

## Authentication

### API Key

```csv
myapi,http,https://api.example.com,,API Server,true,api_key,MY_API_KEY
```

The environment variable `MY_API_KEY` will be read and sent as `X-API-Key` header.

### Bearer Token

```csv
github,stdio,npx,"-y @modelcontextprotocol/server-github",GitHub,true,bearer,GITHUB_TOKEN
```

The environment variable `GITHUB_TOKEN` will be used as a Bearer token.

> **Security**: Authentication credentials are read from environment variables. Never put actual secrets in mcp.csv.

---

## Examples

### Read a File

```bas
' Read a file using filesystem MCP server
content = USE MCP "filesystem", "read_file", {"path": "/data/config.json"}
TALK "File contents: " + content
```

### Query Database

```bas
' Query PostgreSQL using database MCP server
results = USE MCP "postgres", "query", {"sql": "SELECT * FROM users LIMIT 10"}
FOR EACH row IN results
    TALK row.name + " - " + row.email
NEXT
```

### Search GitHub

```bas
' Search GitHub repositories
repos = USE MCP "github", "search_repositories", {"query": "general bots language:rust"}
TALK "Found " + repos.length + " repositories"
```

### Send Slack Message

```bas
' Send message to Slack channel
USE MCP "slack", "send_message", {
    "channel": "#general",
    "text": "Hello from General Bots!"
}
```

### Create GitHub Issue

```bas
' Create an issue (requires approval if configured)
issue = USE MCP "github", "create_issue", {
    "owner": "myorg",
    "repo": "myproject",
    "title": "Bug: Login not working",
    "body": "Users cannot log in with SSO"
}
TALK "Created issue #" + issue.number
```

---

## Related Keywords

### MCP LIST TOOLS

List available tools from an MCP server:

```bas
tools = MCP LIST TOOLS "filesystem"
FOR EACH tool IN tools
    TALK tool.name + ": " + tool.description
NEXT
```

### MCP INVOKE

Alternative syntax for direct tool invocation:

```bas
result = MCP INVOKE "filesystem.read_file", {"path": "/data/file.txt"}
```

---

## Risk Levels

Tools have risk levels that determine how they're handled:

| Level | Description | Behavior |
|-------|-------------|----------|
| **safe** | Read-only, no side effects | Always allowed |
| **low** | Minor changes, reversible | Usually allowed |
| **medium** | Significant changes | May require approval |
| **high** | Destructive or irreversible | Requires approval |
| **critical** | System-level changes | Always requires approval |

When `requires_approval` is set to `true`, the task will pause and wait for human approval before executing the tool.

---

## Tool Discovery

MCP tools are discovered automatically when the server starts. You can see available tools in:

1. **Sources UI** → MCP Servers tab → View Tools
2. **Sources UI** → LLM Tools tab → MCP Tools section
3. **BASIC** → `MCP LIST TOOLS "server_name"`

---

## Available MCP Servers

Popular MCP servers you can use:

| Server | Package | Description |
|--------|---------|-------------|
| **Filesystem** | `@modelcontextprotocol/server-filesystem` | File operations |
| **GitHub** | `@modelcontextprotocol/server-github` | GitHub API |
| **PostgreSQL** | `@modelcontextprotocol/server-postgres` | Database queries |
| **SQLite** | `@modelcontextprotocol/server-sqlite` | SQLite database |
| **Slack** | `@modelcontextprotocol/server-slack` | Slack messaging |
| **Puppeteer** | `@modelcontextprotocol/server-puppeteer` | Browser automation |
| **Brave Search** | `@modelcontextprotocol/server-brave-search` | Web search |

See [modelcontextprotocol.io](https://modelcontextprotocol.io) for more servers.

---

## Integration with Tasks

When you add MCP servers to your bot via `mcp.csv`, their tools become available to the Autonomous Task system. The AI can:

1. **Discover tools** from your MCP servers
2. **Plan execution** using MCP tools alongside BASIC keywords
3. **Request approval** for high-risk operations
4. **Execute tools** and process results

Example task flow:

```
User: "Read the config file and update the database accordingly"

AI Plan:
1. USE MCP "filesystem", "read_file" → Read config.json
2. Parse JSON configuration
3. USE MCP "postgres", "query" → Update database
4. Report results
```

---

## Troubleshooting

### Server Not Found

```
Error: MCP server 'myserver' not found
```

- Check that `mcp.csv` exists in your `.gbai` folder
- Verify the server name matches exactly (case-sensitive)
- Ensure `enabled` is not set to `false`
- Reload servers in Sources UI

### Connection Failed

```
Error: Failed to connect to MCP server
```

- Verify the command/URL is correct
- Check that required packages are installed (`npm install`)
- Ensure environment variables are set for authentication
- Test the server manually first

### Tool Not Available

```
Error: Tool 'unknown_tool' not found on server 'myserver'
```

- List available tools with `MCP LIST TOOLS`
- Check tool name spelling
- Verify server is properly started

### Authentication Error

```
Error: Authentication failed for MCP server
```

- Check environment variables are set correctly
- Verify credentials are valid
- Ensure auth type matches server requirements

---

## Best Practices

1. **Use environment variables** for all credentials
2. **Set appropriate risk levels** for tools that modify data
3. **Enable approval** for destructive operations
4. **Comment your mcp.csv** with `#` lines to document servers
5. **Test locally** before deploying
6. **Start with enabled=false** for new servers until tested

---

## See Also

- [Sources](../07-user-interface/apps/sources.md) - Managing MCP servers in the UI
- [Autonomous Tasks](../07-gbapp/autonomous-tasks.md) - How Tasks use MCP tools
- [MCP Format](../08-rest-api-tools/mcp-format.md) - MCP tool definition format
- [LLM Tools](../08-rest-api-tools/README.md) - All available tool types