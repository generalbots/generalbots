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
- [USE WEBSITE](./keyword-use-website.md) - Associate website with conversation
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
- [WEBHOOK](./keyword-webhook.md) - Create webhook endpoint

## Communication

- [SEND MAIL](./keyword-send-mail.md) - Send email
- [ADD MEMBER](./keyword-add-member.md) - Add group member

## Special Functions

- [BOOK](./keyword-book.md) - Book appointment
- [REMEMBER](./keyword-remember.md) - Store in memory
- [WEATHER](./keyword-weather.md) - Get weather info

---

## HTTP & API Operations

These keywords enable integration with external REST APIs, GraphQL endpoints, and SOAP services.

### POST

Sends an HTTP POST request with JSON body.

**Syntax:**
```basic
result = POST "url", data
```

**Parameters:**
- `url` - The endpoint URL (string)
- `data` - The request body (object or map)

**Example:**
```basic
payload = #{ "name": "John", "email": "john@example.com" }
response = POST "https://api.example.com/users", payload
TALK "Created user with ID: " + response.data.id
```

### PUT

Sends an HTTP PUT request to update a resource.

**Syntax:**
```basic
result = PUT "url", data
```

**Example:**
```basic
updates = #{ "name": "John Updated" }
response = PUT "https://api.example.com/users/123", updates
```

### PATCH

Sends an HTTP PATCH request for partial updates.

**Syntax:**
```basic
result = PATCH "url", data
```

**Example:**
```basic
partial = #{ "status": "active" }
response = PATCH "https://api.example.com/users/123", partial
```

### DELETE_HTTP

Sends an HTTP DELETE request.

**Syntax:**
```basic
result = DELETE_HTTP "url"
```

**Example:**
```basic
response = DELETE_HTTP "https://api.example.com/users/123"
```

### SET_HEADER

Sets an HTTP header for subsequent requests.

**Syntax:**
```basic
SET_HEADER "header-name", "value"
```

**Example:**
```basic
SET_HEADER "Authorization", "Bearer " + token
SET_HEADER "X-Custom-Header", "custom-value"
response = GET "https://api.example.com/protected"
```

### CLEAR_HEADERS

Clears all previously set HTTP headers.

**Syntax:**
```basic
CLEAR_HEADERS
```

### GRAPHQL

Executes a GraphQL query.

**Syntax:**
```basic
result = GRAPHQL "endpoint", "query", variables
```

**Example:**
```basic
query = "query GetUser($id: ID!) { user(id: $id) { name email } }"
vars = #{ "id": "123" }
response = GRAPHQL "https://api.example.com/graphql", query, vars
TALK "User name: " + response.data.user.name
```

### SOAP

Executes a SOAP API call.

**Syntax:**
```basic
result = SOAP "wsdl_url", "operation", params
```

**Example:**
```basic
params = #{ "customerId": "12345", "amount": 100.00 }
response = SOAP "https://api.example.com/service.wsdl", "CreateOrder", params
```

---

## Database & Data Operations

These keywords provide comprehensive data manipulation capabilities.

### SAVE

Saves data to a table (upsert - inserts if new, updates if exists).

**Syntax:**
```basic
SAVE "table", id, data
```

**Parameters:**
- `table` - Table name (string)
- `id` - Record identifier
- `data` - Data object to save

**Example:**
```basic
customer = #{ "name": "John", "email": "john@example.com", "status": "active" }
SAVE "customers", "cust-001", customer
```

### INSERT

Inserts a new record into a table.

**Syntax:**
```basic
result = INSERT "table", data
```

**Example:**
```basic
order = #{ "product": "Widget", "quantity": 5, "price": 29.99 }
result = INSERT "orders", order
TALK "Created order: " + result.id
```

### UPDATE

Updates records matching a filter.

**Syntax:**
```basic
rows = UPDATE "table", "filter", data
```

**Parameters:**
- `table` - Table name
- `filter` - Filter condition (e.g., "id=123" or "status=pending")
- `data` - Fields to update

