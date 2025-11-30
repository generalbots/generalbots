# Complete Keyword Reference

This document provides a comprehensive reference of all BASIC keywords in General Bots, including existing implementations and planned additions.

---

## Quick Reference Table

| Keyword | Status | Category | Description |
|---------|--------|----------|-------------|
| `TALK` | ✅ Implemented | Dialog | Send message to user |
| `HEAR` | ✅ Implemented | Dialog | Get input from user |
| `WAIT` | ✅ Implemented | Dialog | Pause execution |
| `PRINT` | ✅ Implemented | Debug | Debug output |
| `SET` | ✅ Implemented | Variables | Set variable value |
| `GET` | ✅ Implemented | Variables | Get variable or fetch data |
| `LET` | ✅ Implemented | Variables | Assign variable (classic BASIC) |
| `SET BOT MEMORY` | ✅ Implemented | Memory | Persist bot-level data |
| `GET BOT MEMORY` | ✅ Implemented | Memory | Retrieve bot-level data |
| `REMEMBER` | ✅ Implemented | Memory | Store user-specific memory |
| `LLM` | ✅ Implemented | AI | Query language model |
| `SET CONTEXT` | ✅ Implemented | AI | Add context for LLM |
| `SET USER` | ✅ Implemented | Session | Set user context |
| `USE KB` | ✅ Implemented | Knowledge | Load knowledge base |
| `CLEAR KB` | ✅ Implemented | Knowledge | Unload knowledge base |
| `USE WEBSITE` | ✅ Implemented | Knowledge | Associate website |
| `ADD TOOL` | ✅ Implemented | Tools | Register tool |
| `USE TOOL` | ✅ Implemented | Tools | Load tool definition |
| `CLEAR TOOLS` | ✅ Implemented | Tools | Remove all tools |
| `ADD SUGGESTION` | ✅ Implemented | UI | Add clickable button |
| `CLEAR SUGGESTIONS` | ✅ Implemented | UI | Remove buttons |
| `SAVE` | ✅ Implemented | Data | Save to table |
| `FIND` | ✅ Implemented | Data | Search in files |
| `INSERT` | ✅ Implemented | Data | Insert record |
| `UPDATE` | ✅ Implemented | Data | Update records |
| `DELETE` | ✅ Implemented | Data | Delete records |
| `FILTER` | ✅ Implemented | Data | Filter records |
| `AGGREGATE` | ✅ Implemented | Data | SUM, AVG, COUNT, etc. |
| `JOIN` | ✅ Implemented | Data | Join datasets |
| `MERGE` | ✅ Implemented | Data | Merge data |
| `MAP` | ✅ Implemented | Data | Map field names |
| `FILL` | ✅ Implemented | Data | Fill template |
| `PIVOT` | ✅ Implemented | Data | Create pivot table |
| `GROUP BY` | ✅ Implemented | Data | Group data |
| `FIRST` | ✅ Implemented | Data | Get first element |
| `LAST` | ✅ Implemented | Data | Get last element |
| `FORMAT` | ✅ Implemented | Data | Format values |
| `SAVE FROM UNSTRUCTURED` | ✅ Implemented | Data | Extract structured data |
| `POST` | ✅ Implemented | HTTP | HTTP POST |
| `PUT` | ✅ Implemented | HTTP | HTTP PUT |
| `PATCH` | ✅ Implemented | HTTP | HTTP PATCH |
| `DELETE HTTP` | ✅ Implemented | HTTP | HTTP DELETE |
| `SET HEADER` | ✅ Implemented | HTTP | Set HTTP header |
| `CLEAR HEADERS` | ✅ Implemented | HTTP | Clear headers |
| `GRAPHQL` | ✅ Implemented | HTTP | GraphQL query |
| `SOAP` | ✅ Implemented | HTTP | SOAP call |
| `READ` | ✅ Implemented | Files | Read file |
| `WRITE` | ✅ Implemented | Files | Write file |
| `DELETE FILE` | ✅ Implemented | Files | Delete file |
| `COPY` | ✅ Implemented | Files | Copy file |
| `MOVE` | ✅ Implemented | Files | Move/rename file |
| `LIST` | ✅ Implemented | Files | List directory |
| `UPLOAD` | ✅ Implemented | Files | Upload file |
| `DOWNLOAD` | ✅ Implemented | Files | Download file |
| `COMPRESS` | ✅ Implemented | Files | Create ZIP |
| `EXTRACT` | ✅ Implemented | Files | Extract ZIP |
| `GENERATE PDF` | ✅ Implemented | Files | Generate PDF |
| `MERGE PDF` | ✅ Implemented | Files | Merge PDFs |
| `IF...THEN...ELSE...END IF` | ✅ Implemented | Control | Conditional |
| `FOR EACH...NEXT` | ✅ Implemented | Control | Loop |
| `EXIT FOR` | ✅ Implemented | Control | Exit loop |
| `WHILE...WEND` | ✅ Implemented | Control | While loop |
| `DO...LOOP` | ✅ Implemented | Control | Do loop |
| `SWITCH...CASE...END SWITCH` | ✅ Implemented | Control | Switch |
| `SUB...END SUB` | ✅ Implemented | Procedures | Define subroutine |
| `FUNCTION...END FUNCTION` | ✅ Implemented | Procedures | Define function |
| `CALL` | ✅ Implemented | Procedures | Call procedure |
| `RETURN` | ✅ Implemented | Procedures | Return value |
| `ON` | ✅ Implemented | Events | Event handler |
| `SET SCHEDULE` | ✅ Implemented | Events | Schedule execution |
| `WEBHOOK` | ✅ Implemented | Events | Create webhook |
| `SEND MAIL` | ✅ Implemented | Communication | Send email |
| `CREATE DRAFT` | ✅ Implemented | Communication | Create email draft |
| `ADD MEMBER` | ✅ Implemented | Communication | Add to group |
| `CREATE TASK` | ✅ Implemented | Tools | Create task |
| `CREATE SITE` | ✅ Implemented | Tools | Generate website |
| `BOOK` | ✅ Implemented | Special | Book appointment |
| `WEATHER` | ✅ Implemented | Special | Get weather |
| `INSTR` | ✅ Implemented | String | Find substring |
| `IS NUMERIC` | ✅ Implemented | Validation | Check if numeric |

