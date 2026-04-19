PARAM job_name AS STRING

user_id = GET "session.user_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF job_name = "lead_scoring" THEN
    leads = FIND "leads", "status != 'converted' AND status != 'unqualified'"

    FOR EACH lead IN leads DO
        score = 0

        days_old = DAYS_BETWEEN(lead.created_at, current_time)
        IF days_old < 7 THEN
            score = score + 10
        ELSE IF days_old < 30 THEN
            score = score + 5
        END IF

        activities = FIND "activities", "lead_id = '" + lead.id + "'"
        activity_count = 0
        FOR EACH activity IN activities DO
            activity_count = activity_count + 1
        END FOR

        IF activity_count > 10 THEN
            score = score + 20
        ELSE IF activity_count > 5 THEN
            score = score + 10
        ELSE IF activity_count > 0 THEN
            score = score + 5
        END IF

        IF lead.email != "" THEN
            score = score + 5
        END IF

        IF lead.phone != "" THEN
            score = score + 5
        END IF

        IF lead.company_name != "" THEN
            score = score + 10
        END IF

        lead.score = score

        IF score > 50 THEN
            lead.status = "hot"
        ELSE IF score > 30 THEN
            lead.status = "warm"
        ELSE IF score > 10 THEN
            lead.status = "cold"
        END IF

        SAVE_FROM_UNSTRUCTURED "leads", FORMAT lead AS JSON
    END FOR

    TALK "Lead scoring completed for " + activity_count + " leads"
END IF

IF job_name = "opportunity_reminder" THEN
    opportunities = FIND "opportunities", "stage != 'closed_won' AND stage != 'closed_lost'"

    FOR EACH opp IN opportunities DO
        days_until_close = DAYS_BETWEEN(current_time, opp.close_date)

        IF days_until_close = 7 THEN
            notification = "Opportunity " + opp.name + " closes in 7 days"
            SEND MAIL opp.owner_id, "Opportunity Reminder", notification
            CREATE_TASK "Follow up on " + opp.name, "high", opp.owner_id

        ELSE IF days_until_close = 1 THEN
            notification = "URGENT: Opportunity " + opp.name + " closes tomorrow!"
            SEND MAIL opp.owner_id, "Urgent Opportunity Alert", notification
            CREATE_TASK "Close deal: " + opp.name, "critical", opp.owner_id

        ELSE IF days_until_close < 0 THEN
            opp.stage = "closed_lost"
            opp.closed_at = current_time
            opp.loss_reason = "Expired - no action taken"
            SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opp AS JSON
        END IF
    END FOR
END IF

IF job_name = "case_escalation" THEN
    cases = FIND "cases", "status = 'new' OR status = 'in_progress'"

    FOR EACH case IN cases DO
        hours_open = HOURS_BETWEEN(case.created_at, current_time)

        escalate = false
        IF case.priority = "critical" AND hours_open > 2 THEN
            escalate = true
        ELSE IF case.priority = "high" AND hours_open > 4 THEN
            escalate = true
        ELSE IF case.priority = "medium" AND hours_open > 8 THEN
            escalate = true
        ELSE IF case.priority = "low" AND hours_open > 24 THEN
            escalate = true
        END IF

        IF escalate = true AND case.status != "escalated" THEN
            case.status = "escalated"
            case.escalated_at = current_time
            SAVE_FROM_UNSTRUCTURED "cases", FORMAT case AS JSON

            notification = "ESCALATION: Case " + case.case_number + " - " + case.subject
            SEND MAIL "support-manager@company.com", "Case Escalation", notification
            CREATE_TASK "Handle escalated case " + case.case_number, "critical", "support-manager"
        END IF
    END FOR
END IF

