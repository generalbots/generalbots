REM ============================================================================
REM Lead Nurturing Campaign with AI Scoring
REM General Bots Marketing Automation Template
REM ============================================================================
REM This campaign automatically nurtures leads based on their AI-calculated
REM lead score, sending personalized content at optimal intervals.
REM ============================================================================

DESCRIPTION "AI-powered lead nurturing campaign with dynamic scoring and personalized messaging"

REM ============================================================================
REM Campaign Configuration
REM ============================================================================

campaign_name = "lead-nurture-2025"
campaign_duration_days = 30
min_score_threshold = 20
mql_threshold = 70
sql_threshold = 85

REM Email sending intervals (in days)
email_interval_cold = 7
email_interval_warm = 3
email_interval_hot = 1

REM ============================================================================
REM Main Campaign Entry Point
REM ============================================================================

REM This is triggered by ON FORM SUBMIT from landing pages
ON FORM SUBMIT "landing-page"
    REM Extract lead data from form submission
    lead_email = fields.email
    lead_name = NVL(fields.name, "there")
    lead_company = fields.company
    lead_phone = fields.phone
    lead_source = metadata.utm_source

    TALK "🎯 New lead captured: " + lead_email

    REM Create lead profile for scoring
    lead = NEW OBJECT
    lead.email = lead_email
    lead.name = lead_name
    lead.company = lead_company
    lead.source = lead_source
    lead.created_at = NOW()

    REM Calculate initial AI lead score
    score_result = AI SCORE LEAD lead

    TALK "📊 Lead Score: " + score_result.score + " (Grade: " + score_result.grade + ")"

    REM Save lead to CRM
    SAVE "leads", lead_email, lead_name, lead_company, lead_phone, lead_source, score_result.score, score_result.grade, NOW()

    REM Determine nurture track based on score
    IF score_result.score >= sql_threshold THEN
        REM Hot lead - immediate sales handoff
        CALL hot_lead_workflow(lead, score_result)
    ELSE IF score_result.score >= mql_threshold THEN
        REM Warm lead - accelerated nurture
        CALL warm_lead_workflow(lead, score_result)
    ELSE IF score_result.score >= min_score_threshold THEN
        REM Cold lead - standard nurture
        CALL cold_lead_workflow(lead, score_result)
    ELSE
        REM Very cold - add to long-term drip
        CALL drip_campaign_workflow(lead, score_result)
    END IF
END ON

REM ============================================================================
REM Hot Lead Workflow (Score >= 85)
REM ============================================================================

SUB hot_lead_workflow(lead, score_result)
    TALK "🔥 HOT LEAD: " + lead.email + " - Initiating sales handoff"

    REM Send immediate welcome + calendar booking
    vars = NEW OBJECT
    vars.name = lead.name
    vars.score = score_result.score
    vars.company = NVL(lead.company, "your company")

    REM Send personalized welcome via multiple channels
    SEND TEMPLATE "hot-lead-welcome", "email", lead.email, vars

    IF NOT ISEMPTY(lead.phone) THEN
        SEND TEMPLATE "hot-lead-sms", "sms", lead.phone, vars
    END IF

    REM Create task for sales team
    CREATE TASK "Contact hot lead: " + lead.email, "sales-team", "high", NOW()

    REM Send Slack notification to sales
    sales_alert = "🔥 *HOT LEAD ALERT*\n"
    sales_alert = sales_alert + "Email: " + lead.email + "\n"
    sales_alert = sales_alert + "Score: " + score_result.score + "\n"
    sales_alert = sales_alert + "Company: " + NVL(lead.company, "Unknown") + "\n"
    sales_alert = sales_alert + "Action: Immediate follow-up required!"

    POST "https://hooks.slack.com/services/YOUR_WEBHOOK", #{text: sales_alert}

    REM Schedule follow-up if no response in 24 hours
    SET SCHEDULE "0 9 * * *", "hot-lead-followup.bas"

    TALK "✅ Hot lead workflow completed for " + lead.email
END SUB

REM ============================================================================
REM Warm Lead Workflow (Score 70-84)
REM ============================================================================

SUB warm_lead_workflow(lead, score_result)
    TALK "🌡️ WARM LEAD: " + lead.email + " - Starting accelerated nurture"

    vars = NEW OBJECT
    vars.name = lead.name
    vars.company = NVL(lead.company, "your company")

    REM Day 0: Welcome email with case study
    SEND TEMPLATE "warm-welcome", "email", lead.email, vars

    REM Schedule Day 3: Value proposition email
    nurture_data = #{
        lead_email: lead.email,
        lead_name: lead.name,
        template: "warm-value-prop",
        step: 2
    }
    SET SCHEDULE DATEADD(NOW(), 3, "day"), "send-nurture-email.bas"

    REM Schedule Day 7: Demo invitation
    SET SCHEDULE DATEADD(NOW(), 7, "day"), "warm-demo-invite.bas"

    REM Schedule Day 14: Re-score and evaluate
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "rescore-lead.bas"

    TALK "✅ Warm lead nurture sequence started for " + lead.email