---

## Planned Keywords (Priority)

### String Functions (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `LEN` | `length = LEN(string)` | Get string length | ⭐⭐⭐ |
| `LEFT` | `result = LEFT(string, n)` | Get left n characters | ⭐⭐⭐ |
| `RIGHT` | `result = RIGHT(string, n)` | Get right n characters | ⭐⭐⭐ |
| `MID` | `result = MID(string, start, length)` | Get substring | ⭐⭐⭐ |
| `TRIM` | `result = TRIM(string)` | Remove whitespace | ⭐⭐⭐ |
| `LTRIM` | `result = LTRIM(string)` | Remove left whitespace | ⭐⭐ |
| `RTRIM` | `result = RTRIM(string)` | Remove right whitespace | ⭐⭐ |
| `UCASE` | `result = UCASE(string)` | Convert to uppercase | ⭐⭐⭐ |
| `LCASE` | `result = LCASE(string)` | Convert to lowercase | ⭐⭐⭐ |
| `REPLACE` | `result = REPLACE(string, old, new)` | Replace substring | ⭐⭐⭐ |
| `SPLIT` | `array = SPLIT(string, delimiter)` | Split string to array | ⭐⭐⭐ |
| `ASC` | `code = ASC(char)` | Get ASCII code | ⭐⭐ |
| `CHR` | `char = CHR(code)` | Get character from code | ⭐⭐ |

### Math Functions (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `ABS` | `result = ABS(number)` | Absolute value | ⭐⭐⭐ |
| `ROUND` | `result = ROUND(number, decimals)` | Round number | ⭐⭐⭐ |
| `INT` | `result = INT(number)` | Integer part | ⭐⭐⭐ |
| `SQRT` | `result = SQRT(number)` | Square root | ⭐⭐ |
| `MAX` | `result = MAX(a, b)` or `MAX(array)` | Maximum value | ⭐⭐⭐ |
| `MIN` | `result = MIN(a, b)` or `MIN(array)` | Minimum value | ⭐⭐⭐ |
| `MOD` | `result = a MOD b` | Modulo/remainder | ⭐⭐⭐ |
| `RND` | `result = RND()` | Random 0-1 | ⭐⭐ |
| `RANDOM` | `result = RANDOM(min, max)` | Random in range | ⭐⭐⭐ |
| `CEILING` | `result = CEILING(number)` | Round up | ⭐⭐ |
| `FLOOR` | `result = FLOOR(number)` | Round down | ⭐⭐ |
| `SGN` | `result = SGN(number)` | Sign (-1, 0, 1) | ⭐ |

