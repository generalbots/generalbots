# Keyword Reference

This section lists every BASIC keyword implemented in the GeneralBots engine. Each keyword page includes:

* **Syntax** – Exact command format
* **Parameters** – Expected arguments
* **Description** – What the keyword does
* **Example** – A short snippet showing usage

The source code for each keyword lives in `src/basic/keywords/`. Only the keywords listed here exist in the system.

## Important: Case Insensitivity

**All variables in General Bots BASIC are case-insensitive.** The preprocessor normalizes variable names to lowercase automatically.

```basic
' These all refer to the same variable
host = "https://api.example.com"
result = GET Host + "/endpoint"
TALK HOST
```

Keywords are also case-insensitive but conventionally written in UPPERCASE:

```basic
' Both work identically
TALK "Hello"
talk "Hello"
```

## Configuration Variables (param-*)

Variables defined with `param-` prefix in `config.csv` are automatically available in scripts without the prefix:

```csv
name,value
param-host,https://api.example.com
param-limit,100
param-pages,50
```

```basic
' Access directly (lowercase, no param- prefix)
result = GET host + "/items?limit=" + limit
```

See [Script Execution Flow](./script-execution-flow.md) for complete details.

---

## Complete Keyword List (Flat Reference)

| Keyword | Category | Description |
|---------|----------|-------------|
| `ADD BOT` | Multi-Agent | Add a bot to the current session with triggers |
| `ADD MEMBER` | Communication | Add member to a group |
| `ADD SUGGESTION` | UI | Add clickable suggestion button |
| `ADD TOOL` | Tools | Register a tool for the session |
| `AGGREGATE` | Data | Perform SUM, AVG, COUNT, MIN, MAX operations |
| `BOOK` | Special | Book an appointment |
| `BOT REFLECTION` | Multi-Agent | Enable agent self-analysis and improvement |
| `BROADCAST TO BOTS` | Multi-Agent | Send message to all bots in session |
| `CLEAR HEADERS` | HTTP | Clear all HTTP headers |
| `CLEAR KB` | Knowledge | Unload knowledge base from session |
| `CLEAR SUGGESTIONS` | UI | Remove all suggestion buttons |
| `CLEAR TOOLS` | Tools | Remove all registered tools |
| `COMPRESS` | Files | Create ZIP archive |
| `COPY` | Files | Copy a file |
| `CREATE DRAFT` | Communication | Create email draft |
| `CREATE SITE` | Tools | Generate a website |
| `CREATE TASK` | Tools | Create a task |
| `DELEGATE TO BOT` | Multi-Agent | Send task to another bot |
| `DELETE` | Data | Delete records from table |
| `DELETE FILE` | Files | Delete a file |
| `DELETE HTTP` | HTTP | Send HTTP DELETE request |
| `DOWNLOAD` | Files | Download file from URL |
| `EXIT FOR` | Control | Exit loop early |
| `EXTRACT` | Files | Extract ZIP archive |
| `FILL` | Data | Fill template with data |
| `FILTER` | Data | Filter records by condition |
| `FIND` | Data | Search in files or KB |
| `FIRST` | Data | Get first element |
| `FOR EACH ... NEXT` | Control | Loop through items |
| `FORMAT` | Data | Format strings and dates |
| `GENERATE PDF` | Files | Generate PDF from template |
| `GET` | Variables | Get variable or API data |
| `GET BOT MEMORY` | Memory | Retrieve bot-level persisted data |
| `GET USER MEMORY` | Memory | Retrieve user-level persisted data (cross-bot) |
| `GRAPHQL` | HTTP | Execute GraphQL query |
| `GROUP BY` | Data | Group data by field |
| `HEAR` | Dialog | Get input from user |
| `IF ... THEN ... ELSE ... END IF` | Control | Conditional logic |
| `INSERT` | Data | Insert new record |
| `INSTR` | String | Find position of substring |
| `IS NUMERIC` | String | Check if value is numeric |
| `JOIN` | Data | Join two datasets |
| `LAST` | Data | Get last element |
| `LIST` | Files | List directory contents |
| `LLM` | AI | Query language model |
| `MAP` | Data | Map field names |
| `MERGE` | Data | Merge data into table |
| `MERGE PDF` | Files | Merge multiple PDFs |
| `MOVE` | Files | Move or rename file |
| `ON` | Events | Event handler |
| `PATCH` | HTTP | Send HTTP PATCH request |
| `PIVOT` | Data | Create pivot table |
| `POST` | HTTP | Send HTTP POST request |
| `PRINT` | Debug | Debug output to console |
| `PUT` | HTTP | Send HTTP PUT request |
| `READ` | Files | Read file content |
| `REMEMBER` | Memory | Store user-specific memory |
| `RUN BASH` | Code Execution | Execute Bash script in sandbox |
| `RUN JAVASCRIPT` | Code Execution | Execute JavaScript in sandbox |
| `RUN PYTHON` | Code Execution | Execute Python code in sandbox |
| `SAVE` | Data | Save data to table (upsert) |
| `SAVE FROM UNSTRUCTURED` | Data | Extract structured data from text |
| `SEND MAIL` | Communication | Send email |
| `SET` | Variables | Set variable value |
| `SET BOT MEMORY` | Memory | Persist data at bot level |
| `SET CONTEXT` | AI | Add context for LLM |
| `SET HEADER` | HTTP | Set HTTP header |
| `SET SCHEDULE` | Events | Schedule script execution |
| `SET USER` | Session | Set user context |
| `SET USER FACT` | Memory | Store a fact about the user |
| `SET USER MEMORY` | Memory | Persist data at user level (cross-bot) |
| `SOAP` | HTTP | Execute SOAP API call |
| `SWITCH ... CASE ... END SWITCH` | Control | Switch statement |
| `SYNCHRONIZE` | Data | Sync API data to table (planned) |
| `TALK` | Dialog | Send message to user |
| `TRANSFER CONVERSATION` | Multi-Agent | Hand off conversation to another bot |
| `UPDATE` | Data | Update existing records |
| `USE MODEL` | AI | Switch LLM model for subsequent operations |
| `USER FACTS` | Memory | Get all stored user facts |
| `UPLOAD` | Files | Upload file to storage |
| `USE KB` | Knowledge | Load knowledge base |
| `USE TOOL` | Tools | Register tool definition |
| `USE WEBSITE` | Knowledge | Associate website with session |
| `WAIT` | Control | Pause execution |
| `WEATHER` | Special | Get weather information |
| `WEBHOOK` | Events | Create webhook endpoint |
| `WHILE ... WEND` | Control | While loop |
| `WRITE` | Files | Write content to file |

