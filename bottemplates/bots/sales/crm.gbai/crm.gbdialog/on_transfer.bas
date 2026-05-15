' On Transfer Event Handler
' This script is triggered when a conversation is transferred between agents or bots
' It handles context preservation, handoff notifications, and transfer logging

PARAM session_id AS STRING
PARAM from_agent AS STRING
PARAM to_agent AS STRING
PARAM transfer_reason AS STRING
PARAM transfer_type AS STRING  ' bot_to_human, human_to_bot, human_to_human
PARAM context AS OBJECT

' Get session context from cache
session_context = CACHE GET "session:" + session_id
IF session_context IS NULL THEN
    session_context = {}
    session_context.session_id = session_id
END IF

' Update session context with transfer info
session_context.last_transfer = NOW()
session_context.transfer_count = (session_context.transfer_count OR 0) + 1
session_context.current_agent = to_agent
session_context.transfer_history = session_context.transfer_history OR []

' Add to transfer history
transfer_record = {}
transfer_record.from = from_agent
transfer_record.to = to_agent
transfer_record.reason = transfer_reason
transfer_record.type = transfer_type
transfer_record.timestamp = NOW()
transfer_record.context_preserved = context

APPEND session_context.transfer_history, transfer_record

' Get user information
user = NULL
IF session_context.contact_id IS NOT NULL THEN
    user = FIND "contacts", "id", session_context.contact_id
ELSE IF session_context.lead_id IS NOT NULL THEN
    lead = FIND "leads", "id", session_context.lead_id
    user = {"first_name": lead.contact_name, "email": lead.email}
END IF

' Create activity for transfer
activity = {}
activity.type = "transfer"
activity.subject = "Conversation transferred from " + from_agent + " to " + to_agent
activity.description = "Transfer reason: " + transfer_reason + "\nTransfer type: " + transfer_type
activity.status = "completed"
activity.assigned_to = to_agent
activity.created_by = from_agent

IF session_context.contact_id IS NOT NULL THEN
    activity.contact_id = session_context.contact_id
END IF
IF session_context.lead_id IS NOT NULL THEN
    activity.lead_id = session_context.lead_id
END IF
IF session_context.case_id IS NOT NULL THEN
    activity.case_id = session_context.case_id
END IF
IF session_context.opportunity_id IS NOT NULL THEN
    activity.opportunity_id = session_context.opportunity_id
END IF

SAVE "activities", activity

' Handle different transfer types
IF transfer_type = "bot_to_human" THEN
    ' Bot to Human handoff
    SEND MESSAGE "I'm transferring you to " + to_agent + " who will be better able to assist you."

    ' Prepare summary for human agent
    summary = "=== Transfer Summary ===\n"
    summary = summary + "Customer: " + (user.first_name OR "Unknown") + "\n"
    summary = summary + "Email: " + (user.email OR "Not provided") + "\n"
    summary = summary + "Transfer Reason: " + transfer_reason + "\n"

    ' Add conversation history
    IF context.conversation_history IS NOT NULL THEN
        summary = summary + "\n=== Recent Conversation ===\n"
        FOR message IN context.conversation_history LAST 10 DO
            summary = summary + message.sender + ": " + message.text + "\n"
        END FOR
    END IF

    ' Add open issues
    IF session_context.case_id IS NOT NULL THEN
        case = FIND "cases", "id", session_context.case_id
        summary = summary + "\n=== Open Case ===\n"
        summary = summary + "Case #: " + case.case_number + "\n"
        summary = summary + "Subject: " + case.subject + "\n"
        summary = summary + "Priority: " + case.priority + "\n"
    END IF

    ' Send summary to human agent
    NOTIFY AGENT to_agent WITH summary

    ' Update case if exists
    IF session_context.case_id IS NOT NULL THEN
        UPDATE "cases", session_context.case_id, {
            "assigned_to": to_agent,
            "status": "in_progress",
            "escalated_to": to_agent
        }
    END IF

ELSE IF transfer_type = "human_to_bot" THEN
    ' Human to Bot handoff
    SEND MESSAGE "You've been transferred back to the automated assistant. How can I help you?"

    ' Reset bot context
    session_context.bot_context = {}
    session_context.bot_context.resumed_at = NOW()
    session_context.bot_context.previous_human_agent = from_agent

ELSE IF transfer_type = "human_to_human" THEN
    ' Human to Human handoff
    SEND MESSAGE to_agent + " will now assist you with your inquiry."

    ' Notify new agent
    notification = "You've received a transfer from " + from_agent + "\n"
    notification = notification + "Customer: " + (user.first_name OR "Unknown") + "\n"
    notification = notification + "Reason: " + transfer_reason + "\n"
    notification = notification + "Please review the conversation history."

    NOTIFY AGENT to_agent WITH notification

    ' Update assignment in all related entities
    IF session_context.case_id IS NOT NULL THEN
        UPDATE "cases", session_context.case_id, "assigned_to", to_agent
    END IF
    IF session_context.opportunity_id IS NOT NULL THEN
        UPDATE "opportunities", session_context.opportunity_id, "owner_id", to_agent
    END IF
END IF

' Check if this is a VIP customer
IF user IS NOT NULL AND user.account_id IS NOT NULL THEN
    account = FIND "accounts", "id", user.account_id
    IF account.type = "vip" OR account.type = "enterprise" THEN
        NOTIFY AGENT to_agent WITH "⚠️ VIP Customer Alert: " + account.name

        ' Add VIP handling
        session_context.is_vip = TRUE
        session_context.account_tier = account.type
    END IF
END IF

' Update session cache
CACHE SET "session:" + session_id, session_context, 3600

' Set up quality check
IF transfer_type = "bot_to_human" THEN
    SCHEDULE IN 600 SECONDS DO
        ' After 10 minutes, check satisfaction
        IF IS_ACTIVE(session_id) THEN
            satisfaction_check = {}
            satisfaction_check.session_id = session_id
            satisfaction_check.transfer_id = transfer_record.id
            satisfaction_check.checked_at = NOW()

            SEND MESSAGE "Quick question: Has " + to_agent + " been able to help you with your issue? (Yes/No)"

            WAIT FOR RESPONSE AS response TIMEOUT 60

            IF response IS NOT NULL THEN
                satisfaction_check.response = response
                SAVE "transfer_satisfaction", satisfaction_check

                IF response CONTAINS "no" OR response CONTAINS "not" THEN
                    ESCALATE TO SUPERVISOR
                END IF
            END IF
        END IF
    END SCHEDULE
END IF

' Log transfer metrics
LOG "conversation_transfer", {
    "session_id": session_id,
    "from_agent": from_agent,
    "to_agent": to_agent,
    "transfer_type": transfer_type,
    "transfer_reason": transfer_reason,
    "customer_type": user IS NOT NULL ? "existing" : "new",
    "transfer_number": session_context.transfer_count,
    "timestamp": NOW()
}

' Send transfer confirmation
confirmation = {}
confirmation.success = TRUE
confirmation.message = "Transfer completed successfully"
confirmation.new_agent = to_agent
confirmation.session_context = session_context

RETURN confirmation
