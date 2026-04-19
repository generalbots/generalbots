# File Operations

This section covers keywords for working with files in the bot's storage system. These keywords enable bots to read, write, copy, move, and manage files stored in the bot's drive bucket.

---

## Overview

General Bots provides a complete set of file operation keywords:

| Keyword | Purpose |
|---------|---------|
| [READ](keyword-read.md) | Load content from files |
| [WRITE](keyword-write.md) | Save content to files |
| [DELETE FILE](keyword-delete-file.md) | Remove files |
| [COPY](keyword-copy.md) | Copy files within storage |
| [MOVE](keyword-move.md) | Move or rename files |
| [LIST](keyword-list.md) | List files in a directory |
| [COMPRESS](keyword-compress.md) | Create ZIP archives |
| [EXTRACT](keyword-extract.md) | Extract archive contents |
| [UPLOAD](keyword-upload.md) | Upload files from URLs or users |
| [DOWNLOAD](keyword-download.md) | Send files to users |
| [GENERATE PDF](keyword-generate-pdf.md) | Create PDF documents |
| [MERGE PDF](keyword-merge-pdf.md) | Combine multiple PDFs |

---

## Quick Examples

### Basic File Operations

```basic
' Read a file
content = READ "documents/report.txt"
TALK content

' Write to a file
WRITE "Hello, World!" TO "greeting.txt"

' Append to a file
WRITE "New line\n" TO "log.txt" APPEND

' Delete a file
DELETE FILE "temp/old-file.txt"

' Copy a file
COPY "templates/form.docx" TO "user-forms/form-copy.docx"

' Move/rename a file
MOVE "inbox/message.txt" TO "archive/message.txt"

' List files in a directory
files = LIST "documents/"
FOR EACH file IN files
    TALK file.name + " (" + file.size + " bytes)"
NEXT
```

### Working with CSV Data

```basic
' Read CSV as structured data
customers = READ "data/customers.csv" AS TABLE

FOR EACH customer IN customers
    TALK customer.name + ": " + customer.email
NEXT

' Write data as CSV from database query
orders = FIND "orders" WHERE status = "pending" LIMIT 100
WRITE orders TO "exports/orders.csv" AS TABLE
```

### File Upload and Download

```basic
' Accept file from user
TALK "Please send me a document."
HEAR user_file
result = UPLOAD user_file TO "uploads/" + user.id
TALK "File saved: " + result.filename

' Send file to user
DOWNLOAD "reports/summary.pdf" AS "Monthly Summary.pdf"
TALK "Here's your report!"
```

### PDF Operations

```basic
' Generate PDF from template
GENERATE PDF "templates/invoice.html" TO "invoices/inv-001.pdf" WITH
    customer = "John Doe",
    amount = 150.00,
    date = FORMAT(NOW(), "YYYY-MM-DD")

' Merge multiple PDFs
MERGE PDF ["cover.pdf", "report.pdf", "appendix.pdf"] TO "complete-report.pdf"
```

### Archive Operations

```basic
' Create a ZIP archive
COMPRESS ["doc1.pdf", "doc2.pdf", "images/"] TO "package.zip"

' Extract archive contents
EXTRACT "uploaded.zip" TO "extracted/"
```

---

## Storage Structure

Files are stored in the bot's drive bucket with the following structure:

```
bot-name/
├── documents/
├── templates/
├── exports/
├── uploads/
│   └── user-123/
├── reports/
├── temp/
└── archives/
```

### Path Rules

| Path | Description |
|------|-------------|
| `file.txt` | Root of bot's storage |
| `folder/file.txt` | Subdirectory |
| `folder/sub/file.txt` | Nested subdirectory |
| `../file.txt` | **Not allowed** — no parent traversal |
| `/absolute/path` | **Not allowed** — paths are always relative |

```basic
' Valid paths
content = READ "documents/report.pdf"
WRITE data TO "exports/2025/january.csv"

' Invalid paths (will error)
' READ "../other-bot/file.txt"  ' Parent traversal blocked
' READ "/etc/passwd"            ' Absolute paths blocked
```

---

## Supported File Types

### Text Files

| Extension | Description |
|-----------|-------------|
| `.txt` | Plain text |
| `.md` | Markdown |
| `.json` | JSON data |
| `.csv` | Comma-separated values |
| `.xml` | XML data |
| `.html` | HTML documents |
| `.yaml` | YAML configuration |

### Documents

| Extension | Description | Auto-Extract |
|-----------|-------------|--------------|
| `.pdf` | PDF documents | ✓ Text extracted |
| `.docx` | Word documents | ✓ Text extracted |
| `.xlsx` | Excel spreadsheets | ✓ As table data |
| `.pptx` | PowerPoint | ✓ Text from slides |

### Media

