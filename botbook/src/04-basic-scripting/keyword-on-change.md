# ON CHANGE

Monitors folders for file changes and triggers a script when files are created, modified, or deleted.

## Syntax

```basic
' Using account:// syntax (recommended)
ON CHANGE "account://email@domain.com/path/to/folder"

' Direct provider syntax
ON CHANGE "gdrive:///path/to/folder"
ON CHANGE "onedrive:///path/to/folder"
ON CHANGE "dropbox:///path/to/folder"

' Local filesystem
ON CHANGE "/local/path/to/folder"

' With event type filter
ON CHANGE "account://email@domain.com/folder" EVENTS "create, modify"
```

## Description

The `ON CHANGE` keyword registers a folder monitor that triggers a script whenever files change in the specified folder. This works with cloud storage providers (Google Drive, OneDrive, Dropbox) and local filesystem.

Uses the same `account://` syntax as `COPY`, `MOVE`, and other file operations for consistency.

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| path | String | Yes | Folder path using account:// or provider:// syntax |
| EVENTS | String | No | Comma-separated list of event types to monitor |

## Path Syntax

| Format | Description | Example |
|--------|-------------|---------|
| `account://email/path` | Uses connected account (auto-detects provider) | `account://user@gmail.com/Documents` |
| `gdrive:///path` | Google Drive direct | `gdrive:///shared/reports` |
| `onedrive:///path` | OneDrive direct | `onedrive:///business/docs` |
| `dropbox:///path` | Dropbox direct | `dropbox:///team/assets` |
| `/path` | Local filesystem | `/var/uploads/incoming` |

## Provider Auto-Detection

When using `account://` syntax, the provider is auto-detected from the email:

| Email Domain | Provider |
|--------------|----------|
| `@gmail.com`, `@google.com` | Google Drive |
| `@outlook.com`, `@hotmail.com`, `@live.com` | OneDrive |
| Other | Defaults to Google Drive |

## Event Types

| Event | Aliases | Description |
|-------|---------|-------------|
| `create` | `created`, `new` | New file created |
| `modify` | `modified`, `change`, `changed` | File content changed |
| `delete` | `deleted`, `remove`, `removed` | File deleted |
| `rename` | `renamed` | File renamed |
| `move` | `moved` | File moved to different folder |

Default events (if not specified): `create`, `modify`, `delete`

## Examples

### Basic Folder Monitoring

```basic
' Monitor Google Drive folder via connected account
ON CHANGE "account://user@gmail.com/Documents/invoices"
    event = GET LAST "folder_change_events"
    TALK "File changed: " + event.file_name + " (" + event.event_type + ")"
END ON
```

### Process New Uploads

```basic
ON CHANGE "account://user@gmail.com/Uploads" EVENTS "create"
    event = GET LAST "folder_change_events"
    
    ' Only process PDF files
    IF event.mime_type = "application/pdf" THEN
        ' Extract data from invoice
        data = SAVE FROM UNSTRUCTURED event.file_path, "invoices"
        TALK "Processed invoice: " + data.invoice_number
        
        ' Move to processed folder
        MOVE event.file_path TO "account://user@gmail.com/Processed/"
    END IF
END ON
```

### Sync Between Providers

```basic
' When file is added to Google Drive, copy to OneDrive
ON CHANGE "account://user@gmail.com/Shared/Reports" EVENTS "create"
    event = GET LAST "folder_change_events"
    
    ' Copy to OneDrive backup
    COPY event.file_path TO "account://user@outlook.com/Backup/Reports/"
    
    TALK "Synced " + event.file_name + " to OneDrive"
END ON
```

### Monitor for Deletions

```basic
ON CHANGE "gdrive:///archive" EVENTS "delete"
    event = GET LAST "folder_change_events"
    
    ' Log deletion to audit table
    INSERT INTO "audit_log", {
        "action": "file_deleted",
        "file_path": event.file_path,
        "file_name": event.file_name,
        "deleted_at": NOW()
    }
    
    ' Send notification
    SEND MAIL "admin@company.com", "File Deleted", "
        File was deleted from archive:
        Name: " + event.file_name + "
        Path: " + event.file_path + "
    "
END ON
```

### Watch for Modifications

```basic
ON CHANGE "account://user@gmail.com/Config" EVENTS "modify"
    event = GET LAST "folder_change_events"
    
    ' Reload configuration when config files change
    IF event.file_name = "settings.json" THEN
        config = READ event.file_path
        SET BOT MEMORY "config", config
        TALK "Configuration reloaded"
    END IF
END ON
```

### Process Images

