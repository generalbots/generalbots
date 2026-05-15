PARAM action AS STRING LIKE "capture" DESCRIPTION "Action: capture, qualify, convert, follow_up, nurture"
PARAM lead_data AS OBJECT LIKE "{name: 'John', email: 'john@company.com'}" DESCRIPTION "Lead information object"

DESCRIPTION "Manage leads through the sales pipeline - capture, qualify, convert, follow up, and nurture"

lead_id = GET "session.lead_id"
user_id = GET "session.user_id"

IF action = "capture" THEN
    WITH new_lead
        id = FORMAT(GUID())
        name = lead_data.name
        email = lead_data.email
        phone = lead_data.phone
        company = lead_data.company
        source = lead_data.source
        status = "new"
        score = 0
        created_at = NOW()
        assigned_to = user_id
    END WITH

    SAVE "leads.csv", new_lead

    SET "session.lead_id", new_lead.id
    SET "session.lead_status", "captured"
    SET BOT MEMORY "lead_" + new_lead.id, new_lead.name

    TALK "Thank you " + new_lead.name + "! I've captured your information."
    RETURN new_lead.id
END IF

IF action = "qualify" THEN
    lead = FIND "leads.csv", "id = '" + lead_id + "'"

    IF NOT lead THEN
        TALK "No lead found to qualify."
        RETURN NULL
    END IF

    score = 0

    TALK "I need to ask you a few questions to better assist you."

    TALK "What is your company's annual revenue range?"
    ADD SUGGESTION "1" AS "Under $1M"
    ADD SUGGESTION "2" AS "$1M - $10M"
    ADD SUGGESTION "3" AS "$10M - $50M"
    ADD SUGGESTION "4" AS "Over $50M"
    HEAR revenue_answer AS INTEGER

    IF revenue_answer = 4 THEN
        score = score + 30
    ELSE IF revenue_answer = 3 THEN
        score = score + 20
    ELSE IF revenue_answer = 2 THEN
        score = score + 10
    ELSE
        score = score + 5
    END IF

    TALK "How many employees does your company have?"
    HEAR employees AS INTEGER

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
    ADD SUGGESTION "1" AS "This month"
    ADD SUGGESTION "2" AS "This quarter"
    ADD SUGGESTION "3" AS "This year"
    ADD SUGGESTION "4" AS "Just researching"
    HEAR timeline AS INTEGER

    IF timeline = 1 THEN
        score = score + 30
    ELSE IF timeline = 2 THEN
        score = score + 20
    ELSE IF timeline = 3 THEN
        score = score + 10
    END IF

    TALK "Do you have budget allocated for this?"
    HEAR has_budget AS BOOLEAN

    IF has_budget THEN
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

    WITH qualification
        lead_id = lead_id
        score = score
        status = lead_status
        qualified_at = NOW()
        revenue_range = revenue_answer
        employees = employees
        timeline = timeline
        has_budget = has_budget
    END WITH

    SAVE "lead_qualification.csv", qualification

    SET BOT MEMORY "lead_score_" + lead_id, score
    SET BOT MEMORY "lead_status_" + lead_id, lead_status

    IF lead_status = "hot" THEN
        TALK "Great! You're a perfect fit for our solution. Let me connect you with a specialist."
        SEND EMAIL "sales@company.com", "Hot Lead Alert", "Hot lead alert: " + lead.name + " from " + lead.company + " - Score: " + score
        CREATE_TASK "Follow up with hot lead " + lead.name, "high", user_id
    ELSE IF lead_status = "warm" THEN
        TALK "Thank you! Based on your needs, I'll have someone reach out within 24 hours."
        CREATE_TASK "Contact warm lead " + lead.name, "medium", user_id
    ELSE
        TALK "Thank you for your time. I'll send you some helpful resources via email."
    END IF

    RETURN score
END IF

