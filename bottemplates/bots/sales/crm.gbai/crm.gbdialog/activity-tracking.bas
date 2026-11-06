' =============================================================================
' Activity Tracking Dialog - CRM Template
' Microsoft Dynamics CRM-style Activity Management
' =============================================================================
' This dialog handles logging and tracking of customer activities:
' - Phone Calls
' - Emails
' - Meetings/Appointments
' - Tasks
' - Notes
' =============================================================================

TALK "Activity Tracking - Log and manage customer interactions"

HEAR activity_type AS TEXT WITH "Select activity type: (call, email, meeting, task, note)"

SELECT CASE LCASE(activity_type)
    CASE "call", "phone", "1"
        CALL "log-call.bas"

    CASE "email", "2"
        CALL "log-email-activity.bas"

    CASE "meeting", "appointment", "3"
        CALL "log-meeting.bas"

    CASE "task", "4"
        CALL "log-task.bas"

    CASE "note", "5"
        CALL "log-note.bas"

    CASE ELSE
        TALK "I will help you log a general activity."

        HEAR regarding_type AS TEXT WITH "What does this activity relate to? (lead, contact, account, opportunity)"
        HEAR regarding_id AS TEXT WITH "Enter the record ID or name:"

        SELECT CASE LCASE(regarding_type)
            CASE "lead"
                record = FIND "leads" WHERE id = regarding_id OR name LIKE regarding_id FIRST
            CASE "contact"
                record = FIND "contacts" WHERE id = regarding_id OR full_name LIKE regarding_id FIRST
            CASE "account"
                record = FIND "accounts" WHERE id = regarding_id OR name LIKE regarding_id FIRST
            CASE "opportunity"
                record = FIND "opportunities" WHERE id = regarding_id OR name LIKE regarding_id FIRST
        END SELECT

        IF record IS NULL THEN
            TALK "Record not found. Please verify the ID or name."
            EXIT
        END IF

        TALK "Record found: " + record.name

        HEAR subject AS TEXT WITH "Activity subject:"
        HEAR description AS TEXT WITH "Activity description (details of the interaction):"
        HEAR duration AS INTEGER WITH "Duration in minutes:" DEFAULT 30
        HEAR outcome AS TEXT WITH "Outcome (completed, pending, cancelled):" DEFAULT "completed"

        activity_id = "ACT-" + FORMAT(NOW(), "YYYYMMDDHHmmss") + "-" + RANDOM(1000, 9999)

        INSERT INTO "activities" VALUES {
            "id": activity_id,
            "activity_type": "general",
            "subject": subject,
            "description": description,
            "regarding_type": regarding_type,
            "regarding_id": record.id,
            "regarding_name": record.name,
            "duration_minutes": duration,
            "outcome": outcome,
            "status": "completed",
            "owner_id": GET SESSION "user_id",
            "owner_name": GET SESSION "user_name",
            "created_at": NOW(),
            "activity_date": NOW()
        }

        RECORD METRIC "crm_activities" WITH activity_type = "general", outcome = outcome

        TALK "Activity logged successfully."
        TALK "Activity ID: " + activity_id
END SELECT

' Show recent activities for context
HEAR show_recent AS BOOLEAN WITH "Would you like to see recent activities?"

IF show_recent THEN
    HEAR filter_type AS TEXT WITH "Filter by: (all, lead, contact, account, opportunity)" DEFAULT "all"

    IF filter_type = "all" THEN
        recent = FIND "activities" ORDER BY activity_date DESC LIMIT 10
    ELSE
        recent = FIND "activities" WHERE regarding_type = filter_type ORDER BY activity_date DESC LIMIT 10
    END IF

    IF COUNT(recent) = 0 THEN
        TALK "No recent activities found."
    ELSE
        TALK "Recent Activities:"
        TALK ""

        FOR EACH act IN recent
            date_str = FORMAT(act.activity_date, "MMM DD, YYYY HH:mm")
            TALK act.activity_type + " - " + act.subject
            TALK "  Regarding: " + act.regarding_name + " (" + act.regarding_type + ")"
            TALK "  Date: " + date_str + " | Duration: " + act.duration_minutes + " min"
            TALK "  Status: " + act.status
            TALK ""
        NEXT
    END IF
END IF

' Activity analytics summary
HEAR show_summary AS BOOLEAN WITH "Would you like to see activity summary?"

IF show_summary THEN
    HEAR summary_period AS TEXT WITH "Period: (today, week, month)" DEFAULT "week"

    SELECT CASE summary_period
        CASE "today"
            start_date = TODAY()
        CASE "week"
            start_date = DATEADD(TODAY(), -7, "day")
        CASE "month"
            start_date = DATEADD(TODAY(), -30, "day")
    END SELECT

    activities = FIND "activities" WHERE activity_date >= start_date

    call_count = COUNT(FILTER(activities, "activity_type = 'call'"))
    email_count = COUNT(FILTER(activities, "activity_type = 'email'"))
    meeting_count = COUNT(FILTER(activities, "activity_type = 'meeting'"))
    task_count = COUNT(FILTER(activities, "activity_type = 'task'"))
    total_duration = SUM(activities, "duration_minutes")

    TALK "Activity Summary for " + summary_period
    TALK ""
    TALK "Calls: " + call_count
    TALK "Emails: " + email_count
    TALK "Meetings: " + meeting_count
    TALK "Tasks: " + task_count
    TALK ""
    TALK "Total Activities: " + COUNT(activities)
    TALK "Total Time: " + FORMAT(total_duration / 60, "#.#") + " hours"
END IF
