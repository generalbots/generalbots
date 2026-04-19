# WRITE

The `WRITE` keyword saves content to files in the bot's drive storage, enabling bots to create documents, export data, and persist information.

---

## Syntax

```basic
WRITE content TO "filename"
WRITE data TO "filename.csv" AS TABLE
WRITE lines TO "filename.txt" AS LINES
WRITE content TO "filename" APPEND
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `content` | String | The content to write to the file |
| `filename` | String | Path to the file in the bot's storage |
| `AS TABLE` | Flag | Write structured data as CSV format |
| `AS LINES` | Flag | Write array as separate lines |
| `APPEND` | Flag | Add to existing file instead of overwriting |

---

## Description

`WRITE` saves content to the bot's configured storage (drive bucket). It supports:

- Text files (`.txt`, `.md`, `.json`, `.xml`, `.csv`)
- Creating new files or overwriting existing ones
- Appending to existing files
- Writing structured data as CSV
- Automatic directory creation

The file path is relative to the bot's storage root. Use forward slashes for subdirectories.

---

## Examples

### Basic File Write

```basic
' Write a simple text file
message = "Welcome to our service!"
WRITE message TO "welcome.txt"

TALK "File saved successfully!"
```

### Write to Subdirectory

```basic
' Write file to nested folder (directories created automatically)
report = "Monthly Report\n\nSales: $10,000\nExpenses: $3,000"
WRITE report TO "reports/2025/january.md"
```

### Write JSON Data

```basic
' Create JSON configuration file
config_json = '{"theme": "dark", "language": "en", "notifications": true}'
WRITE config_json TO "settings.json"
```

### Write CSV as Table

```basic
' Export data as CSV - use FIND to get data from database
orders = FIND "orders" WHERE status = "completed" LIMIT 100

WRITE orders TO "exports/orders.csv" AS TABLE
TALK "Exported " + LEN(orders) + " orders to CSV"
```

### Write Lines

```basic
' Write array as separate lines
log_entries = [
    "2025-01-15 10:00 - User logged in",
    "2025-01-15 10:05 - Order placed",
    "2025-01-15 10:10 - Payment processed"
]

WRITE log_entries TO "logs/activity.log" AS LINES
```

### Append to File

```basic
' Add entry to existing log file
new_entry = FORMAT(NOW(), "YYYY-MM-DD HH:mm") + " - " + event_description + "\n"
WRITE new_entry TO "logs/events.log" APPEND
```

---

## Common Use Cases

### Generate Report

```basic
' Create a formatted report
report = "# Sales Report\n\n"
report = report + "**Date:** " + FORMAT(NOW(), "MMMM DD, YYYY") + "\n\n"
report = report + "## Summary\n\n"
report = report + "- Total Sales: $" + FORMAT(total_sales, "#,##0.00") + "\n"
report = report + "- Orders: " + order_count + "\n"
report = report + "- Average Order: $" + FORMAT(total_sales / order_count, "#,##0.00") + "\n"

filename = "reports/sales-" + FORMAT(NOW(), "YYYYMMDD") + ".md"
WRITE report TO filename

TALK "Report saved to " + filename
```

### Export Customer Data

```basic
' Export customer list to CSV
customers = FIND "customers" WHERE status = "active"

WRITE customers TO "exports/active-customers.csv" AS TABLE

' Email the export
SEND MAIL "manager@company.com", "Customer Export", "See attached file", "exports/active-customers.csv"
```

### Save Meeting Notes

```basic
' Save notes from a conversation
notes = "# Meeting Notes\n\n"
notes = notes + "**Date:** " + FORMAT(NOW(), "YYYY-MM-DD HH:mm") + "\n"
notes = notes + "**Participants:** " + participants + "\n\n"
notes = notes + "## Discussion\n\n"
notes = notes + meeting_content + "\n\n"
notes = notes + "## Action Items\n\n"
notes = notes + action_items

filename = "meetings/" + FORMAT(NOW(), "YYYYMMDD") + "-" + meeting_topic + ".md"
WRITE notes TO filename

