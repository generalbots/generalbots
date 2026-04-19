# DOWNLOAD

The `DOWNLOAD` keyword retrieves files from the bot's storage and sends them to users or saves them to external locations, enabling bots to share documents, export data, and deliver files through chat channels.

---

## Syntax

```basic
DOWNLOAD "filename"
DOWNLOAD "filename" TO user
DOWNLOAD "filename" AS "display_name"
url = DOWNLOAD "filename" AS LINK
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `filename` | String | Path to the file in the bot's storage |
| `TO user` | Flag | Send file to specific user (default: current user) |
| `AS "name"` | String | Custom display name for the file |
| `AS LINK` | Flag | Return a download URL instead of sending file |

---

## Description

`DOWNLOAD` retrieves a file from the bot's configured storage (drive bucket) and delivers it to the user through their chat channel. It supports:

- Sending files directly in chat (WhatsApp, Telegram, web, etc.)
- Custom display names for downloaded files
- Generating shareable download links
- Sending files to specific users
- Automatic MIME type detection

The file path is relative to the bot's storage root. Use forward slashes for subdirectories.

---

## Examples

### Basic File Download

```basic
' Send a file to the current user
DOWNLOAD "documents/user-guide.pdf"
TALK "Here's the user guide you requested!"
```

### Download with Custom Name

```basic
' Send file with a friendly display name
DOWNLOAD "reports/rpt-2025-01.pdf" AS "January 2025 Report.pdf"
```

### Generate Download Link

```basic
' Get a shareable URL instead of sending directly
link = DOWNLOAD "exports/data.xlsx" AS LINK
TALK "Download your data here: " + link

' Link expires after 24 hours by default
```

### Send to Specific User

```basic
' Send file to a different user
DOWNLOAD "contracts/agreement.pdf" TO manager_email
TALK "I've sent the contract to your manager for review."
```

### Download After Processing

```basic
' Generate a report and send it
report_content = "# Sales Report\n\n" + sales_data
WRITE report_content TO "temp/report.md"

' Convert to PDF (if configured)
GENERATE PDF "temp/report.md" TO "temp/report.pdf"

DOWNLOAD "temp/report.pdf" AS "Sales Report.pdf"
TALK "Here's your sales report!"
```

---

## Common Use Cases

### Send Invoice

```basic
' Lookup and send customer invoice
invoice_path = "invoices/" + customer_id + "/" + invoice_id + ".pdf"

DOWNLOAD invoice_path AS "Invoice-" + invoice_id + ".pdf"
TALK "Here's your invoice. Let me know if you have any questions!"
```

### Export Data

```basic
' Export user's data to file and send
user_data = FIND "orders" WHERE customer_id = user.id
WRITE user_data TO "exports/user-" + user.id + "-orders.csv" AS TABLE

DOWNLOAD "exports/user-" + user.id + "-orders.csv" AS "My Orders.csv"
TALK "Here's a complete export of your order history."
```

### Share Meeting Notes

```basic
' Send meeting notes from earlier session
meeting_date = FORMAT(NOW(), "YYYY-MM-DD")
notes_file = "meetings/" + meeting_date + "-notes.md"

IF FILE_EXISTS(notes_file) THEN
    DOWNLOAD notes_file AS "Meeting Notes - " + meeting_date + ".md"
    TALK "Here are the notes from today's meeting!"
ELSE
    TALK "I don't have any meeting notes for today."
END IF
```

### Provide Template

```basic
' Send a template file for user to fill out
TALK "I'll send you the application form. Please fill it out and send it back."
DOWNLOAD "templates/application-form.docx" AS "Application Form.docx"
```

### Generate and Share Report

```basic
' Create report on demand
TALK "Generating your monthly report..."

' Build report content
report = "# Monthly Summary\n\n"
report = report + "**Period:** " + month_name + " " + year + "\n\n"
report = report + "## Key Metrics\n\n"
report = report + "- Revenue: $" + FORMAT(revenue, "#,##0.00") + "\n"
report = report + "- Orders: " + order_count + "\n"
report = report + "- New Customers: " + new_customers + "\n"

