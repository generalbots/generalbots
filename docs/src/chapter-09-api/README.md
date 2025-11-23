## Tooling
The **Tooling** chapter lists all built‑in keywords and their one‑line descriptions.

| Keyword | Description |
|---------|-------------|
| `TALK` | Send a message to the user. |
| `HEAR` | Receive user input. |
| `LLM` | Invoke the configured large‑language‑model. |
| `USE TOOL` | Register a custom tool at runtime. |
| `GET` | Retrieve a value from the session store. |
| `SET` | Store a value in the session store. |
| `FORMAT` | Format numbers, dates, or text. |
| `USE KB` | Create a new knowledge‑base collection. |
| `SET KB` | Switch the active knowledge‑base. |
| `ADD WEBSITE` | Crawl and index a website. |
| `CALL` | Invoke a registered tool synchronously. |
| `CALL_ASYNC` | Invoke a tool asynchronously. |
| `LIST TOOLS` | List all available tools for the session. |
| `REMOVE TOOL` | Unregister a tool. |
| `CLEAR TOOLS` | Remove all custom tools. |
| `FIRST` / `LAST` | Access first/last element of a collection. |
| `FOR EACH` / `EXIT FOR` | Iterate over collections. |
| `IF` / `ELSE` / `ENDIF` | Conditional execution. |
| `WHILE` / `ENDWHILE` | Loop while a condition holds. |
| `REPEAT` / `UNTIL` | Loop until a condition is met. |
| `WAIT` | Pause execution for a given duration. |
| `ON` | Register an event handler. |
| `SET SCHEDULE` | Define a scheduled task. |
| `PRINT` | Output debugging information. |
| `GET BOT MEMORY` / `SET BOT MEMORY` | Access bot‑wide memory store. |
| `CREATE SITE` | Create a new website entry in a knowledge base. |
| `CREATE DRAFT` | Generate a draft document. |
| `WEBSITE OF` | Reference a website object. |
| `FIND` | Search within collections. |
| `GET` / `SET` | Generic getters/setters for variables. |
| `...` | See the full keyword list in `chapter-05/keywords.md`. |

## See Also

- [External APIs](./external-apis.md) - Integrating with external services
- [GET Integration](./get-integration.md) - Using the GET command
- [Chapter 2: Packages](../chapter-02/README.md) - Understanding bot components
- [Chapter 3: KB and Tools](../chapter-03/kb-and-tools.md) - Knowledge base and tool system
- [Chapter 5: BASIC Reference](../chapter-05/README.md) - Complete command reference
- [Chapter 6: Extensions](../chapter-06/README.md) - Extending BotServer
- [Chapter 9: Advanced Topics](../chapter-09/README.md) - Advanced integration patterns
- [Chapter 12: Web API](../chapter-12/README.md) - REST and WebSocket APIs
