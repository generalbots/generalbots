' Lead Nurturing Campaign with AI Scoring
' AI-powered lead nurturing with dynamic scoring and personalized messaging

DESCRIPTION "AI-powered lead nurturing campaign with dynamic scoring"

' Campaign Configuration
campaign_name = "lead-nurture-2025"
mql_threshold = 70
sql_threshold = 85

' Form Submit Handler - triggered by landing page submissions
ON FORM SUBMIT "landing-page"
    lead_email = fields.email
    lead_name = NVL(fields.name, "there")
    lead_company = fields.company
    lead_phone = fields.phone
    lead_source = metadata.utm_source

    TALK "New lead captured: " + lead_email

    ' Score the lead with AI
    WITH lead
        .email = lead_email
        .name = lead_name
        .company = lead_company
        .source = lead_source
        .created_at = NOW()
    END WITH

    score_result = AI SCORE LEAD lead

    TALK "Lead Score: " + score_result.score + " (Grade: " + score_result.grade + ")"

    ' Save to CRM
    SAVE "leads", lead_email, lead_name, lead_company, lead_phone, lead_source, score_result.score, score_result.grade, NOW()

    ' Route based on score
    IF score_result.score >= sql_threshold THEN
        CALL hot_lead_workflow(lead, score_result)
    ELSE IF score_result.score >= mql_threshold THEN
        CALL warm_lead_workflow(lead, score_result)
    ELSE
        CALL cold_lead_workflow(lead, score_result)
    END IF
END ON

' Hot Lead Workflow (Score >= 85)
SUB hot_lead_workflow(lead, score_result)
    TALK "HOT LEAD: " + lead.email + " - Sales handoff"

    WITH vars
        .name = lead.name
        .score = score_result.score
        .company = NVL(lead.company, "your company")
    END WITH

    SEND TEMPLATE "hot-lead-welcome", "email", lead.email, vars

    IF NOT ISEMPTY(lead.phone) THEN
        SEND TEMPLATE "hot-lead-sms", "sms", lead.phone, vars
    END IF

    CREATE TASK "Contact hot lead: " + lead.email, "sales-team", "high", NOW()

    ' Slack notification
    WITH alert
        .text = "HOT LEAD: " + lead.email + " | Score: " + score_result.score
    END WITH
    POST "https://hooks.slack.com/services/YOUR_WEBHOOK", alert

    SET SCHEDULE DATEADD(NOW(), 1, "day"), "hot-lead-followup.bas"

    TALK "Hot lead workflow completed"
END SUB

' Warm Lead Workflow (Score 70-84)
SUB warm_lead_workflow(lead, score_result)
    TALK "WARM LEAD: " + lead.email + " - Accelerated nurture"

    WITH vars
        .name = lead.name
        .company = NVL(lead.company, "your company")
    END WITH

    SEND TEMPLATE "warm-welcome", "email", lead.email, vars

    SET SCHEDULE DATEADD(NOW(), 3, "day"), "warm-nurture-2.bas"
    SET SCHEDULE DATEADD(NOW(), 7, "day"), "warm-demo-invite.bas"
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "rescore-lead.bas"

    TALK "Warm lead nurture started"
END SUB

' Cold Lead Workflow (Score < 70)
SUB cold_lead_workflow(lead, score_result)
    TALK "COLD LEAD: " + lead.email + " - Standard nurture"

    WITH vars
        .name = lead.name
        .company = NVL(lead.company, "your organization")
    END WITH

    SEND TEMPLATE "cold-welcome", "email", lead.email, vars

    SET SCHEDULE DATEADD(NOW(), 7, "day"), "cold-education-1.bas"
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "cold-education-2.bas"
    SET SCHEDULE DATEADD(NOW(), 21, "day"), "cold-soft-pitch.bas"
    SET SCHEDULE DATEADD(NOW(), 30, "day"), "rescore-lead.bas"

    TALK "Cold lead nurture started"
END SUB