| Extension | Description |
|-----------|-------------|
| `.jpg`, `.png`, `.gif` | Images |
| `.mp3`, `.wav` | Audio |
| `.mp4`, `.mov` | Video |

### Archives

| Extension | Description |
|-----------|-------------|
| `.zip` | ZIP archives |
| `.tar.gz` | Compressed tarballs |

---

## Common Patterns

### Template Processing

```basic
' Load template and fill placeholders
template = READ "templates/welcome-email.html"

email_body = REPLACE(template, "{{name}}", customer.name)
email_body = REPLACE(email_body, "{{date}}", FORMAT(NOW(), "MMMM DD, YYYY"))
email_body = REPLACE(email_body, "{{order_id}}", order.id)

SEND MAIL customer.email, "Welcome!", email_body
```

### Data Export

```basic
' Export query results to CSV
results = FIND "orders" WHERE status = "completed" AND date > "2025-01-01"
WRITE results TO "exports/completed-orders.csv" AS TABLE

' Generate download link
link = DOWNLOAD "exports/completed-orders.csv" AS LINK
TALK "Download your export: " + link
```

### Backup and Archive

```basic
' Create dated backup
backup_name = "backups/data-" + FORMAT(NOW(), "YYYYMMDD") + ".json"
data = GET BOT MEMORY "important_data"
WRITE JSON_STRINGIFY(data) TO backup_name

' Archive old files
old_files = LIST "reports/2024/"
COMPRESS old_files TO "archives/reports-2024.zip"

' Clean up originals
FOR EACH file IN old_files
    DELETE FILE file.path
NEXT
```

### File Validation

```basic
' Check file exists before processing
files = LIST "uploads/" + user.id + "/"
document_found = false

FOR EACH file IN files
    IF file.name = expected_filename THEN
        document_found = true
        EXIT FOR
    END IF
NEXT

IF document_found THEN
    content = READ "uploads/" + user.id + "/" + expected_filename
    ' Process content...
ELSE
    TALK "I couldn't find that document. Please upload it again."
END IF
```

### Organize Uploads

```basic
' Organize uploaded files by type
HEAR uploaded_file

file_type = uploaded_file.mime_type

IF INSTR(file_type, "image") > 0 THEN
    folder = "images"
ELSE IF INSTR(file_type, "pdf") > 0 THEN
    folder = "documents"
ELSE IF INSTR(file_type, "spreadsheet") > 0 OR INSTR(file_type, "excel") > 0 THEN
    folder = "spreadsheets"
ELSE
    folder = "other"
END IF

result = UPLOAD uploaded_file TO folder + "/" + FORMAT(NOW(), "YYYY/MM")
TALK "File saved to " + folder + "!"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

content = READ "documents/important.pdf"

IF ERROR THEN
    PRINT "File error: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't access that file. It may have been moved or deleted."
ELSE
    TALK "File loaded successfully!"
    ' Process content...
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | File doesn't exist | Check path, list directory first |
| `PERMISSION_DENIED` | Access blocked | Check file permissions |
| `PATH_TRAVERSAL` | Invalid path with `..` | Use only relative paths |
| `FILE_TOO_LARGE` | Exceeds size limit | Increase limit or split file |
| `INVALID_FORMAT` | Unsupported file type | Convert or use different format |

---

## Configuration

Configure file operations in `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
drive-read-timeout,30
drive-write-timeout,60
drive-max-file-size,52428800
drive-allowed-extensions,pdf,docx,xlsx,jpg,png,csv,json
```

---

## Size Limits

| Operation | Default Limit | Configurable |
|-----------|---------------|--------------|
| Read file | 50 MB | Yes |
| Write file | 50 MB | Yes |
| Upload file | 50 MB | Yes |
| Total storage | 10 GB per bot | Yes |
| Files per directory | 10,000 | Yes |

---

## Security Considerations

1. **Path validation** — All paths are sanitized to prevent directory traversal
2. **File type restrictions** — Executable files blocked by default
3. **Size limits** — Prevents storage exhaustion attacks
4. **Access control** — Files isolated per bot
5. **Malware scanning** — Uploaded files scanned before storage

---

## See Also

- [READ](keyword-read.md) — Load file content
- [WRITE](keyword-write.md) — Save content to files
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [COPY](keyword-copy.md) — Copy files
- [MOVE](keyword-move.md) — Move/rename files
- [LIST](keyword-list.md) — List directory contents
- [COMPRESS](keyword-compress.md) — Create archives
- [EXTRACT](keyword-extract.md) — Extract archives
- [UPLOAD](keyword-upload.md) — Upload files
- [DOWNLOAD](keyword-download.md) — Send files to users
- [GENERATE PDF](keyword-generate-pdf.md) — Create PDFs
- [MERGE PDF](keyword-merge-pdf.md) — Combine PDFs