---

## Keywords by Category

### Core Dialog Keywords

| Keyword | Syntax | Description |
|---------|--------|-------------|
| TALK | `TALK "message"` | Send message to user |
| HEAR | `HEAR variable` or `HEAR variable AS TYPE` | Get input from user |
| WAIT | `WAIT seconds` | Pause execution |
| PRINT | `PRINT "debug message"` | Debug output to console |

### Variable & Memory

| Keyword | Syntax | Description |
|---------|--------|-------------|
| SET | `SET variable = value` or `let variable = value` | Set variable value |
| GET | `result = GET "path"` | Get variable or fetch data |
| SET BOT MEMORY | `SET BOT MEMORY "key", value` | Persist data at bot level |
| GET BOT MEMORY | `value = GET BOT MEMORY("key")` | Retrieve persisted data |
| SET USER MEMORY | `SET USER MEMORY "key", value` | Persist data at user level (cross-bot) |
| GET USER MEMORY | `value = GET USER MEMORY("key")` | Retrieve user-level data |
| SET USER FACT | `SET USER FACT "key", value` | Store fact about user |
| USER FACTS | `facts = USER FACTS()` | Get all user facts |
| REMEMBER | `REMEMBER "key", value` | Store user-specific memory |

### AI & Context

| Keyword | Syntax | Description |
|---------|--------|-------------|
| LLM | `result = LLM "prompt"` | Query language model |
| SET CONTEXT | `SET CONTEXT "name" AS "value"` | Add context for LLM |
| SET USER | `SET USER userid` | Set user context |
| USE MODEL | `USE MODEL "modelname"` | Switch LLM model (fast/quality/code/auto) |

### Multi-Agent Orchestration

| Keyword | Syntax | Description |
|---------|--------|-------------|
| ADD BOT | `ADD BOT "name" TRIGGER ON "keywords"` | Add bot with triggers |
| DELEGATE TO BOT | `result = DELEGATE "message" TO BOT "name"` | Send task to bot |
| BROADCAST TO BOTS | `BROADCAST "message" TO BOTS` | Message all bots |
| TRANSFER CONVERSATION | `TRANSFER CONVERSATION TO "botname"` | Hand off to bot |
| BOT REFLECTION | `BOT REFLECTION true` | Enable self-analysis |
| BOT REFLECTION INSIGHTS | `insights = BOT REFLECTION INSIGHTS()` | Get analysis results |

### Code Execution (Sandboxed)

| Keyword | Syntax | Description |
|---------|--------|-------------|
| RUN PYTHON | `result = RUN PYTHON "code"` | Execute Python in sandbox |
| RUN JAVASCRIPT | `result = RUN JAVASCRIPT "code"` | Execute JS in sandbox |
| RUN BASH | `result = RUN BASH "code"` | Execute Bash in sandbox |
| RUN ... WITH FILE | `result = RUN PYTHON WITH FILE "script.py"` | Run script file |

