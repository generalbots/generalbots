' =============================================================================
' Custom Report Generator Dialog
' General Bots Analytics Template
' =============================================================================
' This dialog allows users to create custom reports from platform metrics
' =============================================================================

TALK "Custom Report Generator"
TALK "I will help you create a custom analytics report."

HEAR report_name AS TEXT WITH "What would you like to name this report?"

TALK "Select the time range for your report:"
TALK "1. Last Hour"
TALK "2. Last 24 Hours"
TALK "3. Last 7 Days"
TALK "4. Last 30 Days"
TALK "5. Last 90 Days"
TALK "6. Custom Range"

HEAR time_choice AS INTEGER

SELECT CASE time_choice
    CASE 1
        time_range = "1h"
        time_label = "Last Hour"
    CASE 2
        time_range = "24h"
        time_label = "Last 24 Hours"
    CASE 3
        time_range = "7d"
        time_label = "Last 7 Days"
    CASE 4
        time_range = "30d"
        time_label = "Last 30 Days"
    CASE 5
        time_range = "90d"
        time_label = "Last 90 Days"
    CASE 6
        HEAR start_date AS DATE WITH "Enter start date (YYYY-MM-DD):"
        HEAR end_date AS DATE WITH "Enter end date (YYYY-MM-DD):"
        time_range = "custom"
        time_label = FORMAT(start_date, "YYYY-MM-DD") + " to " + FORMAT(end_date, "YYYY-MM-DD")
    CASE ELSE
        time_range = "7d"
        time_label = "Last 7 Days"
END SELECT

TALK "Select metrics to include (enter numbers separated by commas):"
TALK "1. Message Volume"
TALK "2. Active Sessions"
TALK "3. Response Time"
TALK "4. LLM Token Usage"
TALK "5. Error Rate"
TALK "6. Storage Usage"
TALK "7. API Calls"
TALK "8. User Activity"
TALK "9. Bot Performance"
TALK "10. All Metrics"

HEAR metrics_choice AS TEXT

metrics_list = SPLIT(metrics_choice, ",")

include_messages = CONTAINS(metrics_list, "1") OR CONTAINS(metrics_list, "10")
include_sessions = CONTAINS(metrics_list, "2") OR CONTAINS(metrics_list, "10")
include_response = CONTAINS(metrics_list, "3") OR CONTAINS(metrics_list, "10")
include_tokens = CONTAINS(metrics_list, "4") OR CONTAINS(metrics_list, "10")
include_errors = CONTAINS(metrics_list, "5") OR CONTAINS(metrics_list, "10")
include_storage = CONTAINS(metrics_list, "6") OR CONTAINS(metrics_list, "10")
include_api = CONTAINS(metrics_list, "7") OR CONTAINS(metrics_list, "10")
include_users = CONTAINS(metrics_list, "8") OR CONTAINS(metrics_list, "10")
include_bots = CONTAINS(metrics_list, "9") OR CONTAINS(metrics_list, "10")

TALK "Select grouping interval:"
TALK "1. Hourly"
TALK "2. Daily"
TALK "3. Weekly"
TALK "4. Monthly"

HEAR group_choice AS INTEGER

SELECT CASE group_choice
    CASE 1
        group_interval = "1h"
    CASE 2
        group_interval = "1d"
    CASE 3
        group_interval = "1w"
    CASE 4
        group_interval = "1mo"
    CASE ELSE
        group_interval = "1d"
END SELECT

TALK "Generating your custom report..."

report_data = {}
report_data.name = report_name
report_data.time_range = time_label
report_data.generated_at = NOW()
report_data.generated_by = GET SESSION "user_email"

IF include_messages THEN
    messages = QUERY METRICS "messages" FOR time_range BY group_interval
    report_data.messages = messages
    report_data.total_messages = SUM(messages, "count")
END IF

IF include_sessions THEN
    sessions = QUERY METRICS "active_sessions" FOR time_range BY group_interval
    report_data.sessions = sessions
    report_data.peak_sessions = MAX(sessions, "count")
END IF

IF include_response THEN
    response_times = QUERY METRICS "response_time" FOR time_range BY group_interval
    report_data.response_times = response_times
    report_data.avg_response_ms = AVG(response_times, "duration_ms")
END IF

IF include_tokens THEN
    tokens = QUERY METRICS "llm_tokens" FOR time_range BY group_interval
    report_data.tokens = tokens
    report_data.total_tokens = SUM(tokens, "total_tokens")
END IF

IF include_errors THEN
    errors = QUERY METRICS "errors" FOR time_range BY group_interval
    report_data.errors = errors
    report_data.total_errors = SUM(errors, "count")
