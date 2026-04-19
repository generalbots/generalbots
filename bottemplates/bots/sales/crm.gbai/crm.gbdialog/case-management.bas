PARAM action AS STRING
PARAM case_data AS OBJECT

user_id = GET "session.user_id"
case_id = GET "session.case_id"
contact_id = GET "session.contact_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF action = "create" THEN
    subject = GET "case_data.subject"
    description = GET "case_data.description"
    priority = GET "case_data.priority"

    IF subject = "" THEN
        TALK "What is the issue you're experiencing?"
        subject = HEAR
    END IF

    IF description = "" THEN
        TALK "Please describe the issue in detail:"
        description = HEAR
    END IF

    IF priority = "" THEN
        TALK "How urgent is this? (low/medium/high/critical)"
        priority = HEAR
    END IF

    case_number = "CS-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

    new_case = CREATE OBJECT
    SET new_case.id = FORMAT GUID()
    SET new_case.case_number = case_number
    SET new_case.subject = subject
    SET new_case.description = description
    SET new_case.status = "new"
    SET new_case.priority = priority
    SET new_case.contact_id = contact_id
    SET new_case.created_at = current_time
    SET new_case.assigned_to = user_id

    SAVE_FROM_UNSTRUCTURED "cases", FORMAT new_case AS JSON

    SET "session.case_id" = new_case.id
    REMEMBER "case_" + new_case.id = new_case

    TALK "Case " + case_number + " created successfully."

    IF priority = "critical" OR priority = "high" THEN
        notification = "URGENT: New " + priority + " priority case: " + case_number + " - " + subject
        SEND MAIL "support-manager@company.com", "Urgent Case", notification

        CREATE_TASK "Resolve case " + case_number + " immediately", "critical", user_id
    ELSE
        CREATE_TASK "Review case " + case_number, priority, user_id
    END IF

    activity = CREATE OBJECT
    SET activity.type = "case_created"
    SET activity.case_id = new_case.id
    SET activity.description = "Case created: " + subject
    SET activity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "activities", FORMAT activity AS JSON

END IF

IF action = "update_status" THEN
    IF case_id = "" THEN
        TALK "Enter case number:"
        case_number = HEAR
        case = FIND "cases", "case_number = '" + case_number + "'"
        IF case != NULL THEN
            case_id = case.id
        ELSE
            TALK "Case not found."
            EXIT
        END IF
    END IF

    case = FIND "cases", "id = '" + case_id + "'"

    IF case = NULL THEN
        TALK "Case not found."
        EXIT
    END IF

    TALK "Current status: " + case.status
    TALK "Select new status:"
    TALK "1. New"
    TALK "2. In Progress"
    TALK "3. Waiting on Customer"
    TALK "4. Waiting on Vendor"
    TALK "5. Escalated"
    TALK "6. Resolved"
    TALK "7. Closed"

    status_choice = HEAR

    new_status = ""
    IF status_choice = "1" THEN
        new_status = "new"
    ELSE IF status_choice = "2" THEN
        new_status = "in_progress"
    ELSE IF status_choice = "3" THEN
        new_status = "waiting_customer"
    ELSE IF status_choice = "4" THEN
        new_status = "waiting_vendor"
    ELSE IF status_choice = "5" THEN
        new_status = "escalated"
    ELSE IF status_choice = "6" THEN
        new_status = "resolved"
    ELSE IF status_choice = "7" THEN
        new_status = "closed"
    END IF

    old_status = case.status
    case.status = new_status
    case.updated_at = current_time

    IF new_status = "resolved" OR new_status = "closed" THEN
        case.resolved_at = current_time

        TALK "Please provide resolution details:"
        resolution = HEAR
        case.resolution = resolution
    END IF

    IF new_status = "escalated" THEN
        TALK "Reason for escalation:"
        escalation_reason = HEAR
        case.escalation_reason = escalation_reason

        notification = "Case Escalated: " + case.case_number + " - " + case.subject + "\nReason: " + escalation_reason
        SEND MAIL "support-manager@company.com", "Case Escalation", notification
    END IF

    SAVE_FROM_UNSTRUCTURED "cases", FORMAT case AS JSON

    activity = CREATE OBJECT
    SET activity.type = "status_change"
    SET activity.case_id = case_id
    SET activity.description = "Status changed from " + old_status + " to " + new_status
    SET activity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "activities", FORMAT activity AS JSON

    TALK "Case status updated to " + new_status

    IF new_status = "resolved" THEN
        contact = FIND "contacts", "id = '" + case.contact_id + "'"
        IF contact != NULL AND contact.email != "" THEN
            subject = "Case " + case.case_number + " Resolved"
            message = "Your case has been resolved.\n\nResolution: " + resolution + "\n\nThank you for your patience."
            SEND MAIL contact.email, subject, message
        END IF
    END IF

END IF

IF action = "add_note" THEN
    IF case_id = "" THEN
        TALK "Enter case number:"
        case_number = HEAR
        case = FIND "cases", "case_number = '" + case_number + "'"
        IF case != NULL THEN
            case_id = case.id
        ELSE
            TALK "Case not found."
            EXIT
        END IF
    END IF

    TALK "Enter your note:"
    note_text = HEAR

    note = CREATE OBJECT
    SET note.id = FORMAT GUID()
    SET note.entity_type = "case"
    SET note.entity_id = case_id
    SET note.body = note_text
    SET note.created_by = user_id
    SET note.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "notes", FORMAT note AS JSON

    TALK "Note added to case."

