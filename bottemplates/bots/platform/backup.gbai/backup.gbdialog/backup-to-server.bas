PARAM folder AS STRING LIKE "default.gbdrive" DESCRIPTION "Folder to backup files from" OPTIONAL
PARAM days_old AS INTEGER LIKE 3 DESCRIPTION "Archive files older than this many days" OPTIONAL

DESCRIPTION "Backup and archive expired files to server storage"

IF NOT folder THEN
    folder = "default.gbdrive"
END IF

IF NOT days_old THEN
    days_old = 3
END IF

list = DIR folder
archived = 0

FOR EACH item IN list
    oldDays = DATEDIFF today, item.modified, "day"

    IF oldDays > days_old THEN
        blob = UPLOAD item

        WITH logEntry
            action = "archived"
            date = today
            time = now
            path = item.path
            name = item.name
            size = item.size
            modified = item.modified
            md5 = blob.md5
        END WITH

        SAVE "log.xlsx", logEntry
        DELETE item
        archived = archived + 1
    END IF
NEXT

TALK "Backup complete. " + archived + " files archived."

RETURN archived
