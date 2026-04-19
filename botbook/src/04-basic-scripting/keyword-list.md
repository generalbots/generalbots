# LIST

The `LIST` keyword retrieves a directory listing from the bot's drive storage, returning information about files and subdirectories.

---

## Syntax

```basic
files = LIST "path/"
files = LIST "path/" FILTER "*.pdf"
files = LIST "path/" RECURSIVE
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `path` | String | Directory path to list (must end with `/`) |
| `FILTER` | String | Optional glob pattern to filter results |
| `RECURSIVE` | Flag | Include files in subdirectories |

---

## Description

`LIST` returns an array of file and directory information from the specified path in the bot's storage. Each item in the result includes metadata such as name, size, type, and modification date.

Use cases include:
- Browsing user uploads
- Finding files matching patterns
- Checking if files exist
- Building file inventories
- Processing batches of files

---

## Examples

### Basic Directory Listing

```basic
' List all files in a directory
files = LIST "documents/"

FOR EACH file IN files
    TALK file.name + " (" + file.size + " bytes)"
NEXT
```

### Filter by Extension

```basic
' List only PDF files
pdfs = LIST "documents/" FILTER "*.pdf"

TALK "Found " + LEN(pdfs) + " PDF files"

FOR EACH pdf IN pdfs
    TALK "- " + pdf.name
NEXT
```

### Recursive Listing

```basic
' List all files including subdirectories
all_files = LIST "uploads/" RECURSIVE

TALK "Total files: " + LEN(all_files)
```

### Check File Exists

```basic
' Check if a specific file exists
files = LIST "reports/"

found = false
FOR EACH file IN files
    IF file.name = "monthly-report.pdf" THEN
        found = true
        EXIT FOR
    END IF
NEXT

IF found THEN
    TALK "Report found!"
ELSE
    TALK "Report not found. Would you like me to generate one?"
END IF
```

### Find Recent Files

```basic
' List files modified in last 24 hours
files = LIST "inbox/"
yesterday = DATEADD(NOW(), -1, "day")

recent = FILTER files WHERE modified > yesterday

TALK "You have " + LEN(recent) + " new files since yesterday"
```

### Calculate Folder Size

```basic
' Sum up total size of files in folder
files = LIST "backups/" RECURSIVE

total_size = 0
FOR EACH file IN files
    total_size = total_size + file.size
NEXT

size_mb = total_size / 1048576
TALK "Backup folder size: " + FORMAT(size_mb, "#,##0.00") + " MB"
```

### Process All Files of Type

```basic
' Process all CSV files in a folder
csv_files = LIST "imports/" FILTER "*.csv"

FOR EACH csv_file IN csv_files
    data = READ "imports/" + csv_file.name AS TABLE
    ' Process each file...
    MOVE "imports/" + csv_file.name TO "processed/" + csv_file.name
NEXT

TALK "Processed " + LEN(csv_files) + " CSV files"
```

---

## Return Value

Returns an array of file objects. Each object contains:

| Property | Type | Description |
|----------|------|-------------|
| `name` | String | File or directory name |
| `path` | String | Full path relative to storage root |
| `size` | Number | File size in bytes (0 for directories) |
| `type` | String | `file` or `directory` |
| `mime_type` | String | MIME type (e.g., `application/pdf`) |
| `modified` | DateTime | Last modification timestamp |
| `created` | DateTime | Creation timestamp |

### Example Result

```basic
files = LIST "documents/"

' files[0] might be:
' {
'   name: "report.pdf",
'   path: "documents/report.pdf",
'   size: 245678,
'   type: "file",
'   mime_type: "application/pdf",
'   modified: "2025-01-15T10:30:00Z",
'   created: "2025-01-10T09:00:00Z"
' }
```

---

## Filter Patterns

| Pattern | Matches |
|---------|---------|
| `*` | All files |
| `*.pdf` | All PDF files |
| `*.csv` | All CSV files |
| `report*` | Files starting with "report" |
| `*2025*` | Files containing "2025" |
| `*.jpg,*.png` | Multiple extensions |

```basic
' Multiple extensions
images = LIST "photos/" FILTER "*.jpg,*.png,*.gif"

' Wildcard in name
reports = LIST "exports/" FILTER "sales-*"
```

---

## Error Handling

```basic
ON ERROR RESUME NEXT

files = LIST "nonexistent-folder/"

IF ERROR THEN
    PRINT "List failed: " + ERROR_MESSAGE
    TALK "That folder doesn't exist."
ELSE IF LEN(files) = 0 THEN
    TALK "The folder is empty."
ELSE
    TALK "Found " + LEN(files) + " items"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `PATH_NOT_FOUND` | Directory doesn't exist | Check path spelling |
| `NOT_A_DIRECTORY` | Path is a file, not folder | Add trailing `/` |
| `PERMISSION_DENIED` | Access blocked | Check permissions |

---

## Behavior Notes

- **Trailing slash required**: Paths must end with `/` to indicate directory
- **Excludes hidden files**: Files starting with `.` are excluded by default
- **Sorted alphabetically**: Results are sorted by name
- **Non-recursive by default**: Only lists immediate contents unless `RECURSIVE` specified

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

## Related Keywords

- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write file contents
- [COPY](keyword-copy.md) — Copy files
- [MOVE](keyword-move.md) — Move or rename files
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [UPLOAD](keyword-upload.md) — Upload files to storage

---

## Summary

`LIST` retrieves directory contents from storage, returning detailed metadata about each file and subdirectory. Use it to browse files, find matching documents, check existence, calculate sizes, and process batches of files. Filter patterns and recursive options help narrow results to exactly what you need.