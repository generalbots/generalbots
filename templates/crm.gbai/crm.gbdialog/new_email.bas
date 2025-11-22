' New Email Event Handler
' This script is triggered when a new email is received by the CRM system
' It handles email parsing, sender identification, automatic routing, and case creation

PARAM email_id AS STRING
PARAM from_address AS STRING
PARAM to_addresses AS ARRAY
PARAM cc_addresses AS ARRAY
PARAM subject AS STRING
PARAM body_text AS STRING
PARAM body_html AS STRING
PARAM attachments AS ARRAY
PARAM headers AS OBJECT
PARAM received_at AS DATETIME

' Initialize email context
email_context = {}
email_context.email_id = email_id
email_context.from = from_address
email_context.to = to_addresses
email_context.subject = subject
email_context.received_at = received_at

' Clean email address for lookup
clean_email = LOWERCASE(TRIM(from_address))
IF clean_email CONTAINS "<" THEN
    clean_email = EXTRACT_BETWEEN(clean_email, "<", ">")
END IF

' Look up sender in CRM
contact = FIND "contacts", "email", clean_email
lead = NULL
account = NULL

IF contact IS NULL THEN
    ' Check if sender is a lead
    lead = FIND "leads", "email", clean_email

    IF lead IS NULL THEN
        ' Create new lead from email
        lead = {}
        lead.email = clean_email
        lead.lead_source = "email"
        lead.lead_status = "new"
        lead.notes = "Auto-created from email: " + subject

        ' Try to extract name from email
        IF from_address CONTAINS "<" THEN
            display_name = TRIM(EXTRACT_BEFORE(from_address, "<"))
            IF display_name != "" THEN
                lead.contact_name = display_name
            END IF
        END IF

        ' Extract company domain
        domain = EXTRACT_AFTER(clean_email, "@")
        IF domain != "" AND NOT IS_PERSONAL_EMAIL(domain) THEN
            lead.company_name = CAPITALIZE(EXTRACT_BEFORE(domain, "."))
            lead.website = "https://" + domain
        END IF

        lead_id = SAVE "leads", lead
        email_context.lead_id = lead_id
        email_context.is_new_lead = TRUE
    ELSE
        email_context.lead_id = lead.id
    END IF
ELSE
    ' Existing contact found
    email_context.contact_id = contact.id
    email_context.account_id = contact.account_id

    IF contact.account_id IS NOT NULL THEN
        account = FIND "accounts", "id", contact.account_id
        email_context.account = account
    END IF
END IF

' Check for email thread/conversation
thread_id = NULL
IF headers.references IS NOT NULL THEN
    ' Email is part of a thread
    thread_references = SPLIT(headers.references, " ")
    FOR ref IN thread_references DO
        existing_email = FIND "email_tracking", "message_id", ref
        IF existing_email IS NOT NULL THEN
            thread_id = existing_email.thread_id OR existing_email.id
            BREAK
        END IF
    END FOR
END IF

' Analyze email content
sentiment = ANALYZE_SENTIMENT(body_text)
urgency = DETECT_URGENCY(subject + " " + body_text)
intent = CLASSIFY_INTENT(body_text)

' Determine email category
category = "general"
IF subject CONTAINS "support" OR subject CONTAINS "help" OR subject CONTAINS "issue" OR subject CONTAINS "problem" THEN
    category = "support"
ELSE IF subject CONTAINS "quote" OR subject CONTAINS "pricing" OR subject CONTAINS "cost" THEN
    category = "sales"
ELSE IF subject CONTAINS "invoice" OR subject CONTAINS "payment" OR subject CONTAINS "billing" THEN
    category = "billing"
ELSE IF subject CONTAINS "complaint" OR sentiment = "negative" THEN
    category = "complaint"
END IF

' Check for existing open case with this email
existing_case = NULL
IF contact IS NOT NULL THEN
    existing_case = FIND "cases" WHERE contact_id = contact.id AND status != "closed" ORDER BY created_at DESC LIMIT 1
ELSE IF lead IS NOT NULL THEN
    ' Check for case linked to lead's email in case description
    existing_case = FIND "cases" WHERE description CONTAINS clean_email AND status != "closed" ORDER BY created_at DESC LIMIT 1
END IF

' Determine priority
priority = "medium"
IF urgency = "high" OR subject CONTAINS "urgent" OR subject CONTAINS "asap" THEN
    priority = "high"
ELSE IF account IS NOT NULL AND (account.type = "vip" OR account.type = "enterprise") THEN
    priority = "high"
ELSE IF sentiment = "negative" AND category = "complaint" THEN
    priority = "high"
ELSE IF category = "billing" THEN
    priority = "medium"
ELSE
    priority = "low"
END IF

' Create or update case
IF existing_case IS NOT NULL THEN
    ' Add to existing case
    case_update = {}
    case_update.status = "updated"
    case_update.updated_at = NOW()

    IF priority = "high" AND existing_case.priority != "high" THEN
        case_update.priority = "high"
    END IF

    UPDATE "cases", existing_case.id, case_update

    ' Add note to case
    note = {}
    note.entity_type = "case"
    note.entity_id = existing_case.id
    note.title = "Email received: " + subject
    note.body = "From: " + from_address + "\n\n" + body_text
    note.created_by = "email_system"
    SAVE "notes", note

    email_context.case_id = existing_case.id
    email_context.case_action = "updated"