TALK "Meeting notes saved!"
```

### Create Backup

```basic
' Backup current data
data = GET BOT MEMORY "important_data"
backup_name = "backups/data-" + FORMAT(NOW(), "YYYYMMDD-HHmmss") + ".json"
WRITE JSON_STRINGIFY(data) TO backup_name

TALK "Backup created: " + backup_name
```

### Build Log File

```basic
' Append to daily log
log_line = FORMAT(NOW(), "HH:mm:ss") + " | " + user_id + " | " + action + " | " + details
log_file = "logs/" + FORMAT(NOW(), "YYYY-MM-DD") + ".log"

WRITE log_line + "\n" TO log_file APPEND
```

### Generate HTML Page

```basic
' Create a simple HTML report
html = "<!DOCTYPE html>\n"
html = html + "<html><head><title>Report</title></head>\n"
html = html + "<body>\n"
html = html + "<h1>Daily Summary</h1>\n"
html = html + "<p>Generated: " + FORMAT(NOW(), "YYYY-MM-DD HH:mm") + "</p>\n"
html = html + "<ul>\n"

FOR EACH item IN summary_items
    html = html + "<li>" + item + "</li>\n"
NEXT

html = html + "</ul>\n"
html = html + "</body></html>"

WRITE html TO "reports/daily-summary.html"
```

---

## Writing Different Formats

### Plain Text

```basic
WRITE "Hello, World!" TO "greeting.txt"
```

### Markdown

```basic
doc = "# Title\n\n## Section 1\n\nContent here.\n"
WRITE doc TO "document.md"
```

### JSON

```basic
json_text = '{"name": "Test", "value": 123}'
WRITE json_text TO "data.json"
```

### CSV (Manual)

```basic
csv = "name,email,phone\n"
csv = csv + "Alice,alice@example.com,555-0100\n"
csv = csv + "Bob,bob@example.com,555-0101\n"
WRITE csv TO "contacts.csv"
```

### CSV (From Table)

```basic
' Write query results as CSV
data = FIND "contacts" WHERE active = true
WRITE data TO "contacts.csv" AS TABLE
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

WRITE content TO "protected/file.txt"

IF ERROR THEN
    PRINT "Write failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't save the file. Please try again."
ELSE
    TALK "File saved successfully!"
END IF
```

---

## File Path Rules

| Path | Description |
|------|-------------|
| `file.txt` | Root of bot's storage |
| `folder/file.txt` | Subdirectory (created if needed) |
| `folder/sub/file.txt` | Nested subdirectory |
| `../file.txt` | **Not allowed** — no parent traversal |
| `/absolute/path` | **Not allowed** — paths are always relative |

---

## Overwrite vs Append

| Mode | Behavior |
|------|----------|
| Default | Overwrites existing file completely |
| `APPEND` | Adds content to end of existing file |

```basic
' Overwrite (default)
WRITE "New content" TO "file.txt"

' Append
WRITE "Additional content\n" TO "file.txt" APPEND
```

---

## Configuration

Configure storage settings in `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
drive-write-timeout,60
drive-max-file-size,52428800
```

---

## Implementation Notes

- Implemented in Rust under `src/file/mod.rs`
- Automatically creates parent directories
- Uses UTF-8 encoding for text files
- Maximum file size: 50MB (configurable)
- Atomic writes to prevent corruption
- Returns confirmation on success

---

## Related Keywords

- [READ](keyword-read.md) — Load content from files
- [LIST](keyword-list.md) — List files in a directory
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [COPY](keyword-copy.md) — Copy files
- [MOVE](keyword-move.md) — Move or rename files
- [UPLOAD](keyword-upload.md) — Upload files to storage

---

## Summary

`WRITE` is the primary keyword for creating and saving files. Use it to generate reports, export data, create backups, build logs, and persist any content. Combined with `AS TABLE` for CSV exports and `APPEND` for log files, it provides flexible file creation capabilities for any bot workflow.