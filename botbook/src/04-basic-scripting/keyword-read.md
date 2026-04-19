# READ

The `READ` keyword loads content from files stored in the bot's drive storage, enabling bots to access documents, data files, and other stored resources.

---

## Syntax

```basic
content = READ "filename"
content = READ "path/to/filename"
data = READ "filename.csv" AS TABLE
lines = READ "filename.txt" AS LINES
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `filename` | String | Path to the file in the bot's storage |
| `AS TABLE` | Flag | Parse CSV/Excel files as structured data |
| `AS LINES` | Flag | Return content as array of lines |

---

## Description

`READ` retrieves file content from the bot's configured storage (drive bucket). It supports:

- Text files (`.txt`, `.md`, `.json`, `.xml`, `.csv`)
- Documents (`.pdf`, `.docx`) — automatically extracts text
- Spreadsheets (`.xlsx`, `.csv`) — can parse as structured data
- Binary files — returned as base64 encoded string

The file path is relative to the bot's storage root. Use forward slashes for subdirectories.

---

## Examples

### Basic File Read

```basic
' Read a text file
content = READ "welcome-message.txt"
TALK content
```

### Read from Subdirectory

```basic
' Read file from nested folder
template = READ "templates/email/welcome.html"
```

### Read JSON Data

```basic
' Read and parse JSON configuration
config_text = READ "config.json"
config = JSON_PARSE(config_text)

TALK "Current theme: " + config.theme
```

### Read CSV as Table

```basic
' Load CSV data as structured table
products = READ "inventory/products.csv" AS TABLE

FOR EACH product IN products
    TALK product.name + ": $" + product.price
NEXT
```

### Read as Lines

```basic
' Read file as array of lines
faq_lines = READ "faq.txt" AS LINES

TALK "We have " + LEN(faq_lines) + " FAQ entries"

FOR EACH line IN faq_lines
    IF INSTR(line, user_question) > 0 THEN
        TALK "Found relevant FAQ: " + line
    END IF
NEXT
```

### Read PDF Document

```basic
' Extract text from PDF
contract_text = READ "documents/contract.pdf"
TALK "Contract length: " + LEN(contract_text) + " characters"

' Use LLM to analyze
summary = LLM "Summarize the key points of this contract:\n\n" + contract_text
TALK summary
```

### Read Excel Spreadsheet

```basic
' Load Excel data
sales_data = READ "reports/sales-q1.xlsx" AS TABLE

total = 0
FOR EACH row IN sales_data
    total = total + row.amount
NEXT

TALK "Total Q1 sales: $" + FORMAT(total, "#,##0.00")
```

---

## Working with Different File Types

### Text Files

```basic
' Plain text - returned as string
notes = READ "notes.txt"
readme = READ "README.md"
```

### JSON Files

```basic
' JSON - returned as string, use JSON_PARSE for object
json_text = READ "data.json"
data = JSON_PARSE(json_text)
```

### CSV Files

```basic
' CSV as string
csv_raw = READ "data.csv"

' CSV as structured table (recommended)
csv_data = READ "data.csv" AS TABLE
first_row = csv_data[0]
```

### Documents

```basic
' PDF - text extracted automatically
pdf_content = READ "report.pdf"

' Word documents - text extracted automatically
doc_content = READ "proposal.docx"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

content = READ "optional-file.txt"

IF ERROR THEN
    PRINT "File not found, using default"
    content = "Default content"
END IF
```

### Check File Exists

```basic
' List directory to check if file exists
files = LIST "documents/"

found = false
FOR EACH file IN files
    IF file.name = "report.pdf" THEN
        found = true
        EXIT FOR
    END IF
NEXT

IF found THEN
    content = READ "documents/report.pdf"
ELSE
    TALK "Report not found. Would you like me to generate one?"
END IF
```

---

## Common Use Cases

### Load Email Template

```basic
' Read HTML template and fill variables
template = READ "templates/order-confirmation.html"

' Replace placeholders
email_body = REPLACE(template, "{{customer_name}}", customer.name)
email_body = REPLACE(email_body, "{{order_id}}", order.id)
email_body = REPLACE(email_body, "{{total}}", FORMAT(order.total, "$#,##0.00"))

SEND MAIL customer.email, "Order Confirmation", email_body
```

### Process Data File

```basic
' Read customer list and send personalized messages
customers = READ "campaigns/target-customers.csv" AS TABLE

FOR EACH customer IN customers
    IF customer.opted_in = "yes" THEN
        message = "Hi " + customer.first_name + ", check out our new products!"
        SEND SMS customer.phone, message
    END IF
NEXT

TALK "Campaign sent to " + LEN(customers) + " customers"
```

### Load Bot Configuration

```basic
' Read bot settings from file
settings_text = READ "bot-settings.json"
settings = JSON_PARSE(settings_text)

' Apply settings
SET BOT MEMORY "greeting", settings.greeting
SET BOT MEMORY "language", settings.language
SET BOT MEMORY "max_retries", settings.max_retries
```

### Knowledge Base Lookup

```basic
' Read FAQ document for quick lookups
faq_content = READ "knowledge/faq.md"

' Search for relevant section
IF INSTR(user_question, "return") > 0 THEN
    ' Extract return policy section
    start_pos = INSTR(faq_content, "## Return Policy")
    end_pos = INSTR(faq_content, "##", start_pos + 1)
    policy = MID(faq_content, start_pos, end_pos - start_pos)
    TALK policy
END IF
```

---

## File Path Rules

| Path | Description |
|------|-------------|
| `file.txt` | Root of bot's storage |
| `folder/file.txt` | Subdirectory |
| `folder/sub/file.txt` | Nested subdirectory |
| `../file.txt` | **Not allowed** — no parent traversal |
| `/absolute/path` | **Not allowed** — paths are always relative |

---

## Configuration

Configure storage settings in `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
drive-read-timeout,30
```

---

## Implementation Notes

- Implemented in Rust under `src/file/mod.rs`
- Automatically detects file encoding (UTF-8, UTF-16, etc.)
- PDF extraction uses `pdf-extract` crate
- DOCX extraction parses XML content
- Maximum file size: 50MB (configurable)
- Files are cached in memory for repeated reads

---

## Related Keywords

- [WRITE](keyword-write.md) — Save content to files
- [LIST](keyword-list.md) — List files in a directory
- [DOWNLOAD](keyword-download.md) — Download files from URLs
- [UPLOAD](keyword-upload.md) — Upload files to storage
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [GET](keyword-get.md) — Read from URLs or files

---

## Summary

`READ` is the primary keyword for accessing stored files. It handles text extraction from various document formats, supports structured data parsing for CSV/Excel files, and integrates seamlessly with the bot's storage system. Use it to load templates, process data files, access configuration, and work with uploaded documents.