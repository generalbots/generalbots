# USE MCP 🟡 BETA

| Form | Registered? | Where |
|---|---|---|
| `USE MCP …` | ❌ no | advertised by `get_all_keywords()` only |
| `USE TOOL $expr$` | ✅ yes | `botserver/crates/botbasic_ai/src/keywords/use_tool.rs` |

There is no `USE MCP` keyword. MCP servers are declared per tool in a
`{tool}.mcp.json` manifest under the bot's work directory, and the tool is then
invoked like any other tool — `USE TOOL "<tool>"`. `use_tool.rs` refuses the
call with `Tool '<name>' is not available. .mcp.json file not found.` when the
manifest is missing, which is the failure a `USE MCP …` script produces today.

The generated [keyword reference](./generated/index.md) lists `USE MCP` under
"Advertised but not registered".

## Invoking an MCP-backed tool

```basic
USE TOOL "github"
```

## See also

- [USE TOOL](./keyword-use-tool.md) — the implemented form
- [Generated BASIC reference](./generated/index.md) — registration status and source line