ELSE IF category = "support" OR category = "complaint" THEN
    ' Create new case
    new_case = {}
    new_case.subject = subject
    new_case.description = body_text
    new_case.status = "new"
    new_case.priority = priority
    new_case.origin = "email"
    new_case.type = category

    IF contact IS NOT NULL THEN
        new_case.contact_id = contact.id
        new_case.account_id = contact.account_id
    END IF

    ' Auto-assign based on rules
    assigned_to = NULL
    IF category = "complaint" THEN
        assigned_to = GET "config", "complaint_handler"
    ELSE IF account IS NOT NULL AND account.owner_id IS NOT NULL THEN
        assigned_to = account.owner_id
    ELSE
        ' Round-robin assignment
        assigned_to = GET_NEXT_AVAILABLE_AGENT()
    END IF

    new_case.assigned_to = assigned_to
    new_case.case_number = GENERATE_CASE_NUMBER()

    case_id = SAVE "cases", new_case
    email_context.case_id = case_id
    email_context.case_action = "created"

    ' Send notification to assigned agent
    IF assigned_to IS NOT NULL THEN
        NOTIFY AGENT assigned_to WITH "New case #" + new_case.case_number + " assigned: " + subject
    END IF
END IF

' Save email tracking record
email_record = {}
email_record.message_id = email_id
email_record.from_address = from_address
email_record.to_addresses = to_addresses
email_record.cc_addresses = cc_addresses
email_record.subject = subject
email_record.body = body_text
email_record.html_body = body_html

IF email_context.contact_id IS NOT NULL THEN
    email_record.contact_id = email_context.contact_id
END IF
IF email_context.lead_id IS NOT NULL THEN
    email_record.lead_id = email_context.lead_id
END IF
IF email_context.account_id IS NOT NULL THEN
    email_record.account_id = email_context.account_id
END IF
IF email_context.case_id IS NOT NULL THEN
    email_record.case_id = email_context.case_id
END IF

email_record.sent_at = received_at
email_record.thread_id = thread_id

SAVE "email_tracking", email_record

' Create activity record
activity = {}
activity.type = "email_received"
activity.subject = "Email: " + subject
activity.description = body_text
activity.status = "completed"
activity.email_message_id = email_id

IF email_context.contact_id IS NOT NULL THEN
    activity.contact_id = email_context.contact_id
END IF
IF email_context.lead_id IS NOT NULL THEN
    activity.lead_id = email_context.lead_id
END IF
IF email_context.account_id IS NOT NULL THEN
    activity.account_id = email_context.account_id
END IF
IF email_context.case_id IS NOT NULL THEN
    activity.case_id = email_context.case_id
END IF

activity.assigned_to = assigned_to OR GET "config", "default_email_handler"

SAVE "activities", activity

' Handle attachments
IF attachments IS NOT NULL AND LENGTH(attachments) > 0 THEN
    FOR attachment IN attachments DO
        doc = {}
        doc.name = attachment.filename
        doc.file_path = attachment.path
        doc.file_size = attachment.size
        doc.mime_type = attachment.mime_type
        doc.entity_type = "email"
        doc.entity_id = email_record.id
        doc.uploaded_by = "email_system"

        SAVE "documents", doc
    END FOR
END IF

' Auto-reply based on category and time
business_hours = GET "config", "business_hours"
current_hour = HOUR(NOW())
is_business_hours = current_hour >= business_hours.start AND current_hour <= business_hours.end

auto_reply = NULL
IF category = "support" AND email_context.case_action = "created" THEN
    IF is_business_hours THEN
        auto_reply = "Thank you for contacting support. Your case #" + new_case.case_number + " has been created and assigned to our team. We'll respond within 2 business hours."
    ELSE
        auto_reply = "Thank you for contacting support. Your case #" + new_case.case_number + " has been created. Our business hours are " + business_hours.start + " to " + business_hours.end + ". We'll respond as soon as possible."
    END IF
ELSE IF category = "sales" THEN
    auto_reply = "Thank you for your interest! A sales representative will contact you within 1 business day."
ELSE IF category = "complaint" THEN
    auto_reply = "We've received your message and take your concerns seriously. A manager will contact you within 4 hours."
END IF

IF auto_reply IS NOT NULL AND NOT IS_AUTOREPLY(headers) THEN
    SEND EMAIL TO from_address SUBJECT "RE: " + subject BODY auto_reply
END IF

' Update lead score if applicable
IF lead IS NOT NULL THEN
    score_increase = 0
    IF category = "sales" THEN
        score_increase = 10
    ELSE IF intent = "purchase_intent" THEN
        score_increase = 15
    ELSE
        score_increase = 5
    END IF

    UPDATE "leads", lead.id, "score", lead.score + score_increase

    ' Check if lead should be converted
    IF lead.score > 50 AND category = "sales" THEN
        TRIGGER "lead_qualification", lead.id
    END IF
END IF

' Log email processing
LOG "email_processed", {
    "email_id": email_id,
    "from": from_address,
    "category": category,
    "priority": priority,
    "sentiment": sentiment,
    "case_action": email_context.case_action,
    "case_id": email_context.case_id,
    "is_new_lead": email_context.is_new_lead,
    "auto_replied": auto_reply IS NOT NULL,
    "timestamp": NOW()
}

' Return processing result
RETURN email_context