END IF

IF include_storage THEN
    storage = QUERY METRICS "storage_usage" FOR time_range BY group_interval
    report_data.storage = storage
    report_data.current_storage_gb = LAST(storage, "bytes_used") / 1073741824
END IF

IF include_api THEN
    api_calls = QUERY METRICS "api_requests" FOR time_range BY group_interval
    report_data.api_calls = api_calls
    report_data.total_api_calls = SUM(api_calls, "count")
END IF

IF include_users THEN
    users = FIND "users" WHERE last_login >= DATEADD(NOW(), -30, "day")
    report_data.active_users_30d = COUNT(users)
END IF

IF include_bots THEN
    bots = FIND "bots" WHERE status = "active"
    report_data.active_bots = COUNT(bots)
END IF

SET CONTEXT "You are an analytics expert. Generate executive insights from this report data."

insights = LLM "Analyze this platform data and provide 3-5 key insights for executives: " + JSON(report_data)
report_data.ai_insights = insights

TALK "Select export format:"
TALK "1. PDF Report"
TALK "2. Excel Spreadsheet"
TALK "3. CSV Data"
TALK "4. JSON Data"
TALK "5. All Formats"

HEAR format_choice AS INTEGER

timestamp = FORMAT(NOW(), "YYYYMMDD_HHmmss")
base_filename = "report_" + REPLACE(report_name, " ", "_") + "_" + timestamp

SELECT CASE format_choice
    CASE 1
        filename = base_filename + ".pdf"
        GENERATE PDF filename WITH TEMPLATE "analytics_report" DATA report_data
    CASE 2
        filename = base_filename + ".xlsx"
        WRITE filename, report_data
    CASE 3
        filename = base_filename + ".csv"
        WRITE filename, CSV(report_data)
    CASE 4
        filename = base_filename + ".json"
        WRITE filename, JSON(report_data)
    CASE 5
        GENERATE PDF base_filename + ".pdf" WITH TEMPLATE "analytics_report" DATA report_data
        WRITE base_filename + ".xlsx", report_data
        WRITE base_filename + ".csv", CSV(report_data)
        WRITE base_filename + ".json", JSON(report_data)
        filename = base_filename + ".zip"
        COMPRESS filename, base_filename + ".*"
    CASE ELSE
        filename = base_filename + ".pdf"
        GENERATE PDF filename WITH TEMPLATE "analytics_report" DATA report_data
END SELECT

UPLOAD filename TO "/reports/custom/"

download_link = GENERATE SECURE LINK "/reports/custom/" + filename EXPIRES 7 DAYS

TALK "Report generated successfully."
TALK "Report Name: " + report_name
TALK "Time Range: " + time_label
TALK "Download: " + download_link

HEAR send_email AS BOOLEAN WITH "Would you like to receive this report via email?"

IF send_email THEN
    user_email = GET SESSION "user_email"
    SEND MAIL user_email, "Custom Analytics Report: " + report_name, "Your custom analytics report is ready. Download link: " + download_link + " (expires in 7 days)", ATTACHMENT filename
    TALK "Report sent to " + user_email
END IF

INSERT INTO "report_history" VALUES {
    "id": "RPT-" + timestamp,
    "name": report_name,
    "generated_by": GET SESSION "user_email",
    "generated_at": NOW(),
    "time_range": time_label,
    "metrics_included": metrics_choice,
    "filename": filename
}

TALK "Would you like to schedule this report to run automatically?"
HEAR schedule_report AS BOOLEAN

IF schedule_report THEN
    TALK "Select schedule frequency:"
    TALK "1. Daily"
    TALK "2. Weekly"
    TALK "3. Monthly"

    HEAR freq_choice AS INTEGER

    SELECT CASE freq_choice
        CASE 1
            schedule = "0 8 * * *"
            freq_label = "Daily at 8:00 AM"
        CASE 2
            schedule = "0 8 * * 1"
            freq_label = "Weekly on Monday at 8:00 AM"
        CASE 3
            schedule = "0 8 1 * *"
            freq_label = "Monthly on 1st at 8:00 AM"
        CASE ELSE
            schedule = "0 8 * * 1"
            freq_label = "Weekly on Monday at 8:00 AM"
    END SELECT

    SET BOT MEMORY "scheduled_report_" + report_name, JSON(report_data)
    SET SCHEDULE schedule, "generate-scheduled-report.bas"

    TALK "Report scheduled: " + freq_label
END IF

TALK "Thank you for using the Custom Report Generator."
