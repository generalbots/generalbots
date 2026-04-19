# EXTRACT

The `EXTRACT` keyword unpacks ZIP archives to a specified destination in the bot's storage, enabling bots to process uploaded archives and access their contents.

---

## Syntax

```basic
EXTRACT "archive.zip" TO "destination/"
result = EXTRACT "archive.zip" TO "destination/"
```

---

## Parameters

| Parameter | Type | Description |
|-----------|------|-------------|
| `archive` | String | Path to the ZIP archive to extract |
| `TO` | Clause | Destination folder for extracted contents |

---

## Description

`EXTRACT` unpacks a ZIP archive and places its contents in the specified destination folder. The folder is created automatically if it doesn't exist. Directory structure within the archive is preserved.

Use cases include:
- Processing user-uploaded archives
- Unpacking data imports
- Restoring backups
- Accessing bundled resources
- Handling bulk file uploads

---

## Examples

### Basic Extraction

```basic
' Extract archive to a folder
EXTRACT "uploads/documents.zip" TO "extracted/"

TALK "Archive extracted successfully"
```

### Extract with Result

```basic
' Get extraction details
result = EXTRACT "backup.zip" TO "restored/"

TALK "Extracted " + result.file_count + " files"
TALK "Total size: " + FORMAT(result.total_size / 1024, "#,##0") + " KB"
```

### Extract User Upload

```basic
' Handle uploaded archive from user
TALK "Please upload a ZIP file with your documents."
HEAR uploaded_file

IF uploaded_file.type = "application/zip" THEN
    upload_result = UPLOAD uploaded_file TO "temp"
    
    ' Extract to user's folder
    user_folder = "users/" + user.id + "/imports/" + FORMAT(NOW(), "YYYYMMDD") + "/"
    result = EXTRACT upload_result.path TO user_folder
    
    TALK "Extracted " + result.file_count + " files from your archive!"
    
    ' List extracted files
    files = LIST user_folder
    FOR EACH file IN files
        TALK "- " + file.name
    NEXT
ELSE
    TALK "Please upload a ZIP file."
END IF
```

### Extract and Process

```basic
' Extract data files and process them
result = EXTRACT "imports/data-batch.zip" TO "temp/batch/"

csv_files = LIST "temp/batch/" FILTER "*.csv"

FOR EACH csv_file IN csv_files
    data = READ "temp/batch/" + csv_file.name AS TABLE
    
    ' Process each row
    FOR EACH row IN data
        INSERT INTO "imports" WITH
            source_file = csv_file.name,
            data = row,
            imported_at = NOW()
    NEXT
    
    TALK "Processed: " + csv_file.name
NEXT

' Clean up temp files
DELETE FILE "temp/batch/"

TALK "Import complete: processed " + LEN(csv_files) + " files"
```

### Restore Backup

```basic
' Restore from backup archive
TALK "Enter the backup filename to restore (e.g., backup-20250115.zip)"
HEAR backup_name

backup_path = "backups/" + backup_name

files = LIST "backups/"
found = false
FOR EACH file IN files
    IF file.name = backup_name THEN
        found = true
        EXIT FOR
    END IF
NEXT

IF found THEN
    result = EXTRACT backup_path TO "restored/"
    TALK "Backup restored: " + result.file_count + " files"
ELSE
    TALK "Backup file not found. Available backups:"
    FOR EACH file IN files
        TALK "- " + file.name
    NEXT
END IF
```

---

## Common Use Cases

### Bulk Document Upload

```basic
' Handle bulk document upload
TALK "Upload a ZIP file containing your documents."
HEAR archive

upload = UPLOAD archive TO "temp"
result = EXTRACT upload.path TO "documents/bulk-" + FORMAT(NOW(), "YYYYMMDDHHmmss") + "/"

TALK "Successfully uploaded " + result.file_count + " documents!"

' Clean up temp file
DELETE FILE upload.path
```

### Process Image Pack