### Date/Time Functions (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `NOW` | `datetime = NOW()` | Current date/time | ⭐⭐⭐ |
| `TODAY` | `date = TODAY()` | Current date | ⭐⭐⭐ |
| `YEAR` | `year = YEAR(date)` | Extract year | ⭐⭐⭐ |
| `MONTH` | `month = MONTH(date)` | Extract month | ⭐⭐⭐ |
| `DAY` | `day = DAY(date)` | Extract day | ⭐⭐⭐ |
| `HOUR` | `hour = HOUR(datetime)` | Extract hour | ⭐⭐ |
| `MINUTE` | `minute = MINUTE(datetime)` | Extract minute | ⭐⭐ |
| `SECOND` | `second = SECOND(datetime)` | Extract second | ⭐⭐ |
| `WEEKDAY` | `dow = WEEKDAY(date)` | Day of week (1-7) | ⭐⭐ |
| `DATEADD` | `date = DATEADD(date, n, "day")` | Add to date | ⭐⭐⭐ |
| `DATEDIFF` | `days = DATEDIFF(date1, date2, "day")` | Date difference | ⭐⭐⭐ |
| `DATEVALUE` | `date = DATEVALUE("2025-01-15")` | Parse date string | ⭐⭐ |
| `TIMEVALUE` | `time = TIMEVALUE("14:30:00")` | Parse time string | ⭐⭐ |
| `EOMONTH` | `date = EOMONTH(date, 0)` | End of month | ⭐⭐ |

### Type Conversion (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `VAL` | `number = VAL(string)` | String to number | ⭐⭐⭐ |
| `STR` | `string = STR(number)` | Number to string | ⭐⭐⭐ |
| `CINT` | `integer = CINT(value)` | Convert to integer | ⭐⭐ |
| `CDBL` | `double = CDBL(value)` | Convert to double | ⭐⭐ |
| `CSTR` | `string = CSTR(value)` | Convert to string | ⭐⭐ |
| `CBOOL` | `boolean = CBOOL(value)` | Convert to boolean | ⭐⭐ |
| `CDATE` | `date = CDATE(value)` | Convert to date | ⭐⭐ |

### Validation Functions (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `ISNULL` | `result = ISNULL(value)` | Check if null | ⭐⭐⭐ |
| `ISEMPTY` | `result = ISEMPTY(value)` | Check if empty | ⭐⭐⭐ |
| `ISDATE` | `result = ISDATE(value)` | Check if valid date | ⭐⭐ |
| `ISARRAY` | `result = ISARRAY(value)` | Check if array | ⭐⭐ |

### Array Functions (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `ARRAY` | `arr = ARRAY(1, 2, 3)` | Create array | ⭐⭐⭐ |
| `UBOUND` | `size = UBOUND(array)` | Get array size | ⭐⭐⭐ |
| `LBOUND` | `start = LBOUND(array)` | Get array start | ⭐⭐ |
| `REDIM` | `REDIM array(10)` | Resize array | ⭐⭐ |
| `SORT` | `sorted = SORT(array)` | Sort array | ⭐⭐⭐ |
| `UNIQUE` | `distinct = UNIQUE(array)` | Remove duplicates | ⭐⭐⭐ |
| `CONTAINS` | `result = CONTAINS(array, value)` | Check membership | ⭐⭐⭐ |
| `PUSH` | `PUSH array, value` | Add to end | ⭐⭐ |
| `POP` | `value = POP(array)` | Remove from end | ⭐⭐ |
| `REVERSE` | `reversed = REVERSE(array)` | Reverse array | ⭐ |

### Error Handling (Critical)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `ON ERROR GOTO` | `ON ERROR GOTO handler` | Set error handler | ⭐⭐⭐ |
| `ON ERROR RESUME NEXT` | `ON ERROR RESUME NEXT` | Ignore errors | ⭐⭐ |
| `TRY...CATCH...END TRY` | `TRY ... CATCH e ... END TRY` | Structured errors | ⭐⭐⭐ |
| `THROW` | `THROW "error message"` | Raise error | ⭐⭐ |
| `ERR` | `code = ERR.NUMBER` | Get error code | ⭐⭐ |

### Social Media (New)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `POST TO` | `POST TO "instagram" image, caption` | Post to social | ⭐⭐⭐ |
| `GET METRICS` | `metrics = GET INSTAGRAM METRICS "id"` | Get engagement | ⭐⭐ |
| `SCHEDULE POST` | `SCHEDULE POST TO "instagram" AT datetime` | Schedule post | ⭐⭐ |

