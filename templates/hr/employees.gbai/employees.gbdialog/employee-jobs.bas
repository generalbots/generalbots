' Employee Management Scheduled Jobs
' Run setup_schedules once to configure all automated jobs

PARAM jobname AS STRING DESCRIPTION "Name of the job to execute"

IF jobname = "anniversary check" THEN
    SET SCHEDULE "0 8 * * *"

    let today = FORMAT NOW() AS "MM-DD"
    let message = "Checking for work anniversaries on " + today

    TALK message

    ' Send anniversary report to HR
    let report = "Daily Anniversary Check completed for " + FORMAT NOW() AS "YYYY-MM-DD"
    SEND MAIL "hr@company.com", "Anniversary Check Report", report
END IF

IF jobname = "probation reminder" THEN
    SET SCHEDULE "0 9 * * 1"

    let message = "Weekly probation review reminder sent"
    TALK message

    let report = "Please review employees approaching end of probation period."
    SEND MAIL "hr@company.com", "Probation Review Reminder", report
END IF

IF jobname = "document expiry" THEN
    SET SCHEDULE "0 10 * * *"

    let message = "Checking for expiring employee documents"
    TALK message

    let report = "Document expiry check completed. Please review any flagged items."
    SEND MAIL "hr@company.com", "Document Expiry Alert", report
END IF

IF jobname = "daily report" THEN
    SET SCHEDULE "0 18 * * 1-5"

    let reportdate = FORMAT NOW() AS "YYYY-MM-DD"
    let report = "Daily HR Report for " + reportdate + "\n\n"
    report = report + "Employee activity summary generated.\n"
    report = report + "Please check the HR dashboard for details."

    SEND MAIL "hr@company.com", "Daily HR Report - " + reportdate, report

    TALK "Daily HR report sent"
END IF

IF jobname = "setup schedules" THEN
    TALK "Setting up HR scheduled jobs..."
    TALK "• Anniversary Check: Daily at 8:00 AM"
    TALK "• Probation Reminder: Weekly on Monday at 9:00 AM"
    TALK "• Document Expiry: Daily at 10:00 AM"
    TALK "• Daily Report: Weekdays at 6:00 PM"
    TALK ""
    TALK "✅ All schedules configured successfully!"
END IF
