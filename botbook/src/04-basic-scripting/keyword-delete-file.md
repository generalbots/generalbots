# DELETE FILE

> **Deprecated:** The `DELETE FILE` keyword has been unified into the [`DELETE`](keyword-delete.md) keyword. Use `DELETE` instead.

---

## Unified DELETE Keyword

The `DELETE` keyword now automatically detects file paths and handles file deletion:

```basic
' Delete a file - just use DELETE
DELETE "path/to/file.txt"

' DELETE auto-detects:
' - URLs → HTTP DELETE
' - table, filter → Database DELETE  
' - path → File DELETE
```

---

## Migration

### Old Syntax (Deprecated)

```basic
' Old way - no longer needed
DELETE FILE "temp/report.pdf"
```

### New Syntax (Recommended)

```basic
' New way - unified DELETE
DELETE "temp/report.pdf"
```

---

## Examples

```basic
' Delete a temporary file
DELETE "temp/processed.csv"

' Delete uploaded file
DELETE "uploads/" + filename

' Delete with error handling
ON ERROR RESUME NEXT
DELETE "temp/large-file.pdf"
IF ERROR THEN
    TALK "Could not delete file: " + ERROR MESSAGE
END IF
ON ERROR GOTO 0
```

---

## See Also

- [DELETE](keyword-delete.md) — Unified delete keyword (HTTP, Database, File)
- [READ](keyword-read.md) — Read file contents
- [WRITE](keyword-write.md) — Write file contents
- [COPY](keyword-copy.md) — Copy files
- [MOVE](keyword-move.md) — Move/rename files