### Marketing (New)

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `CREATE LANDING PAGE` | `CREATE LANDING PAGE "name"` | Build page | ⭐⭐⭐ |
| `ADD TO CAMPAIGN` | `ADD TO CAMPAIGN email, "name"` | Add to sequence | ⭐⭐⭐ |
| `TRACK CONVERSION` | `TRACK CONVERSION "campaign", "event"` | Track event | ⭐⭐ |
| `GET CAMPAIGN STATS` | `stats = GET CAMPAIGN STATS "name"` | Get analytics | ⭐⭐ |

### Logic Operators

| Keyword | Syntax | Description | Priority |
|---------|--------|-------------|----------|
| `AND` | `IF a AND b THEN` | Logical AND | ✅ Implemented |
| `OR` | `IF a OR b THEN` | Logical OR | ✅ Implemented |
| `NOT` | `IF NOT a THEN` | Logical NOT | ✅ Implemented |
| `XOR` | `result = a XOR b` | Exclusive OR | ⭐ |
| `IIF` | `result = IIF(condition, true, false)` | Inline IF | ⭐⭐ |
| `LIKE` | `IF name LIKE "John*" THEN` | Pattern match | ⭐⭐ |
| `IN` | `IF value IN (1, 2, 3) THEN` | Set membership | ⭐⭐ |
| `BETWEEN` | `IF value BETWEEN 1 AND 10 THEN` | Range check | ⭐⭐ |

---

## Keyword Categories

### 1. Dialog & Interaction

```basic
' Basic conversation
TALK "Hello! How can I help you?"
HEAR response

' Typed input
HEAR name AS STRING
HEAR age AS INTEGER
HEAR email AS EMAIL
HEAR date AS DATE
HEAR amount AS MONEY
HEAR phone AS MOBILE
HEAR confirm AS BOOLEAN

' Menu selection
HEAR choice AS "Option 1", "Option 2", "Option 3"

' File upload
HEAR document AS FILE
HEAR audio AS AUDIO
HEAR image AS IMAGE
HEAR qrcode AS QRCODE
```

### 2. Memory & Variables

```basic
' Local variables
LET name = "John"
SET age = 25

' Bot-level memory (shared across sessions)
SET BOT MEMORY "config_key", "value"
value = GET BOT MEMORY("config_key")

' User-specific memory
REMEMBER "preference", "dark_mode"
```

### 3. AI & Knowledge

```basic
' Query LLM
response = LLM "Summarize this text: " + text

' Set context for LLM
SET CONTEXT "assistant" AS "You are a helpful sales assistant."

' Load knowledge base
USE KB "product-docs"
USE KB "faq"
CLEAR KB "product-docs"
CLEAR KB

' Load website content
USE WEBSITE "https://example.com"
```

### 4. Tools & Automation

```basic
' Register tools
ADD TOOL "create-ticket"
ADD TOOL "send-email"
USE TOOL "search-database"
CLEAR TOOLS

' Create tasks
CREATE TASK "Review document", "Please review the attached file"

' Generate websites
CREATE SITE "promo" WITH TEMPLATE "landing" USING PROMPT "..."

' Create email drafts
CREATE DRAFT "user@example.com", "Subject", "Body"
```

### 5. Data Operations

```basic
' Save data (variable names = field names)
SAVE "customers.csv", name, email, phone, created

' Find data
customers = FIND "customers.csv"
active = FIND "customers.csv", "status=active"

' Filter data
highValue = FILTER customers, "amount>1000"
recent = FILTER customers, "created>=2025-01-01"

' Aggregate data
total = AGGREGATE "SUM", orders, "amount"
average = AGGREGATE "AVG", orders, "amount"
count = AGGREGATE "COUNT", orders, "id"

' Join datasets
joined = JOIN orders, customers, "customer_id"

' Group and pivot
byCategory = GROUP BY sales, "category"
pivoted = PIVOT sales, "month", "amount"

' Transform data
mapped = MAP data, "firstName->name, lastName->surname"
filled = FILL template, data

' CRUD operations
INSERT "table", data
UPDATE "table", "filter", data
DELETE "table", "filter"
MERGE "table", data, "key_field"

' Extract from text
SAVE FROM UNSTRUCTURED "contacts", unstructuredText
```

### 6. HTTP & API

