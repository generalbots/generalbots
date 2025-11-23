# Keyword Reference

This section lists every BASIC keyword implemented in the GeneralBots engine. Each keyword page includes:

* **Syntax** – Exact command format
* **Parameters** – Expected arguments
* **Description** – What the keyword does
* **Example** – A short snippet showing usage

The source code for each keyword lives in `src/basic/keywords/`. Only the keywords listed here exist in the system.

## Core Dialog Keywords

- [TALK](./keyword-talk.md) - Send message to user
- [HEAR](./keyword-hear.md) - Get input from user
- [WAIT](./keyword-wait.md) - Pause execution
- [PRINT](./keyword-print.md) - Debug output

## Variable & Memory

- [SET](./keyword-set.md) - Set variable value
- [GET](./keyword-get.md) - Get variable value
- [SET BOT MEMORY](./keyword-set-bot-memory.md) - Persist data
- [GET BOT MEMORY](./keyword-get-bot-memory.md) - Retrieve persisted data

## AI & Context

- [LLM](./keyword-llm.md) - Query language model
- [SET CONTEXT](./keyword-set-context.md) - Add context for LLM
- [SET USER](./keyword-set-user.md) - Set user context

## Knowledge Base

- [USE KB](./keyword-use-kb.md) - Load knowledge base
- [CLEAR KB](./keyword-clear-kb.md) - Unload knowledge base
- [ADD WEBSITE](./keyword-add-website.md) - Index website to KB
- [FIND](./keyword-find.md) - Search in KB

## Tools & Automation

- [USE TOOL](./keyword-use-tool.md) - Load tool definition
- [CLEAR TOOLS](./keyword-clear-tools.md) - Remove all tools
- [CREATE TASK](./keyword-create-task.md) - Create task
- [CREATE SITE](./keyword-create-site.md) - Generate website
- [CREATE DRAFT](./keyword-create-draft.md) - Create email draft

## UI & Interaction

- [ADD SUGGESTION](./keyword-add-suggestion.md) - Add clickable button
- [CLEAR SUGGESTIONS](./keyword-clear-suggestions.md) - Remove buttons
- [CHANGE THEME](./keyword-change-theme.md) - Switch UI theme

## Data Processing

- [FORMAT](./keyword-format.md) - Format strings
- [FIRST](./keyword-first.md) - Get first element
- [LAST](./keyword-last.md) - Get last element
- [SAVE FROM UNSTRUCTURED](./keyword-save-from-unstructured.md) - Extract structured data

## Flow Control

- [FOR EACH ... NEXT](./keyword-for-each.md) - Loop through items
- [EXIT FOR](./keyword-exit-for.md) - Exit loop early
- [ON](./keyword-on.md) - Event handler
- [SET SCHEDULE](./keyword-set-schedule.md) - Schedule execution

## Communication

- [SEND MAIL](./keyword-send-mail.md) - Send email
- [ADD MEMBER](./keyword-add-member.md) - Add group member

## Special Functions

- [BOOK](./keyword-book.md) - Book appointment
- [REMEMBER](./keyword-remember.md) - Store in memory
- [WEATHER](./keyword-weather.md) - Get weather info

## Notes

- Keywords are case-insensitive (TALK = talk = Talk)
- String parameters can use double quotes or single quotes
- Comments start with REM or '
- Line continuation uses underscore (_)