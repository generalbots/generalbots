# MOVE

The `MOVE` keyword relocates or renames files within the bot's drive storage.

---

## Syntax

```basic
MOVE "source" TO "destination"
result = MOVE "source" TO "destination"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `source` | String | Current path of the file |
| `destination` | String | New path for the file |

---

## Description

`MOVE` transfers a file from one location to another within the bot's storage. The original file is removed after the move completes. This keyword can also be used to rename files by moving them to a new name in the same directory.

Use cases include:
- Organizing files into folders
- Renaming files
- Archiving processed files
- Moving uploads to permanent storage

---

## Examples

### Basic File Move

```basic
' Move a file to a different folder
MOVE "inbox/document.pdf" TO "processed/document.pdf"
TALK "File moved to processed folder"
```

### Rename a File

```basic
' Rename by moving to same directory with new name
MOVE "reports/report.pdf" TO "reports/sales-report-2025.pdf"
TALK "File renamed successfully"
```

### Move After Processing

```basic
' Process file then move to archive
content = READ "incoming/data.csv"
' ... process the data ...

MOVE "incoming/data.csv" TO "archive/data-" + FORMAT(NOW(), "YYYYMMDD") + ".csv"
TALK "Data processed and archived"
```

### Organize User Uploads

```basic
' Move uploaded file to user's folder
HEAR uploaded_file

temp_path = UPLOAD uploaded_file TO "temp"
permanent_path = "users/" + user.id + "/documents/" + uploaded_file.name

MOVE temp_path.path TO permanent_path
TALK "File saved to your documents"
```

### Move with Category

```basic
' Organize files by type
file_type = GET_FILE_TYPE(filename)

SWITCH file_type
    CASE "pdf"
        MOVE "uploads/" + filename TO "documents/" + filename
    CASE "jpg", "png"
        MOVE "uploads/" + filename TO "images/" + filename
    CASE "csv", "xlsx"
        MOVE "uploads/" + filename TO "data/" + filename
    CASE ELSE
        MOVE "uploads/" + filename TO "other/" + filename
END SWITCH

TALK "File organized into " + file_type + " folder"
```

### Batch Move

```basic
' Move all files from one folder to another
files = LIST "temp/"

FOR EACH file IN files
    MOVE "temp/" + file.name TO "permanent/" + file.name
NEXT

TALK "Moved " + LEN(files) + " files"
```

---

## Return Value

Returns an object with move details:

| Property | Description |
|----------|-------------|
| `result.source` | Original file path |
| `result.destination` | New file path |
| `result.size` | File size in bytes |
| `result.moved_at` | Timestamp of move operation |

---

## Error Handling

```basic
ON ERROR RESUME NEXT

MOVE "documents/report.pdf" TO "archive/report.pdf"

IF ERROR THEN
    PRINT "Move failed: " + ERROR_MESSAGE
    TALK "Sorry, I couldn't move that file."
ELSE
    TALK "File moved successfully!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | Source doesn't exist | Verify source path |
| `PERMISSION_DENIED` | Access blocked | Check permissions |
| `DESTINATION_EXISTS` | Target file exists | Delete target first or use different name |
| `SAME_PATH` | Source equals destination | Use different destination |

---

## Move vs Copy

| Operation | Source After | Use When |
|-----------|--------------|----------|
| `MOVE` | Deleted | Relocating or renaming |
| `COPY` | Preserved | Creating duplicates |

```basic
' MOVE: Original is gone
MOVE "a/file.txt" TO "b/file.txt"
' Only exists at b/file.txt now

' COPY: Original remains
COPY "a/file.txt" TO "b/file.txt"
' Exists at both locations
```

---

## Behavior Notes

- **Atomic operation**: Move completes fully or not at all
- **Creates directories**: Parent folders created automatically
- **Overwrites by default**: Destination replaced if exists
- **Cross-folder**: Can move between any directories in storage

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

- [COPY](keyword-copy.md) — Duplicate files
- [DELETE FILE](keyword-delete-file.md) — Remove files
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write file contents
- [LIST](keyword-list.md) — List directory contents
- [UPLOAD](keyword-upload.md) — Upload files to storage

---

## Summary

`MOVE` relocates or renames files within storage. The original file is removed after the move. Use it to organize files, rename documents, archive processed data, and manage user uploads. Destination directories are created automatically.