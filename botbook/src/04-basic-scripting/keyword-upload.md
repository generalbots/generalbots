# UPLOAD

The `UPLOAD` keyword transfers files from external URLs or local paths to the bot's drive storage, enabling bots to collect documents, images, and other files from users or external sources.

---

## Syntax

```basic
result = UPLOAD url
result = UPLOAD url TO "destination"
result = UPLOAD url TO "destination" AS "filename"
UPLOAD file_data TO "destination"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `url` | String | Source URL to download and upload |
| `destination` | String | Target folder in bot's storage |
| `filename` | String | Custom filename (optional) |
| `file_data` | Binary | File data from user input or API response |

---

## Description

`UPLOAD` retrieves a file from a URL or accepts file data and stores it in the bot's configured storage (drive bucket). It supports:

- Downloading files from external URLs
- Accepting file uploads from chat users
- Storing API response attachments
- Organizing files into folders
- Automatic filename detection or custom naming

The destination path is relative to the bot's storage root. Directories are created automatically if they don't exist.

---

## Examples

### Basic URL Upload

```basic
' Download and store a file from URL
result = UPLOAD "https://example.com/report.pdf"
TALK "File saved as: " + result.filename
```

### Upload to Specific Folder

```basic
' Upload to a specific directory
result = UPLOAD "https://cdn.example.com/image.png" TO "images/products"
TALK "Image stored at: " + result.path
```

### Upload with Custom Filename

```basic
' Upload with a custom name
result = UPLOAD "https://api.example.com/export/data" TO "exports" AS "monthly-report.xlsx"
TALK "Report saved as: " + result.filename
```

### Handle User File Upload

```basic
' When user sends a file via WhatsApp/chat
TALK "Please send me the document you'd like to upload."
HEAR user_file

IF user_file.type = "file" THEN
    result = UPLOAD user_file TO "user-uploads/" + user.id
    TALK "Got it! I've saved your file: " + result.filename
ELSE
    TALK "That doesn't look like a file. Please try again."
END IF
```

### Upload from API Response

```basic
' Download attachment from external API
invoice_url = GET "https://api.billing.com/invoices/" + invoice_id + "/pdf"
result = UPLOAD invoice_url.download_url TO "invoices/" + customer_id

TALK "Invoice downloaded and saved!"
SEND MAIL customer_email, "Your Invoice", "Please find your invoice attached.", result.path
```

---

## Return Value

`UPLOAD` returns an object with:

| Property | Description |
|----------|-------------|
| `result.path` | Full path in storage |
| `result.filename` | Name of the saved file |
| `result.size` | File size in bytes |
| `result.type` | MIME type of the file |
| `result.url` | Internal URL to access the file |

---

## Common Use Cases

### Collect User Documents

```basic
' Document collection flow
TALK "I need a few documents to process your application."

TALK "First, please upload your ID document."
HEAR id_doc
id_result = UPLOAD id_doc TO "applications/" + application_id + "/documents" AS "id-document"

TALK "Great! Now please upload proof of address."
HEAR address_doc
address_result = UPLOAD address_doc TO "applications/" + application_id + "/documents" AS "proof-of-address"

TALK "Thank you! I've received:"
TALK "✓ ID Document: " + id_result.filename
TALK "✓ Proof of Address: " + address_result.filename
```

### Archive External Content

```basic
' Download and archive web content
urls = [
    "https://example.com/report-2024.pdf",
    "https://example.com/report-2025.pdf"
]

FOR EACH url IN urls
    result = UPLOAD url TO "archive/reports"
    TALK "Archived: " + result.filename
NEXT

TALK "All reports archived successfully!"
```

### Profile Photo Upload

```basic
TALK "Would you like to update your profile photo? Send me an image."
HEAR photo

IF photo.type = "image" THEN
    result = UPLOAD photo TO "profiles" AS user.id + "-avatar"
    SET USER MEMORY "avatar_url", result.url
    TALK "Profile photo updated! Looking good! 📸"
ELSE
    TALK "Please send an image file."
END IF
```

### Backup External Data

```basic
' Backup data from external service
backup_url = "https://api.service.com/export?format=json&date=" + FORMAT(NOW(), "YYYY-MM-DD")
SET HEADER "Authorization", "Bearer " + api_token

result = UPLOAD backup_url TO "backups" AS "backup-" + FORMAT(NOW(), "YYYYMMDD") + ".json"

TALK "Backup complete: " + FORMAT(result.size / 1024, "#,##0") + " KB"
```

### Receipt Collection

```basic
' Expense report receipt upload
TALK "Please upload your receipt for the expense."
HEAR receipt

result = UPLOAD receipt TO "expenses/" + expense_id + "/receipts"

' Update expense record
UPDATE "expenses" SET receipt_path = result.path WHERE id = expense_id

TALK "Receipt attached to expense #" + expense_id
```

---

## Supported File Types

| Category | Extensions |
|----------|------------|
| Documents | `.pdf`, `.docx`, `.doc`, `.txt`, `.md`, `.rtf` |
| Spreadsheets | `.xlsx`, `.xls`, `.csv` |
| Images | `.jpg`, `.jpeg`, `.png`, `.gif`, `.webp`, `.svg` |
| Archives | `.zip`, `.tar`, `.gz`, `.rar` |
| Audio | `.mp3`, `.wav`, `.ogg`, `.m4a` |
| Video | `.mp4`, `.mov`, `.avi`, `.webm` |
| Data | `.json`, `.xml`, `.yaml` |

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = UPLOAD "https://example.com/large-file.zip" TO "downloads"

IF ERROR THEN
    PRINT "Upload failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't download that file. The server might be unavailable."
ELSE IF result.size > 50000000 THEN
    TALK "Warning: This is a large file (" + FORMAT(result.size / 1048576, "#,##0") + " MB)"
ELSE
    TALK "File uploaded successfully!"
END IF
```

### Validate File Type

```basic
HEAR user_file

allowed_types = ["application/pdf", "image/jpeg", "image/png"]

IF NOT CONTAINS(allowed_types, user_file.mime_type) THEN
    TALK "Sorry, I only accept PDF and image files."
ELSE
    result = UPLOAD user_file TO "uploads"
    TALK "File accepted!"
END IF
```

---

## Size Limits

| Limit | Default | Configurable |
|-------|---------|--------------|
| Maximum file size | 50 MB | Yes |
| Maximum files per folder | 10,000 | Yes |
| Total storage per bot | 10 GB | Yes |

---

## Configuration

Configure upload settings in `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
upload-max-size,52428800
upload-allowed-types,pdf,docx,xlsx,jpg,png
upload-timeout,120
```

---

## Security Considerations

- Files are scanned for malware before storage
- Executable files (`.exe`, `.sh`, `.bat`) are blocked by default
- File paths are sanitized to prevent directory traversal
- Original filenames are preserved but sanitized
- Large files are chunked for reliable upload

---

## Implementation Notes

- Implemented in Rust under `src/file/mod.rs`
- Uses streaming upload for large files
- Supports resume for interrupted uploads
- Automatic retry on network failures (up to 3 attempts)
- Progress tracking available for large files
- Deduplication based on content hash (optional)

---

## Related Keywords

- [DOWNLOAD](keyword-download.md) — Download files to user
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write content to files
- [LIST](keyword-list.md) — List files in storage
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [COPY](keyword-copy.md) — Copy files within storage

---

## Summary

`UPLOAD` is essential for collecting files from users and external sources. Use it to accept document uploads, archive web content, collect receipts and photos, and store API response attachments. Combined with folder organization and custom naming, it provides flexible file collection for any bot workflow.