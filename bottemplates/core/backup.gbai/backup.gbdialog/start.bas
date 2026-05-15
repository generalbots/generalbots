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