**Example:**
```basic
updates = #{ "status": "completed", "completed_at": NOW() }
rows = UPDATE "orders", "id=ord-123", updates
TALK "Updated " + rows + " record(s)"
```

### DELETE

Deletes records from a table matching the filter.

**Syntax:**
```basic
rows = DELETE "table", "filter"
```

**Example:**
```basic
rows = DELETE "orders", "status=cancelled"
TALK "Deleted " + rows + " cancelled orders"
```

### MERGE

Merges data into a table using a key field for matching.

**Syntax:**
```basic
result = MERGE "table", data, "key_field"
```

**Example:**
```basic
' Import customers from external source
customers = GET "https://api.external.com/customers"
result = MERGE "customers", customers, "email"
TALK "Inserted: " + result.inserted + ", Updated: " + result.updated
```

### FILL

Transforms data by filling a template with values.

**Syntax:**
```basic
result = FILL data, template
```

**Example:**
```basic
data = [#{ "name": "John", "amount": 100 }, #{ "name": "Jane", "amount": 200 }]
template = #{ "greeting": "Hello {{name}}", "total": "Amount: ${{amount}}" }
filled = FILL data, template
```

### MAP

Maps field names from source to destination.

**Syntax:**
```basic
result = MAP data, "old_field->new_field, ..."
```

**Example:**
```basic
data = [#{ "firstName": "John", "lastName": "Doe" }]
mapped = MAP data, "firstName->name, lastName->surname"
' Result: [#{ "name": "John", "surname": "Doe" }]
```

### FILTER

Filters records based on a condition.

**Syntax:**
```basic
result = FILTER data, "condition"
```

**Supported operators:** `=`, `!=`, `>`, `<`, `>=`, `<=`, `like`

**Example:**
```basic
orders = FIND "orders.xlsx"
active = FILTER orders, "status=active"
highValue = FILTER orders, "amount>1000"
matches = FILTER orders, "name like john"
```

### AGGREGATE

Performs aggregation operations on data.

**Syntax:**
```basic
result = AGGREGATE "operation", data, "field"
```

**Operations:** `SUM`, `AVG`, `COUNT`, `MIN`, `MAX`

**Example:**
```basic
orders = FIND "orders.xlsx"
total = AGGREGATE "SUM", orders, "amount"
average = AGGREGATE "AVG", orders, "amount"
count = AGGREGATE "COUNT", orders, "id"
TALK "Total: $" + total + ", Average: $" + average + ", Count: " + count
```

### JOIN

Joins two datasets on a key field (inner join).

**Syntax:**
```basic
result = JOIN left_data, right_data, "key_field"
```

**Example:**
```basic
orders = FIND "orders.xlsx"
customers = FIND "customers.xlsx"
joined = JOIN orders, customers, "customer_id"
```

### PIVOT

Creates a pivot table from data.

**Syntax:**
```basic
result = PIVOT data, "row_field", "value_field"
```

**Example:**
```basic
sales = FIND "sales.xlsx"
byMonth = PIVOT sales, "month", "amount"
' Result: Each unique month with sum of amounts
```

### GROUP_BY

Groups data by a field.

**Syntax:**
```basic
result = GROUP_BY data, "field"
```

**Example:**
```basic
orders = FIND "orders.xlsx"
byStatus = GROUP_BY orders, "status"
' Result: Map with keys for each status value
```

---

## File & Document Operations

These keywords handle file operations within the `.gbdrive` storage.

### READ

Reads content from a file.

**Syntax:**
```basic
content = READ "path"
```

**Example:**
```basic
config = READ "config/settings.json"
data = READ "reports/daily.csv"
```

### WRITE

Writes content to a file.

**Syntax:**
```basic
WRITE "path", data
```

**Example:**
```basic
report = #{ "date": TODAY(), "total": 1500 }
WRITE "reports/summary.json", report
WRITE "logs/activity.txt", "User logged in at " + NOW()
```

### DELETE_FILE

Deletes a file from storage.

**Syntax:**
```basic
DELETE_FILE "path"
```