### Knowledge Base

| Keyword | Syntax | Description |
|---------|--------|-------------|
| USE KB | `USE KB "kbname"` | Load knowledge base |
| CLEAR KB | `CLEAR KB` or `CLEAR KB "kbname"` | Unload knowledge base |
| USE WEBSITE | `USE WEBSITE "url"` | Associate website with session |
| FIND | `result = FIND "file", "filter"` | Search in files or KB |

### Tools & Automation

| Keyword | Syntax | Description |
|---------|--------|-------------|
| ADD TOOL | `ADD TOOL "toolname"` | Register tool for session |
| USE TOOL | `USE TOOL "toolname"` | Load tool definition |
| CLEAR TOOLS | `CLEAR TOOLS` | Remove all registered tools |
| CREATE TASK | `CREATE TASK "title", "description"` | Create a task |
| CREATE SITE | `CREATE SITE "alias", "template", "prompt"` | Generate a website |
| CREATE DRAFT | `CREATE DRAFT "to", "subject", "body"` | Create email draft |

### UI & Interaction

| Keyword | Syntax | Description |
|---------|--------|-------------|
| ADD SUGGESTION | `ADD SUGGESTION "key" AS "display text"` | Add clickable button |
| CLEAR SUGGESTIONS | `CLEAR SUGGESTIONS` | Remove all buttons |

### Data Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| SAVE | `SAVE "table", var1, var2, var3` | Save data (upsert) |
| INSERT | `result = INSERT "table", data` | Insert new record |
| UPDATE | `rows = UPDATE "table", "filter", data` | Update records |
| DELETE | `rows = DELETE "table", "filter"` | Delete records |
| MERGE | `result = MERGE "table", data, "key"` | Merge data into table |
| FILTER | `result = FILTER data, "condition"` | Filter records |
| AGGREGATE | `result = AGGREGATE "SUM", data, "field"` | Aggregate operations |
| JOIN | `result = JOIN left, right, "key"` | Join datasets |
| PIVOT | `result = PIVOT data, "row", "value"` | Create pivot table |
| GROUP BY | `result = GROUP BY data, "field"` | Group data |
| SYNCHRONIZE | `SYNCHRONIZE endpoint, table, key, pageVar, limitVar` | Sync API to table |
| MAP | `result = MAP data, "old->new"` | Map field names |
| FILL | `result = FILL data, template` | Fill template |
| FIRST | `result = FIRST collection` | Get first element |
| LAST | `result = LAST collection` | Get last element |
| FORMAT | `result = FORMAT value AS "pattern"` | Format strings/dates |

### File Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| READ | `content = READ "path"` | Read file content |
| WRITE | `WRITE "path", content` | Write to file |
| DELETE FILE | `DELETE FILE "path"` | Delete a file |
| COPY | `COPY "source", "destination"` | Copy a file |
| MOVE | `MOVE "source", "destination"` | Move/rename file |
| LIST | `files = LIST "path/"` | List directory |
| UPLOAD | `url = UPLOAD file, "path"` | Upload file |
| DOWNLOAD | `path = DOWNLOAD "url", "local"` | Download file |
| COMPRESS | `archive = COMPRESS files, "name.zip"` | Create ZIP |
| EXTRACT | `files = EXTRACT "archive.zip", "dest/"` | Extract ZIP |
| GENERATE PDF | `result = GENERATE PDF "template", data, "output.pdf"` | Generate PDF |
| MERGE PDF | `result = MERGE PDF files, "merged.pdf"` | Merge PDFs |

### HTTP & API Operations

| Keyword | Syntax | Description |
|---------|--------|-------------|
| POST | `result = POST "url", data` | HTTP POST request |
| PUT | `result = PUT "url", data` | HTTP PUT request |
| PATCH | `result = PATCH "url", data` | HTTP PATCH request |
| DELETE HTTP | `result = DELETE HTTP "url"` | HTTP DELETE request |
| SET HEADER | `SET HEADER "name", "value"` | Set HTTP header |
| CLEAR HEADERS | `CLEAR HEADERS` | Clear all headers |
| GRAPHQL | `result = GRAPHQL "url", "query", vars` | GraphQL query |
| SOAP | `result = SOAP "wsdl", "operation", params` | SOAP call |

### Flow Control

