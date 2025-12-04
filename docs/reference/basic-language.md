# BASIC Language Reference

Complete reference for General Bots BASIC dialog scripting language.

## Overview

General Bots BASIC is a domain-specific language for creating conversational AI dialogs. It provides keywords for:

- User interaction (TALK, HEAR)
- Knowledge base management (USE KB, CLEAR KB)
- Tool registration (USE TOOL, CLEAR TOOLS)
- Data operations (SAVE, GET, POST)
- File handling (SEND FILE, DOWNLOAD)
- Flow control (IF/THEN/ELSE, FOR/NEXT)

## Conversation Keywords

### TALK

Send a message to the user.

```basic
TALK "Hello, how can I help you?"
TALK "Your order number is: " + ordernumber
```

#### Multi-line Messages

```basic
BEGIN TALK
    **Welcome!**
    
    I can help you with:
    • Orders
    • Shipping
    • Returns
END TALK
```

### HEAR

Wait for and capture user input.

```basic
answer = HEAR
name = HEAR AS NAME
email = HEAR AS EMAIL
choice = HEAR AS "Option A", "Option B", "Option C"
confirmed = HEAR AS BOOLEAN
```

#### Input Types

| Type | Description | Example |
|------|-------------|---------|
| `STRING` | Free text | `answer = HEAR` |
| `NAME` | Person name | `name = HEAR AS NAME` |
| `EMAIL` | Email address | `email = HEAR AS EMAIL` |
| `PHONE` | Phone number | `phone = HEAR AS PHONE` |
| `INTEGER` | Whole number | `count = HEAR AS INTEGER` |
| `NUMBER` | Decimal number | `amount = HEAR AS NUMBER` |
| `BOOLEAN` | Yes/No | `confirm = HEAR AS BOOLEAN` |
| `DATE` | Date value | `date = HEAR AS DATE` |
| Options | Multiple choice | `choice = HEAR AS "A", "B", "C"` |

### WAIT

Pause execution for specified seconds.

```basic
WAIT 5
TALK "Processing..."
WAIT 2
TALK "Done!"
```

## Knowledge Base Keywords

### USE KB

Load a knowledge base into the current session.

```basic
USE KB "company-docs"
USE KB "product-catalog.gbkb"
```

### CLEAR KB

Remove knowledge base from session.

```basic
CLEAR KB "company-docs"
CLEAR KB  ' Clear all KBs
```

## Tool Keywords

### USE TOOL

Register a tool for the AI to call.

```basic
USE TOOL "create-ticket"
USE TOOL "send-email"
USE TOOL "search-orders"
```

### CLEAR TOOLS

Remove all registered tools.

```basic
CLEAR TOOLS
```

## Context Keywords

### SET CONTEXT

Define AI behavior context.

```basic
SET CONTEXT "assistant" AS "You are a helpful customer service agent for Acme Corp."
```

### System Prompt

Define detailed AI instructions.

```basic
BEGIN SYSTEM PROMPT
    You are a professional assistant.
    Always be polite and helpful.
    If you don't know something, say so.
    Never make up information.
END SYSTEM PROMPT
```

## Suggestion Keywords

### ADD SUGGESTION

Add a quick-reply button for users.

```basic
ADD SUGGESTION "help" AS "Show help"
ADD SUGGESTION "order" AS "Track my order"
ADD SUGGESTION "contact" AS "Contact support"
```

### CLEAR SUGGESTIONS

Remove all suggestions.

```basic
CLEAR SUGGESTIONS
```

## Data Keywords

### SAVE

Save data to storage.

```basic
SAVE "contacts.csv", name, email, phone
SAVE "orders.csv", orderid, product, quantity, total
```

### GET

HTTP GET request.

```basic
data = GET "https://api.example.com/users"
weather = GET "https://api.weather.com/current?city=" + city
```

### POST

HTTP POST request.

```basic
result = POST "https://api.example.com/orders", orderdata
```

### PUT

HTTP PUT request.

```basic
result = PUT "https://api.example.com/users/" + userid, userdata
```

### DELETE HTTP

HTTP DELETE request.

```basic
result = DELETE HTTP "https://api.example.com/users/" + userid
```

### SET HEADER

Set HTTP header for requests.