' Save and send
filename = "reports/monthly-" + FORMAT(NOW(), "YYYYMM") + ".md"
WRITE report TO filename
DOWNLOAD filename AS "Monthly Report - " + month_name + ".md"
```

### Send Multiple Files

```basic
' Send several related files
files = ["contract.pdf", "terms.pdf", "schedule.pdf"]

TALK "I'm sending you the complete documentation package:"

FOR EACH file IN files
    DOWNLOAD "documents/" + file
    WAIT 1  ' Brief pause between files
NEXT

TALK "All documents sent! Please review and let me know if you have questions."
```

---

## Return Values

### Direct Download (default)

Returns a confirmation object:

| Property | Description |
|----------|-------------|
| `result.sent` | Boolean indicating success |
| `result.filename` | Name of file sent |
| `result.size` | File size in bytes |

### Download as Link

Returns a URL string:

```basic
link = DOWNLOAD "file.pdf" AS LINK
' Returns: "https://storage.example.com/download/abc123?expires=..."
```

---

## Channel-Specific Behavior

| Channel | Behavior |
|---------|----------|
| **WhatsApp** | Sends as document attachment |
| **Telegram** | Sends as document or media based on type |
| **Web Chat** | Triggers browser download |
| **Email** | Attaches to email message |
| **SMS** | Sends download link (files not supported) |

---

## File Type Handling

| File Type | Display |
|-----------|---------|
| PDF | Document with preview |
| Images | Inline image display |
| Audio | Audio player |
| Video | Video player |
| Other | Generic document icon |

```basic
' Images display inline in most channels
DOWNLOAD "photos/product.jpg"

' PDFs show with document preview
DOWNLOAD "docs/manual.pdf"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

DOWNLOAD "reports/missing-file.pdf"

IF ERROR THEN
    PRINT "Download failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't find that file. It may have been moved or deleted."
END IF
```

### Check File Exists First

```basic
files = LIST "invoices/" + customer_id + "/"

found = false
FOR EACH file IN files
    IF file.name = invoice_id + ".pdf" THEN
        found = true
        EXIT FOR
    END IF
NEXT

IF found THEN
    DOWNLOAD "invoices/" + customer_id + "/" + invoice_id + ".pdf"
ELSE
    TALK "Invoice not found. Please check the invoice number."
END IF
```

---

## Link Options

When using `AS LINK`, you can configure link behavior:

```basic
' Default link (expires in 24 hours)
link = DOWNLOAD "file.pdf" AS LINK

' Custom expiration (in config.csv)
' download-link-expiry,3600  (1 hour)
```

---

## Size Limits

| Limit | Default | Notes |
|-------|---------|-------|
| WhatsApp | 100 MB | Documents, 16 MB for media |
| Telegram | 50 MB | Standard, 2 GB for premium |
| Web Chat | No limit | Browser handles download |
| Email | 25 MB | Typical email limit |

```basic
' For large files, use link instead
file_info = LIST "exports/large-file.zip"

IF file_info[0].size > 50000000 THEN
    link = DOWNLOAD "exports/large-file.zip" AS LINK
    TALK "This file is large. Download it here: " + link
ELSE
    DOWNLOAD "exports/large-file.zip"
END IF
```

---

## Configuration

Configure download settings in `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
download-link-expiry,86400
download-link-base-url,https://files.mybot.com
download-max-size,104857600
```

---

## Implementation Notes

- Implemented in Rust under `src/file/mod.rs`
- Uses streaming for large file transfers
- Automatic MIME type detection
- Supports range requests for resumable downloads
- Files are served through secure signed URLs
- Access logging for audit trails

---

## Related Keywords

- [UPLOAD](keyword-upload.md) — Upload files to storage
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write content to files
- [LIST](keyword-list.md) — List files in storage
- [GENERATE PDF](keyword-generate-pdf.md) — Create PDF documents

---

## Summary

`DOWNLOAD` is essential for delivering files to users through chat. Use it to send invoices, share reports, provide templates, and export data. Combined with `AS LINK` for large files and custom display names, it provides flexible file delivery for any bot workflow.