REM Vector Database Statistics Dialog
REM Provides knowledge base statistics and management for administrators
REM Can be used in admin bots or regular .gbdialog files

DESCRIPTION "Knowledge base statistics and vector database management"

REM Get overall KB statistics
stats = KB STATISTICS
statsObj = JSON PARSE stats

TALK "📊 **Knowledge Base Statistics**"
TALK ""
TALK "**Collections:** " + statsObj.total_collections
TALK "**Total Documents:** " + FORMAT(statsObj.total_documents, "#,##0")
TALK "**Total Vectors:** " + FORMAT(statsObj.total_vectors, "#,##0")
TALK "**Disk Usage:** " + FORMAT(statsObj.total_disk_size_mb, "#,##0.00") + " MB"
TALK "**RAM Usage:** " + FORMAT(statsObj.total_ram_size_mb, "#,##0.00") + " MB"
TALK ""
TALK "📅 **Recent Activity**"
TALK "Documents added last 7 days: " + FORMAT(statsObj.documents_added_last_week, "#,##0")
TALK "Documents added last 30 days: " + FORMAT(statsObj.documents_added_last_month, "#,##0")

REM Show collection details
ADD SUGGESTION "View Collections"
ADD SUGGESTION "Check Storage"
ADD SUGGESTION "Recent Documents"
ADD SUGGESTION "Exit"

HEAR choice AS MENU "View Collections", "Check Storage", "Recent Documents", "Exit"

SELECT CASE choice
    CASE "View Collections"
        collections = KB LIST COLLECTIONS

        IF LEN(collections) = 0 THEN
            TALK "No collections found for this bot."
        ELSE
            TALK "📁 **Your Collections:**"
            TALK ""

            FOR EACH collection IN collections
                collectionStats = KB COLLECTION STATS collection
                collObj = JSON PARSE collectionStats

                TALK "**" + collObj.name + "**"
                TALK "  • Documents: " + FORMAT(collObj.points_count, "#,##0")
                TALK "  • Vectors: " + FORMAT(collObj.vectors_count, "#,##0")
                TALK "  • Status: " + collObj.status
                TALK "  • Disk: " + FORMAT(collObj.disk_data_size / 1048576, "#,##0.00") + " MB"
                TALK ""
            NEXT
        END IF

    CASE "Check Storage"
        storageSize = KB STORAGE SIZE
        documentsCount = KB DOCUMENTS COUNT

        TALK "💾 **Storage Overview**"
        TALK ""
        TALK "Total storage used: " + FORMAT(storageSize, "#,##0.00") + " MB"
        TALK "Total documents indexed: " + FORMAT(documentsCount, "#,##0")

        IF documentsCount > 0 THEN
            avgSize = storageSize / documentsCount
            TALK "Average per document: " + FORMAT(avgSize * 1024, "#,##0.00") + " KB"
        END IF

    CASE "Recent Documents"
        lastWeek = KB DOCUMENTS ADDED SINCE 7
        lastMonth = KB DOCUMENTS ADDED SINCE 30
        lastDay = KB DOCUMENTS ADDED SINCE 1

        TALK "📈 **Document Activity**"
        TALK ""
        TALK "Added in last 24 hours: " + FORMAT(lastDay, "#,##0")
        TALK "Added in last 7 days: " + FORMAT(lastWeek, "#,##0")
        TALK "Added in last 30 days: " + FORMAT(lastMonth, "#,##0")

        IF lastWeek > 0 THEN
            dailyAvg = lastWeek / 7
            TALK ""
            TALK "Daily average (7 days): " + FORMAT(dailyAvg, "#,##0.0") + " documents"
        END IF

    CASE "Exit"
        TALK "Thank you for using KB Statistics. Goodbye!"

END SELECT

REM Store statistics in bot memory for dashboard
SET BOT MEMORY "kb_last_check", NOW()
SET BOT MEMORY "kb_total_docs", statsObj.total_documents
SET BOT MEMORY "kb_storage_mb", statsObj.total_disk_size_mb
