PARAM action AS STRING
PARAM lead_data AS OBJECT

lead_id = GET "session.lead_id"
user_id = GET "session.user_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF action = "capture" THEN
    lead_name = GET "lead_data.name"
    lead_email = GET "lead_data.email"
    lead_phone = GET "lead_data.phone"
    lead_company = GET "lead_data.company"
    lead_source = GET "lead_data.source"

    IF lead_email = "" THEN
        TALK "I need your email to continue."
        lead_email = HEAR
    END IF

    IF lead_name = "" THEN
        TALK "May I have your name?"
        lead_name = HEAR
    END IF

    new_lead = CREATE OBJECT
    SET new_lead.id = FORMAT GUID()
    SET new_lead.name = lead_name
    SET new_lead.email = lead_email
    SET new_lead.phone = lead_phone
    SET new_lead.company = lead_company
    SET new_lead.source = lead_source
    SET new_lead.status = "new"
    SET new_lead.score = 0
    SET new_lead.created_at = current_time
    SET new_lead.assigned_to = user_id

    SAVE_FROM_UNSTRUCTURED "leads", FORMAT new_lead AS JSON

    SET "session.lead_id" = new_lead.id
    SET "session.lead_status" = "captured"

    REMEMBER "lead_" + new_lead.id = new_lead

    TALK "Thank you " + lead_name + "! I've captured your information."

END IF

IF action = "qualify" THEN
    lead = FIND "leads", "id = '" + lead_id + "'"

    IF lead = NULL THEN
        TALK "No lead found to qualify."
        EXIT
    END IF

    score = 0

    TALK "I need to ask you a few questions to better assist you."

    TALK "What is your company's annual revenue range?"
    TALK "1. Under $1M"
    TALK "2. $1M - $10M"
    TALK "3. $10M - $50M"
    TALK "4. Over $50M"
    revenue_answer = HEAR

    IF revenue_answer = "4" THEN
        score = score + 30
    ELSE IF revenue_answer = "3" THEN
        score = score + 20
    ELSE IF revenue_answer = "2" THEN
        score = score + 10
    ELSE
        score = score + 5
    END IF

    TALK "How many employees does your company have?"
    employees = HEAR

    IF employees > 500 THEN
        score = score + 25
    ELSE IF employees > 100 THEN
        score = score + 15
    ELSE IF employees > 20 THEN
        score = score + 10
    ELSE
        score = score + 5
    END IF

    TALK "What is your timeline for making a decision?"
    TALK "1. This month"
    TALK "2. This quarter"
    TALK "3. This year"
    TALK "4. Just researching"
    timeline = HEAR

    IF timeline = "1" THEN
        score = score + 30
    ELSE IF timeline = "2" THEN
        score = score + 20
    ELSE IF timeline = "3" THEN
        score = score + 10
    ELSE
        score = score + 0
    END IF

    TALK "Do you have budget allocated for this?"
    has_budget = HEAR

    IF has_budget = "yes" OR has_budget = "YES" OR has_budget = "Yes" THEN
        score = score + 25
    ELSE
        score = score + 5
    END IF

    lead_status = "unqualified"
    IF score >= 70 THEN
        lead_status = "hot"
    ELSE IF score >= 50 THEN
        lead_status = "warm"
    ELSE IF score >= 30 THEN
        lead_status = "cold"
    END IF

    update_lead = CREATE OBJECT
    SET update_lead.score = score
    SET update_lead.status = lead_status
    SET update_lead.qualified_at = current_time
    SET update_lead.revenue_range = revenue_answer
    SET update_lead.employees = employees
    SET update_lead.timeline = timeline
    SET update_lead.has_budget = has_budget

    SAVE_FROM_UNSTRUCTURED "leads", FORMAT update_lead AS JSON

    REMEMBER "lead_score_" + lead_id = score
    REMEMBER "lead_status_" + lead_id = lead_status

    IF lead_status = "hot" THEN
        TALK "Great! You're a perfect fit for our solution. Let me connect you with a specialist."

        notification = "Hot lead alert: " + lead.name + " from " + lead.company + " - Score: " + score
        SEND MAIL "sales@company.com", "Hot Lead Alert", notification

        CREATE_TASK "Follow up with hot lead " + lead.name, "high", user_id

    ELSE IF lead_status = "warm" THEN
        TALK "Thank you! Based on your needs, I'll have someone reach out within 24 hours."

        CREATE_TASK "Contact warm lead " + lead.name, "medium", user_id

    ELSE
        TALK "Thank you for your time. I'll send you some helpful resources via email."
    END IF