IF job_name = "email_campaign" THEN
    leads = FIND "leads", "status = 'warm'"

    FOR EACH lead IN leads DO
        last_contact = GET "lead_last_contact_" + lead.id

        IF last_contact = "" THEN
            last_contact = lead.created_at
        END IF

        days_since_contact = DAYS_BETWEEN(last_contact, current_time)

        IF days_since_contact = 3 THEN
            subject = "Following up on your interest"
            message = "Hi " + lead.contact_name + ",\n\nI wanted to follow up on your recent inquiry..."
            SEND MAIL lead.email, subject, message
            REMEMBER "lead_last_contact_" + lead.id = current_time

        ELSE IF days_since_contact = 7 THEN
            subject = "Special offer for you"
            message = "Hi " + lead.contact_name + ",\n\nWe have a special offer..."
            SEND MAIL lead.email, subject, message
            REMEMBER "lead_last_contact_" + lead.id = current_time

        ELSE IF days_since_contact = 14 THEN
            subject = "Last chance - Limited time offer"
            message = "Hi " + lead.contact_name + ",\n\nThis is your last chance..."
            SEND MAIL lead.email, subject, message
            REMEMBER "lead_last_contact_" + lead.id = current_time

        ELSE IF days_since_contact > 30 THEN
            lead.status = "cold"
            SAVE_FROM_UNSTRUCTURED "leads", FORMAT lead AS JSON
        END IF
    END FOR
END IF

IF job_name = "activity_cleanup" THEN
    old_date = FORMAT ADD_DAYS(NOW(), -90) AS "YYYY-MM-DD"
    activities = FIND "activities", "created_at < '" + old_date + "' AND status = 'completed'"

    archive_count = 0
    FOR EACH activity IN activities DO
        archive = CREATE OBJECT
        SET archive.original_id = activity.id
        SET archive.data = FORMAT activity AS JSON
        SET archive.archived_at = current_time

        SAVE_FROM_UNSTRUCTURED "activities_archive", FORMAT archive AS JSON
        archive_count = archive_count + 1
    END FOR

    TALK "Archived " + archive_count + " old activities"
END IF

IF job_name = "daily_digest" THEN
    new_leads = FIND "leads", "DATE(created_at) = DATE('" + current_time + "')"
    new_opportunities = FIND "opportunities", "DATE(created_at) = DATE('" + current_time + "')"
    closed_won = FIND "opportunities", "DATE(closed_at) = DATE('" + current_time + "') AND won = true"
    new_cases = FIND "cases", "DATE(created_at) = DATE('" + current_time + "')"

    lead_count = 0
    opp_count = 0
    won_count = 0
    won_amount = 0
    case_count = 0

    FOR EACH lead IN new_leads DO
        lead_count = lead_count + 1
    END FOR

    FOR EACH opp IN new_opportunities DO
        opp_count = opp_count + 1
    END FOR

    FOR EACH deal IN closed_won DO
        won_count = won_count + 1
        won_amount = won_amount + deal.amount
    END FOR

    FOR EACH case IN new_cases DO
        case_count = case_count + 1
    END FOR

    digest = "DAILY CRM DIGEST - " + current_time + "\n"
    digest = digest + "=====================================\n\n"
    digest = digest + "NEW ACTIVITY TODAY:\n"
    digest = digest + "- New Leads: " + lead_count + "\n"
    digest = digest + "- New Opportunities: " + opp_count + "\n"
    digest = digest + "- Deals Won: " + won_count + " ($" + won_amount + ")\n"
    digest = digest + "- Support Cases: " + case_count + "\n\n"

    digest = digest + "PIPELINE STATUS:\n"

    open_opps = FIND "opportunities", "stage != 'closed_won' AND stage != 'closed_lost'"
    total_pipeline = 0
    FOR EACH opp IN open_opps DO
        total_pipeline = total_pipeline + opp.amount
    END FOR

    digest = digest + "- Total Pipeline Value: $" + total_pipeline + "\n"

    SEND MAIL "management@company.com", "Daily CRM Digest", digest

    TALK "Daily digest sent to management"
END IF

IF job_name = "setup_schedules" THEN
    SET SCHEDULE "0 9 * * *" "crm-jobs.bas" "lead_scoring"
    SET SCHEDULE "0 10 * * *" "crm-jobs.bas" "opportunity_reminder"
    SET SCHEDULE "*/30 * * * *" "crm-jobs.bas" "case_escalation"
    SET SCHEDULE "0 14 * * *" "crm-jobs.bas" "email_campaign"
    SET SCHEDULE "0 2 * * 0" "crm-jobs.bas" "activity_cleanup"
    SET SCHEDULE "0 18 * * *" "crm-jobs.bas" "daily_digest"

    TALK "All CRM schedules have been configured"
END IF