```basic
ON CHANGE "account://user@gmail.com/Photos/Raw" EVENTS "create"
    event = GET LAST "folder_change_events"
    
    ' Check if it's an image
    IF INSTR(event.mime_type, "image/") > 0 THEN
        ' Generate thumbnail using LLM vision
        description = IMAGE event.file_path
        
        ' Save metadata
        INSERT INTO "photos", {
            "file_path": event.file_path,
            "description": description,
            "size": event.file_size,
            "uploaded_at": NOW()
        }
        
        TALK "Processed image: " + event.file_name
    END IF
END ON
```

### Multi-Folder Monitoring

```basic
' Monitor multiple folders with different handlers

ON CHANGE "account://user@gmail.com/Invoices"
    event = GET LAST "folder_change_events"
    ' Process invoices
    CREATE TASK "Process invoice: " + event.file_name, event.file_path, "normal"
END ON

ON CHANGE "account://user@gmail.com/Contracts"
    event = GET LAST "folder_change_events"
    ' Process contracts
    SEND MAIL "legal@company.com", "New Contract", "Please review: " + event.file_name
END ON

ON CHANGE "account://user@outlook.com/Reports" EVENTS "create, modify"
    event = GET LAST "folder_change_events"
    ' Sync reports
    COPY event.file_path TO "gdrive:///Shared/Reports/"
END ON
```

### Local Filesystem Monitoring

```basic
' Monitor local upload directory
ON CHANGE "/var/www/uploads/incoming"
    event = GET LAST "folder_change_events"
    
    ' Scan for viruses
    result = RUN BASH "clamscan " + event.file_path
    
    IF INSTR(result, "FOUND") > 0 THEN
        ' Quarantine infected file
        MOVE event.file_path TO "/var/quarantine/"
        SEND MAIL "security@company.com", "Virus Detected", event.file_name
    ELSE
        ' Move to processed
        MOVE event.file_path TO "/var/www/uploads/processed/"
    END IF
END ON
```

## Change Event Properties

When a file change is detected, the event object contains:

| Property | Type | Description |
|----------|------|-------------|
| `id` | UUID | Unique event identifier |
| `monitor_id` | UUID | ID of the monitor that triggered |
| `event_type` | String | Type of change (create, modify, delete, rename, move) |
| `file_path` | String | Full path to the file |
| `file_id` | String | Provider-specific file ID |
| `file_name` | String | File name without path |
| `file_size` | Integer | Size in bytes |
| `mime_type` | String | MIME type of the file |
| `old_path` | String | Previous path (for rename/move events) |

## Database Tables

### folder_monitors

Stores the monitor configuration:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Monitor ID |
| `bot_id` | UUID | Bot that owns this monitor |
| `provider` | VARCHAR | Provider (gdrive, onedrive, dropbox, local) |
| `account_email` | VARCHAR | Email from account:// path |
| `folder_path` | VARCHAR | Path being monitored |
| `folder_id` | VARCHAR | Provider-specific folder ID |
| `script_path` | VARCHAR | Script to execute |
| `is_active` | BOOLEAN | Whether monitor is active |
| `watch_subfolders` | BOOLEAN | Include subfolders |
| `event_types_json` | TEXT | JSON array of event types |
| `last_change_token` | VARCHAR | Provider change token |

### folder_change_events

Logs detected changes:

| Column | Type | Description |
|--------|------|-------------|
| `id` | UUID | Event ID |
| `monitor_id` | UUID | Monitor that triggered |
| `event_type` | VARCHAR | Type of change |
| `file_path` | VARCHAR | File path |
| `file_name` | VARCHAR | File name |
| `file_size` | BIGINT | Size in bytes |
| `mime_type` | VARCHAR | MIME type |
| `processed` | BOOLEAN | Whether event was processed |

## Best Practices

1. **Use account:// syntax** - Consistent with other file operations
2. **Filter events** - Only monitor events you need to reduce load
3. **Handle all event types** - Check `event_type` before processing
4. **Avoid circular triggers** - Moving files can trigger other monitors
5. **Process idempotently** - Events may be delivered more than once
6. **Clean up processed events** - Archive old events periodically

## Comparison with Other Event Keywords

| Keyword | Trigger Source | Use Case |
|---------|---------------|----------|
| `ON CHANGE` | File system changes | Sync files, process uploads |
| `ON EMAIL` | Incoming emails | Process messages, auto-reply |
| `ON INSERT` | Database inserts | React to new data |
| `SET SCHEDULE` | Time-based | Periodic tasks |
| `WEBHOOK` | HTTP requests | External integrations |

## Related Keywords

- [ON EMAIL](./keyword-on-email.md) - Email monitoring
- [ON](./keyword-on.md) - Database trigger events
- [SET SCHEDULE](./keyword-set-schedule.md) - Time-based scheduling
- [COPY](./keyword-copy.md) - Copy files with account:// syntax
- [MOVE](./keyword-move.md) - Move files with account:// syntax
- [USE ACCOUNT](./keyword-use-account.md) - Connect cloud accounts

---

**TriggerKind:** `FolderChange = 6`