```basic
SET HEADER "Authorization" = "Bearer " + token
SET HEADER "Content-Type" = "application/json"
data = GET "https://api.example.com/protected"
```

### CLEAR HEADERS

Remove all custom headers.

```basic
CLEAR HEADERS
```

## File Keywords

### SEND FILE

Send a file to the user.

```basic
SEND FILE "report.pdf"
SEND FILE filepath
```

### DOWNLOAD

Download a file from URL.

```basic
file = DOWNLOAD "https://example.com/document.pdf"
SEND FILE file
```

### DELETE FILE

Delete a file from storage.

```basic
DELETE FILE "old-report.pdf"
```

## Email Keywords

### SEND MAIL

Send an email.

```basic
SEND MAIL "recipient@example.com", "Subject Line", "Email body text"
SEND MAIL email, subject, body
```

## Memory Keywords

### SET BOT MEMORY

Store a value in bot memory (persists across sessions).

```basic
SET BOT MEMORY "last_order", orderid
SET BOT MEMORY "user_preference", preference
```

### GET BOT MEMORY

Retrieve a value from bot memory.

```basic
lastorder = GET BOT MEMORY("last_order")
pref = GET BOT MEMORY("user_preference")
```

## Schedule Keywords

### SET SCHEDULE

Define when a job should run (cron format).

```basic
SET SCHEDULE "0 9 * * *"     ' Daily at 9 AM
SET SCHEDULE "0 0 * * 1"     ' Weekly on Monday
SET SCHEDULE "0 8 1 * *"     ' Monthly on the 1st at 8 AM
```

#### Cron Format

```
┌───────────── minute (0-59)
│ ┌───────────── hour (0-23)
│ │ ┌───────────── day of month (1-31)
│ │ │ ┌───────────── month (1-12)
│ │ │ │ ┌───────────── day of week (0-6, Sun=0)
│ │ │ │ │
* * * * *
```

## Flow Control

### IF/THEN/ELSE

Conditional execution.

```basic
IF status = "active" THEN
    TALK "Your account is active."
ELSE IF status = "pending" THEN
    TALK "Your account is pending approval."
ELSE
    TALK "Your account is inactive."
END IF
```

### FOR/NEXT

Loop through a range.

```basic
FOR i = 1 TO 10
    TALK "Item " + i
NEXT
```

### FOR EACH

Loop through a collection.

```basic
FOR EACH item IN items
    TALK item.name + ": $" + item.price
END FOR
```

## Variables

### Declaration

```basic
let name = "John"
let count = 42
let price = 19.99
let active = TRUE
```

### String Operations

```basic
let greeting = "Hello, " + name + "!"
let upper = UCASE(text)
let lower = LCASE(text)
let length = LEN(text)
let part = MID(text, 1, 5)
```

### Array Operations

```basic
let items = SPLIT(text, ",")
let first = items[0]
let count = LEN(items)
```

## Tool Definition

Tools are BASIC files that the AI can call.

### Structure

```basic
' tool-name.bas

PARAM parametername AS TYPE LIKE "example" DESCRIPTION "What this parameter is"
PARAM optionalparam AS STRING DESCRIPTION "Optional parameter"

DESCRIPTION "What this tool does. Called when user wants to [action]."

' Implementation
IF parametername = "" THEN
    TALK "Please provide the parameter."
    parametername = HEAR
END IF

let result = "processed: " + parametername

SAVE "records.csv", parametername, result

TALK "✅ Done: " + result

RETURN result
```

### Parameter Types

| Type | Description |
|------|-------------|
| `STRING` | Text value |
| `INTEGER` | Whole number |
| `NUMBER` | Decimal number |
| `BOOLEAN` | True/False |
| `DATE` | Date value |
| `EMAIL` | Email address |
| `PHONE` | Phone number |

## Comments

```basic
' This is a single-line comment

REM This is also a comment

' Multi-line comments use multiple single-line comments
' Line 1
' Line 2
```

## Built-in Functions

### String Functions