```basic
' GET request
data = GET "https://api.example.com/users"

' POST request
response = POST "https://api.example.com/users", userData

' PUT request
response = PUT "https://api.example.com/users/123", userData

' PATCH request
response = PATCH "https://api.example.com/users/123", partialData

' DELETE request
response = DELETE HTTP "https://api.example.com/users/123"

' Set headers
SET HEADER "Authorization", "Bearer " + token
SET HEADER "Content-Type", "application/json"
CLEAR HEADERS

' GraphQL
result = GRAPHQL "https://api.example.com/graphql", query, variables

' SOAP
result = SOAP "https://api.example.com/service.wsdl", "Operation", params
```

### 7. File Operations

```basic
' Read/Write
content = READ "config.json"
WRITE "output.txt", content

' Copy/Move
COPY "source.txt", "backup/source.txt"
MOVE "temp.txt", "final/document.txt"

' Delete
DELETE FILE "temp/old-file.txt"

' List directory
files = LIST "documents/"

' Upload/Download
url = UPLOAD localFile, "uploads/document.pdf"
path = DOWNLOAD "https://example.com/file.pdf", "downloads/file.pdf"

' Archive
archive = COMPRESS files, "backup.zip"
extracted = EXTRACT "archive.zip", "extracted/"

' PDF operations
pdf = GENERATE PDF "template.html", data, "output.pdf"
merged = MERGE PDF pdfFiles, "combined.pdf"
```

### 8. Flow Control

```basic
' Conditional
IF age >= 18 THEN
    TALK "You are an adult."
ELSE
    TALK "You are a minor."
END IF

' Switch/Case
SWITCH category
    CASE "A"
        TALK "Category A"
    CASE "B"
        TALK "Category B"
    CASE ELSE
        TALK "Other category"
END SWITCH

' Loops
FOR EACH item IN items
    TALK item.name
NEXT item

WHILE count < 10
    count = count + 1
WEND

DO
    result = processItem()
LOOP UNTIL result = "done"

' Exit loop
FOR EACH item IN items
    IF item.skip THEN EXIT FOR
    TALK item.name
NEXT item
```

### 9. Procedures

```basic
' Define subroutine
SUB ProcessOrder(orderId)
    order = FIND "orders.csv", "id=" + orderId
    TALK "Processing order: " + order.id
END SUB

' Define function
FUNCTION CalculateTotal(items)
    total = 0
    FOR EACH item IN items
        total = total + item.price * item.quantity
    NEXT item
    RETURN total
END FUNCTION

' Call procedures
CALL ProcessOrder("ORD-001")
total = CalculateTotal(cartItems)
```

### 10. Events & Scheduling

```basic
' Event handlers
ON "new_message" CALL HandleMessage

' Scheduled jobs
SET SCHEDULE "0 8 * * *"  ' Every day at 8 AM

' Webhooks
WEBHOOK "order-received"
' Endpoint: /api/{bot}/webhook/order-received
```

### 11. Communication

```basic
' Send email
SEND MAIL "user@example.com", "Subject", "Body"

' Send to specific number
SEND MAIL TO "5521999999999", "Subject", "Body"

' Add to group
ADD MEMBER "user@example.com", "sales-team"
```

### 12. UI & Suggestions

```basic
' Add quick reply buttons
ADD SUGGESTION "help" AS "Get Help"
ADD SUGGESTION "menu" AS "Show Menu"
ADD SUGGESTION "cancel" AS "Cancel"

' Clear suggestions
CLEAR SUGGESTIONS
```

---

## Input Types for HEAR

| Type | Example | Description |
|------|---------|-------------|
| `STRING` | `HEAR name AS STRING` | Any text input |
| `INTEGER` | `HEAR age AS INTEGER` | Whole number |
| `NUMBER` | `HEAR price AS NUMBER` | Decimal number |
| `BOOLEAN` | `HEAR confirm AS BOOLEAN` | Yes/No, True/False |
| `DATE` | `HEAR birthday AS DATE` | Date value |
| `EMAIL` | `HEAR contact AS EMAIL` | Valid email address |
| `MOBILE` | `HEAR phone AS MOBILE` | Phone number |
| `MONEY` | `HEAR amount AS MONEY` | Currency amount |
| `HOUR` | `HEAR time AS HOUR` | Time value |
| `ZIPCODE` | `HEAR zip AS ZIPCODE` | Postal code |
| `LANGUAGE` | `HEAR lang AS LANGUAGE` | Language code |
| `FILE` | `HEAR doc AS FILE` | File upload |
| `AUDIO` | `HEAR recording AS AUDIO` | Audio upload |
| `IMAGE` | `HEAR photo AS IMAGE` | Image upload |
| `QRCODE` | `HEAR code AS QRCODE` | QR code from image |
| `LOGIN` | `HEAR user AS LOGIN` | Authenticated user |
| Menu | `HEAR choice AS "A", "B", "C"` | Selection from options |

