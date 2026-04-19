# Backup Template

The backup template provides automated file archiving and restoration capabilities, helping you protect important data with scheduled backups and easy recovery options.

## Topic: File Backup & Recovery

This template is perfect for:
- Automated data protection
- Scheduled file archiving
- Disaster recovery preparation
- Compliance with data retention policies

## The Code

```basic
ADD TOOL "backup-to-server"
ADD TOOL "restore-file"
ADD TOOL "list-archived"
ADD TOOL "cleanup-old"

CLEAR SUGGESTIONS

ADD SUGGESTION "backup" AS "Run backup now"
ADD SUGGESTION "list" AS "View archived files"
ADD SUGGESTION "restore" AS "Restore a file"
ADD SUGGESTION "status" AS "Backup status"

SET CONTEXT "backup" AS "You are a backup management assistant. Help users archive files to server storage, restore archived files, and manage backup schedules."

BEGIN TALK
**Backup Manager**

I can help you with:
• Archive files to server storage
• Restore archived files
• View backup history
• Manage backup schedules

Select an option or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are a backup management assistant.

Archive files older than specified days to server storage.
Track all backup operations in log.xlsx.
Support restore operations from archived files.
Maintain MD5 checksums for integrity verification.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the backup template works in real-world scenarios.

### Dialog 1: Running a Backup

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💾</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Backup Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Backup Manager</div>
      <p><strong>Backup Manager</strong></p>
      <p>I can help you with:</p>
      <p>• Archive files to server storage</p>
      <p>• Restore archived files</p>
      <p>• View backup history</p>
      <p>• Manage backup schedules</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Run backup now</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Starting backup process... 🔄</p>
      <p>Scanning files older than 30 days...</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Backup completed successfully!</p>
      <p>📁 <strong>Files archived:</strong> 47</p>
      <p>💾 <strong>Total size:</strong> 128.5 MB</p>
      <p>🔐 <strong>Checksums:</strong> Verified</p>
      <p>📋 <strong>Log:</strong> Updated in log.xlsx</p>
      <div class="wa-time">09:03</div>
    </div>
  </div>
</div>

### Dialog 2: Viewing Archived Files

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💾</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Backup Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>View archived files</p>
      <div class="wa-time">14:20 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📦 <strong>Recent Archives:</strong></p>
      <p></p>
      <p>1. <code>backup-2025-01-15.zip</code> (45.2 MB)</p>
      <p>   📅 Jan 15, 2025 • 32 files</p>
      <p></p>
      <p>2. <code>backup-2025-01-08.zip</code> (38.7 MB)</p>
      <p>   📅 Jan 8, 2025 • 28 files</p>
      <p></p>
      <p>3. <code>backup-2025-01-01.zip</code> (52.1 MB)</p>
      <p>   📅 Jan 1, 2025 • 41 files</p>
      <p></p>
      <p>Would you like to restore any of these?</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>
</div>

### Dialog 3: Restoring a File

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💾</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Backup Manager</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I need to restore a file</p>
      <div class="wa-time">16:45 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Sure! What file are you looking for? You can provide:</p>
      <p>• File name (e.g., "report.xlsx")</p>
      <p>• Partial name (e.g., "report")</p>
      <p>• Date range (e.g., "files from last week")</p>
      <div class="wa-time">16:45</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>quarterly-report.xlsx</p>
      <div class="wa-time">16:46 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔍 Found 2 versions of "quarterly-report.xlsx":</p>
      <p></p>
      <p>1. From Jan 15, 2025 (1.2 MB)</p>
      <p>2. From Jan 1, 2025 (1.1 MB)</p>
      <p></p>
      <p>Which version would you like to restore?</p>
      <div class="wa-time">16:46</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>The one from January 15</p>
      <div class="wa-time">16:46 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ File restored successfully!</p>
      <p>📄 <strong>File:</strong> quarterly-report.xlsx</p>
      <p>📅 <strong>Version:</strong> Jan 15, 2025</p>
      <p>📍 <strong>Location:</strong> /documents/restored/</p>
      <p>🔐 <strong>Checksum:</strong> Verified ✓</p>
      <div class="wa-time">16:47</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register backup tools for AI to use |
| `ADD SUGGESTION` | Create quick action buttons |
| `SET CONTEXT` | Define the bot's role and capabilities |
| `BEGIN TALK` | Welcome message block |
| `BEGIN SYSTEM PROMPT` | AI behavior instructions |

## Backup Tool: backup-to-server.bas

```basic
PARAM folder AS STRING LIKE "documents" DESCRIPTION "Folder to backup"
PARAM days AS INTEGER LIKE 30 DESCRIPTION "Archive files older than X days"

DESCRIPTION "Archive files older than specified days to server storage"

IF NOT folder THEN
    folder = "documents"
END IF

IF NOT days THEN
    days = 30
END IF

' Calculate cutoff date
cutoff = DATEADD(NOW(), -days, "days")

' Find files to archive
files = FIND folder, "modified < '" + FORMAT(cutoff, "YYYY-MM-DD") + "'"

IF UBOUND(files) = 0 THEN
    TALK "No files found older than " + days + " days."
    RETURN 0
END IF

' Create archive name
archiveName = "backup-" + FORMAT(NOW(), "YYYY-MM-DD") + ".zip"