END IF

IF action = "convert" THEN
    lead = FIND "leads", "id = '" + lead_id + "'"

    IF lead = NULL THEN
        TALK "No lead found to convert."
        EXIT
    END IF

    IF lead.status = "unqualified" OR lead.status = "cold" THEN
        TALK "This lead needs to be qualified first."
        EXIT
    END IF

    account = CREATE OBJECT
    SET account.id = FORMAT GUID()
    SET account.name = lead.company
    SET account.type = "customer"
    SET account.owner_id = user_id
    SET account.created_from_lead = lead_id
    SET account.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "accounts", FORMAT account AS JSON

    contact = CREATE OBJECT
    SET contact.id = FORMAT GUID()
    SET contact.account_id = account.id
    SET contact.name = lead.name
    SET contact.email = lead.email
    SET contact.phone = lead.phone
    SET contact.primary_contact = true
    SET contact.created_from_lead = lead_id
    SET contact.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "contacts", FORMAT contact AS JSON

    opportunity = CREATE OBJECT
    SET opportunity.id = FORMAT GUID()
    SET opportunity.name = "Opportunity for " + account.name
    SET opportunity.account_id = account.id
    SET opportunity.contact_id = contact.id
    SET opportunity.stage = "qualification"
    SET opportunity.probability = 20
    SET opportunity.owner_id = user_id
    SET opportunity.lead_source = lead.source
    SET opportunity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opportunity AS JSON

    update_lead = CREATE OBJECT
    SET update_lead.status = "converted"
    SET update_lead.converted_at = current_time
    SET update_lead.converted_to_account_id = account.id

    SAVE_FROM_UNSTRUCTURED "leads", FORMAT update_lead AS JSON

    REMEMBER "account_" + account.id = account
    REMEMBER "contact_" + contact.id = contact
    REMEMBER "opportunity_" + opportunity.id = opportunity

    SET "session.account_id" = account.id
    SET "session.contact_id" = contact.id
    SET "session.opportunity_id" = opportunity.id

    TALK "Successfully converted lead to account: " + account.name

    notification = "Lead converted: " + lead.name + " to account " + account.name
    SEND MAIL user_id, "Lead Conversion", notification

    CREATE_TASK "Initial meeting with " + contact.name, "high", user_id

END IF

IF action = "follow_up" THEN
    lead = FIND "leads", "id = '" + lead_id + "'"

    IF lead = NULL THEN
        TALK "No lead found."
        EXIT
    END IF

    last_contact = GET "lead_last_contact_" + lead_id
    days_since = 0

    IF last_contact != "" THEN
        days_since = DAYS_BETWEEN(last_contact, current_time)
    END IF

    IF days_since > 7 OR last_contact = "" THEN
        subject = "Following up on your inquiry"
        message = "Hi " + lead.name + ",\n\nI wanted to follow up on your recent inquiry about our services."

        SEND MAIL lead.email, subject, message

        activity = CREATE OBJECT
        SET activity.id = FORMAT GUID()
        SET activity.type = "email"
        SET activity.subject = subject
        SET activity.lead_id = lead_id
        SET activity.created_at = current_time

        SAVE_FROM_UNSTRUCTURED "activities", FORMAT activity AS JSON

        REMEMBER "lead_last_contact_" + lead_id = current_time

        TALK "Follow-up email sent to " + lead.name
    ELSE
        TALK "Lead was contacted " + days_since + " days ago. Too soon for follow-up."
    END IF

END IF

IF action = "nurture" THEN
    leads = FIND "leads", "status = 'warm' OR status = 'cold'"

    FOR EACH lead IN leads DO
        days_old = DAYS_BETWEEN(lead.created_at, current_time)

        IF days_old = 3 THEN
            content = "5 Tips to Improve Your Business"
        ELSE IF days_old = 7 THEN
            content = "Case Study: How We Helped Similar Companies"
        ELSE IF days_old = 14 THEN
            content = "Free Consultation Offer"
        ELSE IF days_old = 30 THEN
            content = "Special Limited Time Offer"
        ELSE
            CONTINUE
        END IF

        SEND MAIL lead.email, content, "Nurture content for day " + days_old

        REMEMBER "lead_nurture_" + lead.id + "_day_" + days_old = "sent"
    END FOR

    TALK "Nurture campaign processed"
END IF