```basic
' Extract and catalog images
result = EXTRACT "uploads/images.zip" TO "temp/images/"

images = LIST "temp/images/" FILTER "*.jpg,*.png,*.gif"

FOR EACH image IN images
    ' Move to permanent storage with organized path
    MOVE "temp/images/" + image.name TO "media/images/" + image.name
    
    ' Record in database
    INSERT INTO "media" WITH
        filename = image.name,
        path = "media/images/" + image.name,
        size = image.size,
        uploaded_at = NOW()
NEXT

TALK "Cataloged " + LEN(images) + " images"
```

### Template Installation

```basic
' Install a template pack
result = EXTRACT "templates/new-theme.zip" TO "themes/custom/"

TALK "Template installed with " + result.file_count + " files"

' Verify required files
required = ["style.css", "config.json", "templates/"]
missing = []

FOR EACH req IN required
    files = LIST "themes/custom/" FILTER req
    IF LEN(files) = 0 THEN
        missing = APPEND(missing, req)
    END IF
NEXT

IF LEN(missing) > 0 THEN
    TALK "Warning: Missing required files: " + JOIN(missing, ", ")
ELSE
    TALK "Template is complete and ready to use!"
END IF
```

---

## Return Value

Returns an object with extraction details:

| Property | Description |
|----------|-------------|
| `result.destination` | Destination folder path |
| `result.file_count` | Number of files extracted |
| `result.folder_count` | Number of folders created |
| `result.total_size` | Total size of extracted files |
| `result.files` | Array of extracted file paths |
| `result.extracted_at` | Extraction timestamp |

---

## Error Handling

```basic
ON ERROR RESUME NEXT

result = EXTRACT "uploads/data.zip" TO "extracted/"

IF ERROR THEN
    PRINT "Extraction failed: " + ERROR_MESSAGE
    
    IF INSTR(ERROR_MESSAGE, "corrupt") > 0 THEN
        TALK "The archive appears to be corrupted. Please upload again."
    ELSE IF INSTR(ERROR_MESSAGE, "not found") > 0 THEN
        TALK "Archive file not found."
    ELSE IF INSTR(ERROR_MESSAGE, "storage") > 0 THEN
        TALK "Not enough storage space to extract the archive."
    ELSE
        TALK "Sorry, I couldn't extract the archive. Please try again."
    END IF
ELSE
    TALK "Extraction complete!"
END IF
```

### Common Errors

| Error | Cause | Solution |
|-------|-------|----------|
| `FILE_NOT_FOUND` | Archive doesn't exist | Verify archive path |
| `INVALID_ARCHIVE` | Not a valid ZIP file | Check file format |
| `CORRUPT_ARCHIVE` | Archive is damaged | Request new upload |
| `STORAGE_FULL` | Insufficient space | Clean up storage |
| `PERMISSION_DENIED` | Access blocked | Check permissions |

---

## Security Considerations

- **Path validation**: Extracted paths are validated to prevent directory traversal attacks
- **Size limits**: Maximum extracted size is enforced to prevent storage exhaustion
- **File type filtering**: Executable files can be blocked if configured
- **Malware scanning**: Uploaded archives can be scanned before extraction

---

## Size Limits

| Limit | Default | Notes |
|-------|---------|-------|
| Max archive size | 100 MB | For uploaded archives |
| Max extracted size | 500 MB | Total after extraction |
| Max files | 10,000 | Files in archive |
| Max path depth | 10 | Nested folder depth |

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
- Supports standard ZIP format
- Preserves directory structure
- Handles nested folders
- Progress tracking for large archives
- Atomic extraction (temp folder, then move)
- Cleans up on failure

---

## Related Keywords

- [COMPRESS](keyword-compress.md) — Create ZIP archives
- [UPLOAD](keyword-upload.md) — Upload archives from users
- [LIST](keyword-list.md) — List extracted files
- [MOVE](keyword-move.md) — Organize extracted files
- [DELETE FILE](keyword-delete-file.md) — Clean up after extraction

---

## Summary

`EXTRACT` unpacks ZIP archives to a destination folder. Use it to process uploaded archives, restore backups, handle bulk imports, and access bundled resources. The archive's directory structure is preserved, and the destination folder is created automatically. Combine with `UPLOAD` to accept user archives and `LIST` to discover extracted contents.