IF action = "convert" THEN
    lead = FIND "leads.csv", "id = '" + lead_id + "'"

    IF NOT lead THEN
        TALK "No lead found to convert."
        RETURN NULL
    END IF

    IF lead.status = "unqualified" OR lead.status = "cold" THEN
        TALK "This lead needs to be qualified first."
        RETURN NULL
    END IF

    WITH account
        id = FORMAT(GUID())
        name = lead.company
        type = "customer"
        owner_id = user_id
        created_from_lead = lead_id
        created_at = NOW()
    END WITH

    SAVE "accounts.csv", account

    WITH contact
        id = FORMAT(GUID())
        account_id = account.id
        name = lead.name
        email = lead.email
        phone = lead.phone
        primary_contact = true
        created_from_lead = lead_id
        created_at = NOW()
    END WITH

    SAVE "contacts.csv", contact

    WITH opportunity
        id = FORMAT(GUID())
        name = "Opportunity for " + account.name
        account_id = account.id
        contact_id = contact.id
        stage = "qualification"
        probability = 20
        owner_id = user_id
        lead_source = lead.source
        created_at = NOW()
    END WITH

    SAVE "opportunities.csv", opportunity

    UPDATE "leads.csv" SET status = "converted", converted_at = NOW(), converted_to_account_id = account.id WHERE id = lead_id

    SET BOT MEMORY "account_" + account.id, account.name
    SET "session.account_id", account.id
    SET "session.contact_id", contact.id
    SET "session.opportunity_id", opportunity.id

    TALK "Lead converted to account: " + account.name

    SEND EMAIL user_id, "Lead Conversion", "Lead converted: " + lead.name + " to account " + account.name
    CREATE_TASK "Initial meeting with " + contact.name, "high", user_id

    RETURN account.id
END IF

IF action = "follow_up" THEN
    lead = FIND "leads.csv", "id = '" + lead_id + "'"

    IF NOT lead THEN
        TALK "No lead found."
        RETURN NULL
    END IF

    last_contact = GET BOT MEMORY "lead_last_contact_" + lead_id
    days_since = 0

    IF last_contact THEN
        days_since = DATEDIFF(last_contact, NOW(), "day")
    END IF

    IF days_since > 7 OR NOT last_contact THEN
        subject = "Following up on your inquiry"
        message = "Hi " + lead.name + ",\n\nI wanted to follow up on your recent inquiry about our services."

        SEND EMAIL lead.email, subject, message

        WITH activity
            id = FORMAT(GUID())
            type = "email"
            subject = subject
            lead_id = lead_id
            created_at = NOW()
        END WITH

        SAVE "activities.csv", activity

        SET BOT MEMORY "lead_last_contact_" + lead_id, NOW()

        TALK "Follow-up email sent to " + lead.name
        RETURN "sent"
    ELSE
        TALK "Lead was contacted " + days_since + " days ago. Too soon for follow-up."
        RETURN "skipped"
    END IF
END IF

IF action = "nurture" THEN
    leads = FIND "leads.csv", "status = 'warm' OR status = 'cold'"

    count = 0
    FOR EACH lead IN leads
        days_old = DATEDIFF(lead.created_at, NOW(), "day")

        content = NULL
        IF days_old = 3 THEN
            content = "5 Tips to Improve Your Business"
        ELSE IF days_old = 7 THEN
            content = "Case Study: How We Helped Similar Companies"
        ELSE IF days_old = 14 THEN
            content = "Free Consultation Offer"
        ELSE IF days_old = 30 THEN
            content = "Special Limited Time Offer"
        END IF

        IF content THEN
            SEND EMAIL lead.email, content, "Nurture content for day " + days_old
            SET BOT MEMORY "lead_nurture_" + lead.id + "_day_" + days_old, "sent"
            count = count + 1
        END IF
    NEXT

    TALK "Nurture campaign processed: " + count + " emails sent"
    RETURN count
END IF

TALK "Unknown action: " + action
RETURN NULL