' Compress files
COMPRESS files, archiveName

' Calculate checksums
FOR EACH file IN files
    checksum = MD5(file)
    
    WITH logEntry
        timestamp = NOW()
        filename = file.name
        size = file.size
        md5 = checksum
        archive = archiveName
        status = "archived"
    END WITH
    
    SAVE "log.xlsx", logEntry
NEXT

' Move to server storage
MOVE archiveName, "server://backups/" + archiveName

TALK "✅ Backup completed: " + UBOUND(files) + " files archived to " + archiveName

RETURN UBOUND(files)
```

## Restore Tool: restore-file.bas

```basic
PARAM filename AS STRING LIKE "report.xlsx" DESCRIPTION "Name of file to restore"
PARAM date AS STRING LIKE "2025-01-15" DESCRIPTION "Backup date to restore from" OPTIONAL

DESCRIPTION "Restore a file from archived backups"

' Search for file in backup logs
IF date THEN
    results = FIND "log.xlsx", "filename LIKE '%" + filename + "%' AND archive LIKE '%" + date + "%'"
ELSE
    results = FIND "log.xlsx", "filename LIKE '%" + filename + "%'"
END IF

IF UBOUND(results) = 0 THEN
    TALK "No archived files found matching '" + filename + "'"
    RETURN NULL
END IF

IF UBOUND(results) > 1 AND NOT date THEN
    TALK "Found " + UBOUND(results) + " versions. Please specify which date:"
    FOR EACH result IN results
        TALK "• " + result.archive + " (" + FORMAT(result.timestamp, "MMM DD, YYYY") + ")"
    NEXT
    RETURN results
END IF

' Get the archive
archive = results[1].archive
originalChecksum = results[1].md5

' Download from server
DOWNLOAD "server://backups/" + archive, archive

' Extract the specific file
EXTRACT archive, filename, "restored/"

' Verify checksum
restoredChecksum = MD5("restored/" + filename)

IF restoredChecksum = originalChecksum THEN
    TALK "✅ File restored and verified: restored/" + filename
ELSE
    TALK "⚠️ Warning: Checksum mismatch. File may be corrupted."
END IF

' Log restoration
WITH logEntry
    timestamp = NOW()
    action = "restore"
    filename = filename
    archive = archive
    verified = (restoredChecksum = originalChecksum)
END WITH

SAVE "log.xlsx", logEntry

RETURN "restored/" + filename
```

## How It Works

1. **Tool Registration**: `ADD TOOL` makes backup functions available to the AI
2. **Quick Actions**: `ADD SUGGESTION` creates one-tap backup options
3. **Context Setting**: Defines the bot as a backup management assistant
4. **File Scanning**: Finds files matching age criteria
5. **Compression**: Creates ZIP archives with checksums
6. **Logging**: Tracks all operations in log.xlsx
7. **Restoration**: Extracts files and verifies integrity

## Scheduling Backups

Set up automated backups with scheduled jobs:

```basic
PARAM jobname AS STRING DESCRIPTION "Name of the backup job"

IF jobname = "daily backup" THEN
    SET SCHEDULE "0 2 * * *"  ' Run at 2 AM daily
    
    ' Backup documents folder
    CALL backup-to-server("documents", 7)
    
    ' Backup reports folder
    CALL backup-to-server("reports", 30)
    
    ' Send confirmation
    SEND MAIL "admin@company.com", "Daily Backup Complete", "Backup completed at " + NOW()
END IF

IF jobname = "weekly cleanup" THEN
    SET SCHEDULE "0 3 * * 0"  ' Run at 3 AM on Sundays
    
    ' Remove backups older than 90 days
    CALL cleanup-old(90)
    
    SEND MAIL "admin@company.com", "Weekly Cleanup Complete", "Old backups removed"
END IF
```

## Customization Ideas

### Add Email Notifications

```basic
' After backup completes
SEND MAIL "admin@company.com", "Backup Report", 
    "Files archived: " + fileCount + "\n" +
    "Total size: " + totalSize + " MB\n" +
    "Archive: " + archiveName
```

### Add Backup Verification

```basic
' Verify backup integrity
FOR EACH entry IN FIND("log.xlsx", "archive = '" + archiveName + "'")
    originalFile = GET entry.filename
    archivedChecksum = entry.md5
    
    IF MD5(originalFile) <> archivedChecksum THEN
        TALK "⚠️ Warning: " + entry.filename + " has changed since backup"
    END IF
NEXT
```

### Add Storage Monitoring

```basic
' Check available storage
storageUsed = FOLDER_SIZE("server://backups/")
storageLimit = 10000  ' 10 GB in MB

IF storageUsed > storageLimit * 0.9 THEN
    TALK "⚠️ Storage is 90% full. Consider cleaning old backups."
    SEND MAIL "admin@company.com", "Storage Warning", "Backup storage is almost full"
END IF
```

## Related Templates

- [start.bas](./start.md) - Basic greeting flow
- [analytics-dashboard.bas](./analytics-dashboard.md) - Monitor system metrics
- [broadcast.bas](./broadcast.md) - Send notifications to teams

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-system{text-align:center;margin:15px 0;clear:both}
.wa-system span{background-color:#e1f2fb;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>