END SUB

REM ============================================================================
REM Cold Lead Workflow (Score 20-69)
REM ============================================================================

SUB cold_lead_workflow(lead, score_result)
    TALK "❄️ COLD LEAD: " + lead.email + " - Starting standard nurture"

    vars = NEW OBJECT
    vars.name = lead.name
    vars.company = NVL(lead.company, "your organization")

    REM Day 0: Welcome email
    SEND TEMPLATE "cold-welcome", "email", lead.email, vars

    REM Day 7: Educational content
    SET SCHEDULE DATEADD(NOW(), 7, "day"), "cold-education-1.bas"

    REM Day 14: More educational content
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "cold-education-2.bas"

    REM Day 21: Soft pitch
    SET SCHEDULE DATEADD(NOW(), 21, "day"), "cold-soft-pitch.bas"

    REM Day 30: Re-score and decide next steps
    SET SCHEDULE DATEADD(NOW(), 30, "day"), "rescore-lead.bas"

    TALK "✅ Cold lead nurture sequence started for " + lead.email
END SUB

REM ============================================================================
REM Long-term Drip Campaign (Score < 20)
REM ============================================================================

SUB drip_campaign_workflow(lead, score_result)
    TALK "💧 LOW SCORE LEAD: " + lead.email + " - Adding to drip campaign"

    vars = NEW OBJECT
    vars.name = lead.name

    REM Simple welcome only
    SEND TEMPLATE "drip-welcome", "email", lead.email, vars

    REM Add to monthly newsletter
    SAVE "newsletter_subscribers", lead.email, lead.name, NOW()

    REM Schedule monthly check-in
    SET SCHEDULE "0 10 1 * *", "monthly-drip-check.bas"

    TALK "✅ Added " + lead.email + " to long-term drip campaign"
END SUB

REM ============================================================================
REM Scheduled: Re-score Lead and Adjust Campaign
REM ============================================================================

SUB rescore_lead()
    PARAM lead_email AS string

    REM Get current lead data
    lead_data = FIND "leads", "email = '" + lead_email + "'"

    IF ISEMPTY(lead_data) THEN
        TALK "⚠️ Lead not found: " + lead_email
        RETURN
    END IF

    REM Get updated behavior data
    lead = NEW OBJECT
    lead.email = lead_email
    lead.name = lead_data.name
    lead.company = lead_data.company

    REM Recalculate score with latest behavior
    new_score = AI SCORE LEAD lead
    old_score = lead_data.score

    score_change = new_score.score - old_score

    TALK "📊 Lead Rescore: " + lead_email
    TALK "   Old Score: " + old_score + " → New Score: " + new_score.score
    TALK "   Change: " + IIF(score_change >= 0, "+", "") + score_change

    REM Update stored score
    UPDATE "leads", lead_email, #{score: new_score.score, grade: new_score.grade, updated_at: NOW()}

    REM Check if lead should be promoted to higher tier
    IF old_score < mql_threshold AND new_score.score >= mql_threshold THEN
        TALK "🎉 Lead promoted to MQL: " + lead_email
        CALL warm_lead_workflow(lead, new_score)

        REM Notify marketing team
        SEND TEMPLATE "mql-promotion-alert", "email", "marketing@company.com", #{
            lead_email: lead_email,
            old_score: old_score,
            new_score: new_score.score
        }
    ELSE IF old_score < sql_threshold AND new_score.score >= sql_threshold THEN
        TALK "🔥 Lead promoted to SQL: " + lead_email
        CALL hot_lead_workflow(lead, new_score)
    ELSE IF score_change < -20 THEN
        TALK "⚠️ Significant score drop for: " + lead_email
        REM Move to re-engagement campaign
        CALL reengagement_workflow(lead, new_score)
    END IF
END SUB

REM ============================================================================
REM Re-engagement Workflow for Declining Leads
REM ============================================================================

SUB reengagement_workflow(lead, score_result)
    TALK "🔄 Starting re-engagement for: " + lead.email

    vars = NEW OBJECT
    vars.name = lead.name

    REM Send re-engagement email
    SEND TEMPLATE "reengagement", "email", lead.email, vars

    REM If we have phone, send SMS too
    IF NOT ISEMPTY(lead.phone) THEN
        SEND TEMPLATE "reengagement-sms", "sms", lead.phone, vars
    END IF

    REM Schedule unsubscribe if no engagement in 14 days
    SET SCHEDULE DATEADD(NOW(), 14, "day"), "check-reengagement.bas"

    TALK "✅ Re-engagement campaign started for " + lead.email