' Re-score Lead (scheduled)
SUB rescore_lead()
    PARAM lead_email AS string

    lead_data = FIND "leads", "email = '" + lead_email + "'"

    IF ISEMPTY(lead_data) THEN
        TALK "Lead not found: " + lead_email
        RETURN
    END IF

    WITH lead
        .email = lead_email
        .name = lead_data.name
        .company = lead_data.company
    END WITH

    new_score = AI SCORE LEAD lead
    old_score = lead_data.score
    score_change = new_score.score - old_score

    TALK "Lead Rescore: " + lead_email
    TALK "Old: " + old_score + " -> New: " + new_score.score + " (" + IIF(score_change >= 0, "+", "") + score_change + ")"

    UPDATE "leads", lead_email, new_score.score, new_score.grade, NOW()

    IF old_score < mql_threshold AND new_score.score >= mql_threshold THEN
        TALK "Lead promoted to MQL: " + lead_email
        CALL warm_lead_workflow(lead, new_score)
        SEND TEMPLATE "mql-promotion-alert", "email", "marketing@company.com", lead
    ELSE IF old_score < sql_threshold AND new_score.score >= sql_threshold THEN
        TALK "Lead promoted to SQL: " + lead_email
        CALL hot_lead_workflow(lead, new_score)
    END IF
END SUB

' Send Nurture Email (utility)
SUB send_nurture_email()
    PARAM lead_email AS string
    PARAM template_name AS string
    PARAM step AS integer

    lead_data = FIND "leads", "email = '" + lead_email + "'"

    IF ISEMPTY(lead_data) THEN
        RETURN
    END IF

    unsubscribed = FIND "unsubscribes", "email = '" + lead_email + "'"
    IF NOT ISEMPTY(unsubscribed) THEN
        TALK "Lead unsubscribed: " + lead_email
        RETURN
    END IF

    current_score = GET LEAD SCORE lead_email

    IF current_score.score >= sql_threshold THEN
        WITH lead
            .email = lead_email
            .name = lead_data.name
            .company = lead_data.company
        END WITH
        CALL hot_lead_workflow(lead, current_score)
        RETURN
    END IF

    WITH vars
        .name = lead_data.name
        .company = NVL(lead_data.company, "your organization")
        .step = step
    END WITH

    result = SEND TEMPLATE template_name, "email", lead_email, vars

    IF result[0].success THEN
        TALK "Nurture email sent: " + template_name + " to " + lead_email
        SAVE "email_tracking", lead_email, template_name, step, NOW(), "sent"
        UPDATE LEAD SCORE lead_email, 2, "Nurture email " + step + " sent"
    ELSE
        TALK "Failed to send: " + result[0].error
    END IF
END SUB

' Campaign Analytics
SUB get_campaign_analytics()
    TALK "Campaign Analytics: " + campaign_name

    total_leads = AGGREGATE "leads", "COUNT", "email"
    grade_a = AGGREGATE "leads", "COUNT", "email", "grade = 'A'"
    grade_b = AGGREGATE "leads", "COUNT", "email", "grade = 'B'"
    grade_c = AGGREGATE "leads", "COUNT", "email", "grade = 'C'"
    avg_score = AGGREGATE "leads", "AVG", "score"
    mql_count = AGGREGATE "leads", "COUNT", "email", "score >= " + mql_threshold
    sql_count = AGGREGATE "leads", "COUNT", "email", "score >= " + sql_threshold
    emails_sent = AGGREGATE "email_tracking", "COUNT", "id", "status = 'sent'"

    TALK "Total Leads: " + total_leads
    TALK "Grade A: " + grade_a + " | B: " + grade_b + " | C: " + grade_c
    TALK "Avg Score: " + ROUND(avg_score, 1)

    IF total_leads > 0 THEN
        mql_rate = ROUND((mql_count / total_leads) * 100, 1)
        sql_rate = ROUND((sql_count / total_leads) * 100, 1)
        TALK "MQL Rate: " + mql_rate + "% | SQL Rate: " + sql_rate + "%"
    END IF

    TALK "Emails Sent: " + emails_sent

    WITH stats
        .total_leads = total_leads
        .grade_a = grade_a
        .grade_b = grade_b
        .grade_c = grade_c
        .avg_score = avg_score
        .mql_count = mql_count
        .sql_count = sql_count
    END WITH

    RETURN stats
END SUB

TALK "Lead Nurturing Campaign Ready: " + campaign_name
TALK "MQL Threshold: " + mql_threshold + " | SQL Threshold: " + sql_threshold