| Keyword | Syntax | Description |
|---------|--------|-------------|
| IF...THEN...ELSE | `IF condition THEN ... ELSE ... END IF` | Conditional |
| FOR EACH...NEXT | `FOR EACH item IN collection ... NEXT item` | Loop |
| EXIT FOR | `EXIT FOR` | Exit loop early |
| `WHILE...WEND` | `WHILE condition ... WEND` | While loop |
| `SWITCH...CASE` | `SWITCH value CASE x ... END SWITCH` | Switch statement |
| `REPORT` | `SEND EMAIL admin, REPORT` | Access sync statistics |
| `RESET REPORT` | `RESET REPORT` | Clear sync statistics |

### Events & Scheduling

| Keyword | Syntax | Description |
|---------|--------|-------------|
| ON | `ON "event" CALL handler` | Event handler |
| SET SCHEDULE | `SET SCHEDULE "cron"` | Schedule execution |
| WEBHOOK | `WEBHOOK "endpoint"` | Create webhook |

### Communication

| Keyword | Syntax | Description |
|---------|--------|-------------|
| SEND MAIL | `SEND MAIL "to", "subject", "body"` | Send email |
| ADD MEMBER | `ADD MEMBER "email", "group"` | Add to group |

### Special Functions

| Keyword | Syntax | Description |
|---------|--------|-------------|
| BOOK | `BOOK "appointment"` | Book appointment |
| WEATHER | `weather = WEATHER "location"` | Get weather |
| INSTR | `pos = INSTR(string, search)` | Find substring |
| IS NUMERIC | `result = IS NUMERIC(value)` | Check if numeric |
| SAVE FROM UNSTRUCTURED | `data = SAVE FROM UNSTRUCTURED text, schema` | Extract structured data |

---

## Syntax Rules

### DO ✅

```basic
' Variable names (no underscores in names)
let ticketnumber = "TKT001"
let useremail = "user@example.com"

' SAVE with field names = variable names
SAVE "table.csv", ticketnumber, useremail, status

' Keywords with spaces
SET BOT MEMORY "last_ticket", ticketnumber
SET CONTEXT "name" AS "description"
ADD SUGGESTION "key" AS "Display text"
CLEAR SUGGESTIONS
USE KB "myknowledge"
USE TOOL "mytool"

' GET BOT MEMORY as function
let lastticket = GET BOT MEMORY("last_ticket")
```

### DON'T ❌

```basic
' NO: Complex object operations
SET object.field = value  ' WRONG
SAVE "table", object.id, object  ' WRONG

' NO: IF for input validation (use HEAR AS TYPE instead)
IF value = "" THEN  ' OK for logic, but for input use:
HEAR value AS STRING  ' Better - validates input type
```

---

## Prompt Blocks

Special multi-line blocks for AI configuration and formatted output:

| Block | Purpose | Documentation |
|-------|---------|---------------|
| `BEGIN SYSTEM PROMPT ... END SYSTEM PROMPT` | Define AI persona, rules, capabilities | [Prompt Blocks](./prompt-blocks.md) |
| `BEGIN TALK ... END TALK` | Formatted multi-line messages with Markdown | [Prompt Blocks](./prompt-blocks.md) |

```basic
BEGIN SYSTEM PROMPT
You are a helpful assistant for AcmeStore.
Rules:
1. Always be polite
2. Never discuss competitors
END SYSTEM PROMPT

BEGIN TALK
**Welcome!** 🎉

I can help you with:
• Orders
• Tracking
• Returns
END TALK
```

---

## Script Structure

### No MAIN Function

Scripts execute from line 1 - no `MAIN` or entry point needed:

```basic
' ✅ CORRECT - Start directly
TALK "Hello!"
ADD TOOL "my-tool"

' ❌ WRONG - Don't use MAIN
SUB MAIN()
    TALK "Hello"
END SUB
```

### SUB and FUNCTION for Reuse

Use for helper code within tools, not as entry points:

```basic
FUNCTION CalculateTotal(price, quantity)
    RETURN price * quantity
END FUNCTION

SUB NotifyAdmin(message)
    SEND EMAIL admin1, message
END SUB

' Execution starts here
total = CalculateTotal(19.99, 3)
CALL NotifyAdmin("Order processed")
```

See [Script Execution Flow](./script-execution-flow.md) for entry points and lifecycle.

---

## Notes

- Keywords are case-insensitive (TALK = talk = Talk)
- Variables are case-insensitive (host = HOST = Host)
- String parameters can use double quotes or single quotes
- Comments start with REM or '
- Line continuation uses underscore (_)
- Objects are created with `#{ key: value }` syntax
- Arrays use `[item1, item2, ...]` syntax
- param-* config values become global variables

---

## See Also

- [Script Execution Flow](./script-execution-flow.md) - Entry points and lifecycle
- [Prompt Blocks](./prompt-blocks.md) - BEGIN SYSTEM PROMPT & BEGIN TALK
- [Basics](./basics.md) - Core concepts
- [Examples](./examples-consolidated.md) - Real-world patterns