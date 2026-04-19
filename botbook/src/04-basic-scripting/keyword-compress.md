# COMPRESS

The `COMPRESS` keyword creates ZIP archives from files and directories in the bot's storage, enabling bots to bundle multiple files for download or transfer.

---

## Syntax

```basic
COMPRESS files TO "archive.zip"
result = COMPRESS files TO "archive.zip"
COMPRESS "folder/" TO "archive.zip"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `files` | Array/String | List of file paths or a single folder path |
| `TO` | Clause | Destination path for the archive |

---

## Description

`COMPRESS` creates a ZIP archive containing the specified files or directory contents. The archive is stored in the bot's drive storage and can be downloaded, emailed, or transferred.

Use cases include:
- Bundling multiple documents for download
- Creating backups
- Packaging exports for users
- Archiving old files
- Preparing files for email attachments

---

## Examples

### Compress Multiple Files

```basic
' Create archive from list of files
files = ["report.pdf", "data.csv", "images/logo.png"]
COMPRESS files TO "package.zip"

TALK "Files compressed into package.zip"
```

### Compress a Folder

```basic
' Compress entire folder contents
COMPRESS "documents/project/" TO "project-backup.zip"

TALK "Project folder compressed"
```

### Compress with Result

```basic
' Get compression result details
result = COMPRESS files TO "exports/archive.zip"

TALK "Archive created: " + result.filename
TALK "Size: " + FORMAT(result.size / 1024, "#,##0") + " KB"
TALK "Files included: " + result.file_count
```

### Compress for Download

```basic
' Create archive and send to user
files = LIST "reports/" FILTER "*.pdf"
file_paths = []

FOR EACH file IN files
    file_paths = APPEND(file_paths, "reports/" + file.name)
NEXT

result = COMPRESS file_paths TO "all-reports.zip"

DOWNLOAD "all-reports.zip" AS "Your Reports.zip"
TALK "Here are all your reports in a single download!"
```

### Compress with Timestamp

```basic
' Create dated archive
timestamp = FORMAT(NOW(), "YYYYMMDD-HHmmss")
archive_name = "backup-" + timestamp + ".zip"

COMPRESS "data/" TO "backups/" + archive_name

TALK "Backup created: " + archive_name
```

---

## Common Use Cases

### Create Document Package

```basic
' Bundle documents for a customer
customer_files = [
    "contracts/" + customer_id + "/agreement.pdf",
    "contracts/" + customer_id + "/terms.pdf",
    "invoices/" + customer_id + "/latest.pdf"
]

result = COMPRESS customer_files TO "temp/customer-package.zip"

DOWNLOAD "temp/customer-package.zip" AS "Your Documents.zip"
TALK "Here's your complete document package!"
```

### Archive Old Data

```basic
' Archive and remove old files
old_files = LIST "logs/" FILTER "*" WHERE modified < DATEADD(NOW(), -90, "day")
file_paths = []

FOR EACH file IN old_files
    file_paths = APPEND(file_paths, "logs/" + file.name)
NEXT

IF LEN(file_paths) > 0 THEN
    archive_name = "logs-archive-" + FORMAT(NOW(), "YYYYMM") + ".zip"
    COMPRESS file_paths TO "archives/" + archive_name
    
    ' Remove original files
    FOR EACH path IN file_paths
        DELETE FILE path
    NEXT
    
    TALK "Archived " + LEN(file_paths) + " old log files"
END IF
```

### Export User Data

```basic
' GDPR data export
user_folder = "users/" + user.id + "/"

COMPRESS user_folder TO "exports/user-data-" + user.id + ".zip"

link = DOWNLOAD "exports/user-data-" + user.id + ".zip" AS LINK
TALK "Your data export is ready: " + link
TALK "This link expires in 24 hours."
```

### Email Attachment Bundle

```basic
' Create attachment for email
attachments = [
    "reports/summary.pdf",
    "reports/details.xlsx",
    "reports/charts.png"
]

COMPRESS attachments TO "temp/report-bundle.zip"

SEND MAIL recipient_email, "Monthly Report Bundle", 
    "Please find attached the complete monthly report package.",
    "temp/report-bundle.zip"

TALK "Report bundle sent to " + recipient_email
```

---

## Return Value

Returns an object with archive details:

| Property | Description |
|----------|-------------|
| `result.path` | Full path to the archive |
| `result.filename` | Archive filename |
| `result.size` | Archive size in bytes |
| `result.file_count` | Number of files in archive |
| `result.created_at` | Creation timestamp |

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = COMPRESS files TO "archive.zip"

IF ERROR THEN
    PRINT "Compression failed: " + ERROR_MESSAGE
    
    IF INSTR(ERROR_MESSAGE, "not found") > 0 THEN
        TALK "One or more files could not be found."
    ELSE IF INSTR(ERROR_MESSAGE, "storage") > 0 THEN
        TALK "Not enough storage space for the archive."
    ELSE
        TALK "Sorry, I couldn't create the archive. Please try again."
    END IF
ELSE
    TALK "Archive created successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | Source file doesn't exist | Verify file paths |
| `STORAGE_FULL` | Insufficient space | Clean up storage |
| `EMPTY_ARCHIVE` | No files to compress | Check file list |
| `PERMISSION_DENIED` | Access blocked | Check permissions |

---

## Compression Options

The default compression uses standard ZIP format with deflate compression. This balances file size reduction with compatibility.

---

## Size Limits

| Limit | Default | Notes |
|-------|---------|-------|
| Max archive size | 500 MB | Configurable |
| Max files per archive | 10,000 | Practical limit |
| Max single file | 100 MB | Per file in archive |

---

## Configuration

No specific configuration required. Uses bot's standard drive settings from `config.csv`:

```csv
name,value
drive-provider,seaweedfs
drive-url,http://localhost:8333
drive-bucket,my-bot
```

---

## Implementation Notes

- Implemented in Rust under `src/file/archive.rs`
- Uses standard ZIP format for compatibility
- Preserves directory structure in archive
- Supports recursive folder compression
- Progress tracking for large archives
- Atomic operation (creates temp file, then moves)

---

## Related Keywords

- [EXTRACT](keyword-extract.md) — Extract archive contents
- [LIST](keyword-list.md) — List files to compress
- [DOWNLOAD](keyword-download.md) — Send archive to user
- [DELETE FILE](keyword-delete-file.md) — Remove files after archiving
- [COPY](keyword-copy.md) — Copy files before archiving

---

## Summary

`COMPRESS` creates ZIP archives from files and folders. Use it to bundle documents for download, create backups, package exports, and prepare email attachments. The archive preserves directory structure and can be immediately downloaded or processed. Combine with `LIST` to dynamically select files and `DOWNLOAD` to deliver archives to users.