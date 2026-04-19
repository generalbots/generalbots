# COPY

The `COPY` keyword duplicates files within the bot's drive storage, creating copies in the same or different directories.

---

## Syntax

```basic
COPY "source" TO "destination"
result = COPY "source" TO "destination"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source` | String | Path to the file to copy |
| `destination` | String | Path for the new copy |

---

## Description

`COPY` creates a duplicate of a file in the bot's storage. The original file remains unchanged. If the destination directory doesn't exist, it's created automatically.

Use cases include:
- Creating backups before modifications
- Duplicating templates for new users
- Archiving files while keeping originals accessible
- Organizing files into multiple locations

---

## Examples

### Basic File Copy

```basic
' Copy a file to a new location
COPY "templates/report.docx" TO "user-reports/report-copy.docx"
TALK "File copied successfully!"
```

### Copy with Same Name

```basic
' Copy to different directory, keeping the same filename
COPY "documents/contract.pdf" TO "archive/contract.pdf"
```

### Copy Before Editing

```basic
' Create backup before modifying
COPY "config/settings.json" TO "config/settings.json.backup"

' Now safe to modify original
content = READ "config/settings.json"
modified = REPLACE(content, "old_value", "new_value")
WRITE modified TO "config/settings.json"

TALK "Settings updated. Backup saved."
```

### Copy Template for User

```basic
' Create user-specific copy of template
user_folder = "users/" + user.id
COPY "templates/welcome-kit.pdf" TO user_folder + "/welcome-kit.pdf"
TALK "Your welcome kit is ready!"
```

### Copy with Timestamp

```basic
' Create timestamped copy
timestamp = FORMAT(NOW(), "YYYYMMDD-HHmmss")
COPY "reports/daily.csv" TO "archive/daily-" + timestamp + ".csv"
TALK "Report archived with timestamp"
```

### Batch Copy

```basic
' Copy multiple files
files_to_copy = ["doc1.pdf", "doc2.pdf", "doc3.pdf"]

FOR EACH file IN files_to_copy
    COPY "source/" + file TO "destination/" + file
NEXT

TALK "Copied " + LEN(files_to_copy) + " files"
```

---

## Return Value

Returns an object with copy details:

| Property | Description |
|----------|-------------|
| `result.source` | Original file path |
| `result.destination` | New file path |
| `result.size` | File size in bytes |
| `result.copied_at` | Timestamp of copy operation |

---

## Error Handling

```basic
ON ERROR RESUME NEXT

COPY "documents/important.pdf" TO "backup/important.pdf"

IF ERROR THEN
    PRINT "Copy failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't copy that file."
ELSE
    TALK "File copied successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | Source doesn't exist | Verify source path |
| `PERMISSION_DENIED` | Access blocked | Check permissions |
| `DESTINATION_EXISTS` | File already exists | Use different name or delete first |
| `STORAGE_FULL` | No space available | Clean up storage |

---

## Behavior Notes

- **Overwrites by default**: If destination exists, it's replaced
- **Creates directories**: Parent folders created automatically
- **Preserves metadata**: File type and creation date preserved
- **Atomic operation**: Copy completes fully or not at all

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

- [MOVE](keyword-move.md) — Move or rename files
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write file contents
- [LIST](keyword-list.md) — List directory contents

---

## Summary

`COPY` creates duplicates of files in storage. Use it for backups, templates, archiving, and organizing files. The original file is preserved, and destination directories are created automatically.