END SUB

REM ============================================================================
REM Utility: Send Nurture Email by Step
REM ============================================================================

SUB send_nurture_email()
    PARAM lead_email AS string
    PARAM template_name AS string
    PARAM step AS integer

    REM Get lead data
    lead_data = FIND "leads", "email = '" + lead_email + "'"

    IF ISEMPTY(lead_data) THEN
        TALK "⚠️ Lead not found, skipping: " + lead_email
        RETURN
    END IF

    REM Check if lead has unsubscribed
    unsubscribed = FIND "unsubscribes", "email = '" + lead_email + "'"
    IF NOT ISEMPTY(unsubscribed) THEN
        TALK "⏹️ Lead unsubscribed, stopping nurture: " + lead_email
        RETURN
    END IF

    REM Check current score - maybe they've become hot
    current_score = GET LEAD SCORE lead_email

    IF current_score.score >= sql_threshold THEN
        TALK "🔥 Lead is now hot! Switching to hot workflow: " + lead_email
        lead = #{email: lead_email, name: lead_data.name, company: lead_data.company}
        CALL hot_lead_workflow(lead, current_score)
        RETURN
    END IF

    REM Send the scheduled email
    vars = NEW OBJECT
    vars.name = lead_data.name
    vars.company = NVL(lead_data.company, "your organization")
    vars.step = step

    result = SEND TEMPLATE template_name, "email", lead_email, vars

    IF result[0].success THEN
        TALK "✉️ Nurture email sent: " + template_name + " to " + lead_email

        REM Track email send
        SAVE "email_tracking", lead_email, template_name, step, NOW(), "sent"

        REM Update lead score for engagement
        UPDATE LEAD SCORE lead_email, 2, "Nurture email " + step + " sent"
    ELSE
        TALK "❌ Failed to send nurture email: " + result[0].error
    END IF
END SUB

REM ============================================================================
REM Campaign Analytics
REM ============================================================================

SUB get_campaign_analytics()
    TALK "📈 Campaign Analytics for: " + campaign_name
    TALK "================================"

    REM Total leads
    total_leads = AGGREGATE "leads", "COUNT", "email"
    TALK "Total Leads: " + total_leads

    REM Leads by grade
    grade_a = AGGREGATE "leads", "COUNT", "email", "grade = 'A'"
    grade_b = AGGREGATE "leads", "COUNT", "email", "grade = 'B'"
    grade_c = AGGREGATE "leads", "COUNT", "email", "grade = 'C'"
    grade_d = AGGREGATE "leads", "COUNT", "email", "grade = 'D'"

    TALK "Grade Distribution:"
    TALK "  A (Hot): " + grade_a
    TALK "  B (Warm): " + grade_b
    TALK "  C (Neutral): " + grade_c
    TALK "  D (Cold): " + grade_d

    REM Average score
    avg_score = AGGREGATE "leads", "AVG", "score"
    TALK "Average Lead Score: " + ROUND(avg_score, 1)

    REM Conversion rates
    mql_count = AGGREGATE "leads", "COUNT", "email", "score >= " + mql_threshold
    sql_count = AGGREGATE "leads", "COUNT", "email", "score >= " + sql_threshold

    IF total_leads > 0 THEN
        mql_rate = ROUND((mql_count / total_leads) * 100, 1)
        sql_rate = ROUND((sql_count / total_leads) * 100, 1)

        TALK "Conversion Rates:"
        TALK "  MQL Rate: " + mql_rate + "%"
        TALK "  SQL Rate: " + sql_rate + "%"
    END IF

    REM Email performance
    emails_sent = AGGREGATE "email_tracking", "COUNT", "id", "status = 'sent'"
    TALK "Emails Sent: " + emails_sent

    TALK "================================"

    RETURN #{
        total_leads: total_leads,
        grade_a: grade_a,
        grade_b: grade_b,
        grade_c: grade_c,
        grade_d: grade_d,
        avg_score: avg_score,
        mql_count: mql_count,
        sql_count: sql_count,
        emails_sent: emails_sent
    }
END SUB

REM ============================================================================
REM Entry point for manual campaign trigger
REM ============================================================================

TALK "🚀 Lead Nurturing Campaign Ready: " + campaign_name
TALK "📊 MQL Threshold: " + mql_threshold
TALK "🔥 SQL Threshold: " + sql_threshold
TALK "⏰ Campaign Duration: " + campaign_duration_days + " days"
TALK ""
TALK "Waiting for form submissions..."