**Example:**
```basic
DELETE_FILE "temp/old-report.pdf"
```

### COPY

Copies a file to a new location.

**Syntax:**
```basic
COPY "source", "destination"
```

**Example:**
```basic
COPY "templates/invoice.docx", "customers/john/invoice-001.docx"
```

### MOVE

Moves or renames a file.

**Syntax:**
```basic
MOVE "source", "destination"
```

**Example:**
```basic
MOVE "inbox/new-file.pdf", "processed/file-001.pdf"
```

### LIST

Lists contents of a directory.

**Syntax:**
```basic
files = LIST "path"
```

**Example:**
```basic
reports = LIST "reports/"
FOR EACH file IN reports
    TALK "Found: " + file
NEXT file
```

### COMPRESS

Creates a ZIP archive from files.

**Syntax:**
```basic
archive = COMPRESS files, "archive_name.zip"
```

**Example:**
```basic
files = ["report1.pdf", "report2.pdf", "data.xlsx"]
archive = COMPRESS files, "monthly-reports.zip"
```

### EXTRACT

Extracts an archive to a destination folder.

**Syntax:**
```basic
files = EXTRACT "archive.zip", "destination/"
```

**Example:**
```basic
extracted = EXTRACT "uploads/documents.zip", "processed/"
TALK "Extracted " + UBOUND(extracted) + " files"
```

### UPLOAD

Uploads a file to storage.

**Syntax:**
```basic
url = UPLOAD file, "destination_path"
```

**Example:**
```basic
HEAR attachment AS FILE
url = UPLOAD attachment, "uploads/" + attachment.filename
TALK "File uploaded to: " + url
```

### DOWNLOAD

Downloads a file from a URL.

**Syntax:**
```basic
path = DOWNLOAD "url", "local_path"
```

**Example:**
```basic
path = DOWNLOAD "https://example.com/report.pdf", "downloads/report.pdf"
TALK "Downloaded to: " + path
```

### GENERATE_PDF

Generates a PDF from a template with data.

**Syntax:**
```basic
result = GENERATE_PDF "template", data, "output.pdf"
```

**Example:**
```basic
invoice = #{ "customer": "John", "items": items, "total": 299.99 }
pdf = GENERATE_PDF "templates/invoice.html", invoice, "invoices/inv-001.pdf"
TALK "PDF generated: " + pdf.url
```

### MERGE_PDF

Merges multiple PDF files into one.

**Syntax:**
```basic
result = MERGE_PDF files, "merged.pdf"
```

**Example:**
```basic
pdfs = ["cover.pdf", "chapter1.pdf", "chapter2.pdf", "appendix.pdf"]
merged = MERGE_PDF pdfs, "book.pdf"
```

---

## Webhook & Event-Driven Automation

### WEBHOOK

Creates a webhook endpoint that triggers script execution when called.

**Syntax:**
```basic
WEBHOOK "endpoint-name"
```

This registers an endpoint at: `/api/{botname}/webhook/{endpoint-name}`

When the webhook is called, the script containing the WEBHOOK declaration executes. Request parameters are available as variables.

**Example:**
```basic
' order-webhook.bas
WEBHOOK "order-received"

' Access request data
order_id = params.order_id
customer = body.customer_name
amount = body.total

' Process the order
SAVE "orders", order_id, body

' Send confirmation
SEND MAIL customer.email, "Order Confirmed", "Your order " + order_id + " is confirmed."

' Return response
result = #{ "status": "ok", "order_id": order_id }
```

**Webhook Request:**
```bash
curl -X POST https://bot.example.com/api/mybot/webhook/order-received \
  -H "Content-Type: application/json" \
  -d '{"customer_name": "John", "total": 99.99}'
```

---

## Notes

- Keywords are case-insensitive (TALK = talk = Talk)
- String parameters can use double quotes or single quotes
- Comments start with REM or '
- Line continuation uses underscore (_)
- Objects are created with `#{ key: value }` syntax
- Arrays use `[item1, item2, ...]` syntax