END IF

IF action = "search" THEN
    TALK "Search by:"
    TALK "1. Case Number"
    TALK "2. Subject"
    TALK "3. Contact Email"
    TALK "4. Status"

    search_type = HEAR

    IF search_type = "1" THEN
        TALK "Enter case number:"
        search_term = HEAR
        cases = FIND "cases", "case_number = '" + search_term + "'"
    ELSE IF search_type = "2" THEN
        TALK "Enter subject keywords:"
        search_term = HEAR
        cases = FIND "cases", "subject LIKE '%" + search_term + "%'"
    ELSE IF search_type = "3" THEN
        TALK "Enter contact email:"
        search_term = HEAR
        contact = FIND "contacts", "email = '" + search_term + "'"
        IF contact != NULL THEN
            cases = FIND "cases", "contact_id = '" + contact.id + "'"
        END IF
    ELSE IF search_type = "4" THEN
        TALK "Enter status (new/in_progress/resolved/closed):"
        search_term = HEAR
        cases = FIND "cases", "status = '" + search_term + "'"
    END IF

    IF cases = NULL THEN
        TALK "No cases found."
    ELSE
        TALK "Found cases:"
        FOR EACH case IN cases DO
            TALK case.case_number + " - " + case.subject + " (" + case.status + ")"
        END FOR
    END IF

END IF

IF action = "sla_check" THEN
    cases = FIND "cases", "status != 'closed' AND status != 'resolved'"

    breached_count = 0
    warning_count = 0

    FOR EACH case IN cases DO
        hours_open = HOURS_BETWEEN(case.created_at, current_time)

        sla_hours = 24
        IF case.priority = "critical" THEN
            sla_hours = 2
        ELSE IF case.priority = "high" THEN
            sla_hours = 4
        ELSE IF case.priority = "medium" THEN
            sla_hours = 8
        END IF

        IF hours_open > sla_hours THEN
            breached_count = breached_count + 1

            notification = "SLA BREACH: Case " + case.case_number + " - Open for " + hours_open + " hours"
            SEND MAIL "support-manager@company.com", "SLA Breach Alert", notification

            case.sla_breached = true
            SAVE_FROM_UNSTRUCTURED "cases", FORMAT case AS JSON

        ELSE IF hours_open > sla_hours * 0.8 THEN
            warning_count = warning_count + 1
        END IF
    END FOR

    TALK "SLA Status:"
    TALK "Breached: " + breached_count + " cases"
    TALK "Warning: " + warning_count + " cases"

    IF breached_count > 0 THEN
        CREATE_TASK "Review SLA breached cases immediately", "critical", user_id
    END IF

END IF

IF action = "daily_report" THEN
    new_cases = FIND "cases", "DATE(created_at) = DATE('" + current_time + "')"
    resolved_cases = FIND "cases", "DATE(resolved_at) = DATE('" + current_time + "')"
    open_cases = FIND "cases", "status != 'closed' AND status != 'resolved'"

    new_count = 0
    resolved_count = 0
    open_count = 0

    FOR EACH case IN new_cases DO
        new_count = new_count + 1
    END FOR

    FOR EACH case IN resolved_cases DO
        resolved_count = resolved_count + 1
    END FOR

    FOR EACH case IN open_cases DO
        open_count = open_count + 1
    END FOR

    report = "DAILY CASE REPORT - " + current_time + "\n"
    report = report + "================================\n"
    report = report + "New Cases Today: " + new_count + "\n"
    report = report + "Resolved Today: " + resolved_count + "\n"
    report = report + "Currently Open: " + open_count + "\n\n"

    report = report + "Open Cases by Priority:\n"

    critical_cases = FIND "cases", "status != 'closed' AND status != 'resolved' AND priority = 'critical'"
    high_cases = FIND "cases", "status != 'closed' AND status != 'resolved' AND priority = 'high'"
    medium_cases = FIND "cases", "status != 'closed' AND status != 'resolved' AND priority = 'medium'"
    low_cases = FIND "cases", "status != 'closed' AND status != 'resolved' AND priority = 'low'"

    critical_count = 0
    high_count = 0
    medium_count = 0
    low_count = 0

    FOR EACH case IN critical_cases DO
        critical_count = critical_count + 1
    END FOR

    FOR EACH case IN high_cases DO
        high_count = high_count + 1
    END FOR

    FOR EACH case IN medium_cases DO
        medium_count = medium_count + 1
    END FOR

    FOR EACH case IN low_cases DO
        low_count = low_count + 1
    END FOR

    report = report + "Critical: " + critical_count + "\n"
    report = report + "High: " + high_count + "\n"
    report = report + "Medium: " + medium_count + "\n"
    report = report + "Low: " + low_count + "\n"

    SEND MAIL "support-manager@company.com", "Daily Case Report", report

    TALK "Daily report sent to management."

END IF
