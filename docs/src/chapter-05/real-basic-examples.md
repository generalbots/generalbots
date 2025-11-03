# Real BASIC Keyword Examples in GeneralBots

This section provides **authentic examples** of BASIC commands implemented in the GeneralBots system.  
All examples are derived directly from the source code under `src/basic/keywords/`.

---

## Website Knowledge Base

### `ADD_WEBSITE`
Registers and indexes a website into the bot’s knowledge base.

```basic
ADD_WEBSITE "https://example.com"
```

**Description:**  
Crawls the specified website, extracts text content, and stores it in a Qdrant vector database for semantic search.  
If the `web_automation` feature is disabled, the command validates the URL format only.

---

## Knowledge Base Management

### `SET_KB`
Sets the active knowledge base for the current user session.

```basic
SET_KB "marketing_data"
```

**Description:**  
Links the bot’s context to a specific KB collection, enabling focused queries and responses.

### `ADD_KB`
Adds a new knowledge base collection.

```basic
ADD_KB "customer_feedback"
```

**Description:**  
Creates a new KB collection in Qdrant and prepares it for document indexing.

---

## Communication

### `HEAR_TALK`
Handles conversational input and output between the bot and user.

```basic
HEAR_TALK "Hello, bot!"
```

**Description:**  
Triggers the bot’s response pipeline, processing user input and generating replies using the active LLM model.

### `PRINT`
Outputs text or variable content to the console or chat.

```basic
PRINT "Task completed successfully."
```

**Description:**  
Displays messages or results during script execution.

---

## Context and Tools

### `SET_CONTEXT`
Defines the current operational context for the bot.

```basic
SET_CONTEXT "sales_mode"
```

**Description:**  
Switches the bot’s internal logic to a specific context, affecting how commands are interpreted.

### `ADD_TOOL`
Registers a new tool for automation.

```basic
ADD_TOOL "email_sender"
```

**Description:**  
Adds a tool to the bot’s environment, enabling extended functionality such as sending emails or processing files.

### `REMOVE_TOOL`
Removes a previously registered tool.

```basic
REMOVE_TOOL "email_sender"
```

**Description:**  
Unregisters a tool from the bot’s active environment.

---

## Scheduling and User Management

### `SET_SCHEDULE`
Defines a scheduled task for automation.

```basic
SET_SCHEDULE "daily_report"
```

**Description:**  
Creates a recurring automation trigger based on time or event conditions.

### `SET_USER`
Sets the active user context.

```basic
SET_USER "john_doe"
```

**Description:**  
Associates the current session with a specific user identity.

---

## Utility Commands

### `WAIT`
Pauses execution for a specified duration.

```basic
WAIT 5
```

**Description:**  
Delays script execution for 5 seconds.

### `FIND`
Searches for data or keywords within the current context.

```basic
FIND "project_status"
```

**Description:**  
Queries the bot’s memory or KB for matching entries.

---

## Summary

All examples above are **real commands** implemented in the GeneralBots source code.  
They demonstrate how BASIC syntax integrates with Rust-based logic to perform automation, data management, and conversational tasks.