| Function | Description | Example |
|----------|-------------|---------|
| `LEN(s)` | String length | `LEN("hello")` → `5` |
| `UCASE(s)` | Uppercase | `UCASE("hello")` → `"HELLO"` |
| `LCASE(s)` | Lowercase | `LCASE("HELLO")` → `"hello"` |
| `TRIM(s)` | Remove whitespace | `TRIM("  hi  ")` → `"hi"` |
| `MID(s,start,len)` | Substring | `MID("hello",2,3)` → `"ell"` |
| `LEFT(s,n)` | Left characters | `LEFT("hello",2)` → `"he"` |
| `RIGHT(s,n)` | Right characters | `RIGHT("hello",2)` → `"lo"` |
| `SPLIT(s,delim)` | Split to array | `SPLIT("a,b,c",",")` → `["a","b","c"]` |
| `REPLACE(s,old,new)` | Replace text | `REPLACE("hello","l","x")` → `"hexxo"` |

### Date Functions

| Function | Description | Example |
|----------|-------------|---------|
| `NOW()` | Current datetime | `NOW()` |
| `TODAY()` | Current date | `TODAY()` |
| `YEAR(d)` | Extract year | `YEAR(date)` → `2024` |
| `MONTH(d)` | Extract month | `MONTH(date)` → `12` |
| `DAY(d)` | Extract day | `DAY(date)` → `15` |
| `DATEADD(d,n,unit)` | Add to date | `DATEADD(date,7,"days")` |
| `DATEDIFF(d1,d2,unit)` | Date difference | `DATEDIFF(date1,date2,"days")` |

### Math Functions

| Function | Description | Example |
|----------|-------------|---------|
| `ABS(n)` | Absolute value | `ABS(-5)` → `5` |
| `ROUND(n,d)` | Round number | `ROUND(3.456,2)` → `3.46` |
| `FLOOR(n)` | Round down | `FLOOR(3.7)` → `3` |
| `CEILING(n)` | Round up | `CEILING(3.2)` → `4` |
| `MIN(a,b)` | Minimum | `MIN(5,3)` → `3` |
| `MAX(a,b)` | Maximum | `MAX(5,3)` → `5` |
| `SUM(arr)` | Sum of array | `SUM(numbers)` |
| `AVG(arr)` | Average | `AVG(numbers)` |

### Conversion Functions

| Function | Description | Example |
|----------|-------------|---------|
| `STR(n)` | Number to string | `STR(42)` → `"42"` |
| `VAL(s)` | String to number | `VAL("42")` → `42` |
| `INT(n)` | To integer | `INT(3.7)` → `3` |

## Complete Example

```basic
' customer-support.bas - Main support dialog

' Setup
USE KB "support-docs"
USE TOOL "create-ticket"
USE TOOL "check-order"
USE TOOL "request-refund"

SET CONTEXT "support" AS "You are a helpful customer support agent for Acme Store."

' Welcome
BEGIN TALK
    **Welcome to Acme Support!**
    
    I can help you with:
    • Order tracking
    • Returns and refunds
    • Product questions
END TALK

' Quick actions
CLEAR SUGGESTIONS
ADD SUGGESTION "order" AS "Track my order"
ADD SUGGESTION "return" AS "Request a return"
ADD SUGGESTION "help" AS "Other questions"

BEGIN SYSTEM PROMPT
    Be friendly and professional.
    Always verify order numbers before making changes.
    For refunds over $100, escalate to human support.
    If asked about competitors, politely redirect to our products.
END SYSTEM PROMPT
```

## Keyword Quick Reference

| Category | Keywords |
|----------|----------|
| Conversation | `TALK`, `HEAR`, `WAIT` |
| Knowledge | `USE KB`, `CLEAR KB` |
| Tools | `USE TOOL`, `CLEAR TOOLS` |
| Context | `SET CONTEXT`, `SYSTEM PROMPT` |
| Suggestions | `ADD SUGGESTION`, `CLEAR SUGGESTIONS` |
| Data | `SAVE`, `GET`, `POST`, `PUT`, `DELETE HTTP` |
| HTTP | `SET HEADER`, `CLEAR HEADERS` |
| Files | `SEND FILE`, `DOWNLOAD`, `DELETE FILE` |
| Email | `SEND MAIL` |
| Memory | `SET BOT MEMORY`, `GET BOT MEMORY` |
| Schedule | `SET SCHEDULE` |
| Flow | `IF/THEN/ELSE/END IF`, `FOR/NEXT`, `FOR EACH` |
| Tools | `PARAM`, `DESCRIPTION`, `RETURN` |