---

## FORMAT Patterns

| Pattern | Example | Result |
|---------|---------|--------|
| `YYYY` | `FORMAT date AS "YYYY"` | 2025 |
| `MM` | `FORMAT date AS "MM"` | 01 |
| `DD` | `FORMAT date AS "DD"` | 15 |
| `YYYY-MM-DD` | `FORMAT date AS "YYYY-MM-DD"` | 2025-01-15 |
| `HH:mm:ss` | `FORMAT time AS "HH:mm:ss"` | 14:30:00 |
| `#,##0` | `FORMAT number AS "#,##0"` | 1,234 |
| `#,##0.00` | `FORMAT number AS "#,##0.00"` | 1,234.56 |
| `0%` | `FORMAT decimal AS "0%"` | 75% |

---

## AGGREGATE Operations

| Operation | Example | Description |
|-----------|---------|-------------|
| `SUM` | `AGGREGATE "SUM", data, "amount"` | Sum of values |
| `AVG` | `AGGREGATE "AVG", data, "amount"` | Average |
| `COUNT` | `AGGREGATE "COUNT", data, "id"` | Count records |
| `MIN` | `AGGREGATE "MIN", data, "price"` | Minimum value |
| `MAX` | `AGGREGATE "MAX", data, "price"` | Maximum value |

---

## Schedule Patterns (Cron)

| Pattern | Description |
|---------|-------------|
| `0 8 * * *` | Every day at 8:00 AM |
| `0 9 * * 1` | Every Monday at 9:00 AM |
| `0 0 1 * *` | First day of every month |
| `*/15 * * * *` | Every 15 minutes |
| `0 8,17 * * *` | At 8:00 AM and 5:00 PM |
| `0 8 * * 1-5` | Weekdays at 8:00 AM |

---

## Best Practices

### DO ✅

```basic
' Use descriptive variable names
let customername = "John Smith"
let orderamount = 150.00

' Use SAVE with variable names as field names
SAVE "orders.csv", customername, orderamount, orderdate

' Use spaces in keywords
SET BOT MEMORY "last_order", orderid
SET CONTEXT "assistant" AS "You are helpful."
ADD SUGGESTION "help" AS "Get Help"
CLEAR SUGGESTIONS
USE KB "products"
USE TOOL "search"

' Use HEAR AS for type validation
HEAR email AS EMAIL
HEAR amount AS MONEY
HEAR confirm AS BOOLEAN
```

### DON'T ❌

```basic
' Don't use underscores in keywords (old syntax)
SET_BOT_MEMORY "key", value  ' WRONG - use spaces

' Don't use complex object operations
SET object.field = value  ' WRONG
SAVE "table", object.id, object  ' WRONG

' Don't skip input validation
HEAR value  ' OK but less safe
' Better: HEAR value AS STRING
```

---

## Migration from Old Syntax

| Old Syntax | New Syntax |
|------------|------------|
| `SET_BOT_MEMORY` | `SET BOT MEMORY` |
| `GET_BOT_MEMORY()` | `GET BOT MEMORY()` |
| `SET_CONTEXT` | `SET CONTEXT` |
| `ADD_SUGGESTION` | `ADD SUGGESTION` |
| `CLEAR_SUGGESTIONS` | `CLEAR SUGGESTIONS` |
| `USE_KB` | `USE KB` |
| `USE_TOOL` | `USE TOOL` |
| `CLEAR_KB` | `CLEAR KB` |
| `CLEAR_TOOLS` | `CLEAR TOOLS` |
| `SET_SCHEDULE` | `SET SCHEDULE` |
| `SET_HEADER` | `SET HEADER` |
| `CLEAR_HEADERS` | `CLEAR HEADERS` |
| `DELETE_HTTP` | `DELETE HTTP` |
| `DELETE_FILE` | `DELETE FILE` |
| `GENERATE_PDF` | `GENERATE PDF` |
| `MERGE_PDF` | `MERGE PDF` |
| `GROUP_BY` | `GROUP BY` |
| `SAVE_FROM_UNSTRUCTURED` | `SAVE FROM UNSTRUCTURED` |

---

*Last updated: January 2025*