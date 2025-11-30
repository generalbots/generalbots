' General Bots - Welcome Campaign Template
' Automated drip sequence for new leads/subscribers

PARAM lead_email AS string LIKE "user@example.com"
PARAM lead_name AS string LIKE "John Doe"
PARAM lead_source AS string LIKE "website"

DESCRIPTION "Welcome campaign drip sequence for new leads"

' Validate input
IF ISEMPTY(lead_email) THEN
    THROW "Email is required for welcome campaign"
END IF

IF ISEMPTY(lead_name) THEN
    lead_name = "Friend"
END IF

TALK "Starting welcome campaign for: " + lead_email

' Step 1: Immediate Welcome Email
WITH variables
    .name = lead_name
    .source = lead_source
    .date = TODAY()
END WITH

result = SEND TEMPLATE "welcome-email-1", "email", lead_email, variables

IF result[0].success THEN
    TALK "Welcome email #1 sent successfully"
    SAVE "campaign_logs", NOW(), lead_email, "welcome-1", "sent", ""
ELSE
    TALK "Failed to send welcome email #1"
    SAVE "campaign_logs", NOW(), lead_email, "welcome-1", "failed", result[0].error
END IF

' Step 2: Schedule Follow-up Emails
SET SCHEDULE "0 9 * * *" DATEADD(TODAY(), 2, "day"), "send-welcome-2.bas"
SET SCHEDULE "0 9 * * *" DATEADD(TODAY(), 5, "day"), "send-welcome-3.bas"
SET SCHEDULE "0 9 * * *" DATEADD(TODAY(), 8, "day"), "send-welcome-4.bas"
SET SCHEDULE "0 9 * * *" DATEADD(TODAY(), 14, "day"), "send-welcome-5.bas"

' Step 3: Add to CRM and Score Lead
SAVE "leads", lead_email, lead_name, lead_source, "Welcome Series", "nurturing", NOW()

' Calculate initial lead score
WITH score_data
    .email = lead_email
    .name = lead_name
    .source = lead_source
    .form_submissions = 1
END WITH

lead_score = SCORE LEAD score_data

TALK "Initial lead score: " + lead_score.score + " (Grade: " + lead_score.grade + ")"

' Step 4: Track Campaign Enrollment
SAVE "campaign_enrollments", "Welcome Series", lead_email, lead_name, NOW(), "active", 1, 5

' Step 5: Send to WhatsApp if phone provided
lead_phone = GET BOT MEMORY "lead_phone_" + lead_email

IF NOT ISEMPTY(lead_phone) THEN
    WITH wa_vars
        .name = lead_name
    END WITH

    wa_result = SEND TEMPLATE "welcome-whatsapp", "whatsapp", lead_phone, wa_vars

    IF wa_result[0].success THEN
        TALK "WhatsApp welcome sent"
    END IF
END IF

' Output Summary
TALK ""
TALK "Welcome Campaign Started"
TALK "Lead: " + lead_name + " <" + lead_email + ">"
TALK "Score: " + lead_score.score + " (" + lead_score.grade + ")"
TALK "Status: " + lead_score.status
TALK "Emails Scheduled: 5"
TALK "Campaign Duration: 14 days"

WITH campaign_status
    .campaign = "Welcome Series"
    .lead = lead_email
    .initial_score = lead_score.score
    .grade = lead_score.grade
    .emails_scheduled = 5
    .next_email_date = DATEADD(TODAY(), 2, "day")
    .status = "active"
END WITH

RETURN campaign_status
