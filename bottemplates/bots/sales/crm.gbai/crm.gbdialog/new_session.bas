' New Session Event Handler
' This script is triggered when a new session starts with the bot
' It handles initial setup, user identification, and welcome messages

PARAM session_id AS STRING
PARAM user_id AS STRING
PARAM channel AS STRING
PARAM metadata AS OBJECT

' Initialize session context
SET session_context = {}
SET session_context.id = session_id
SET session_context.user_id = user_id
SET session_context.channel = channel
SET session_context.start_time = NOW()
SET session_context.metadata = metadata

' Check if user exists in CRM
user = FIND "contacts", "email", user_id
IF user IS NULL THEN
    user = FIND "contacts", "phone", user_id
END IF

' Create activity record for new session
activity = {}
activity.type = "session_start"
activity.subject = "New " + channel + " session initiated"
activity.description = "User connected via " + channel + " at " + NOW()
activity.status = "open"
activity.assigned_to = GET "config", "default_agent"

IF user IS NOT NULL THEN
    ' Existing user found
    activity.contact_id = user.id
    activity.account_id = user.account_id

    ' Get user's recent interactions
    recent_activities = FIND ALL "activities" WHERE contact_id = user.id ORDER BY created_at DESC LIMIT 5

    ' Check for open cases
    open_cases = FIND ALL "cases" WHERE contact_id = user.id AND status != "closed"

    ' Set personalized greeting
    IF open_cases.count > 0 THEN
        greeting = "Welcome back, " + user.first_name + "! I see you have an open support case. Would you like to continue with that?"
        SET session_context.has_open_case = TRUE
        SET session_context.case_id = open_cases[0].id
    ELSE IF recent_activities.count > 0 AND DAYS_BETWEEN(recent_activities[0].created_at, NOW()) < 7 THEN
        greeting = "Hi " + user.first_name + "! Good to see you again. How can I help you today?"
    ELSE
        greeting = "Welcome back, " + user.first_name + "! It's been a while. How can I assist you today?"
    END IF

    ' Update contact's last interaction
    UPDATE "contacts", user.id, "last_interaction", NOW()

ELSE
    ' New user - create lead
    lead = {}
    lead.lead_source = channel
    lead.lead_status = "new"
    lead.notes = "Auto-created from " + channel + " session"

    ' Try to extract contact info from metadata
    IF metadata.email IS NOT NULL THEN
        lead.email = metadata.email
    END IF

    IF metadata.phone IS NOT NULL THEN
        lead.phone = metadata.phone
    END IF

    IF metadata.name IS NOT NULL THEN
        lead.contact_name = metadata.name
    END IF

    ' Save lead
    lead_id = SAVE "leads", lead
    activity.lead_id = lead_id

    SET session_context.is_new_lead = TRUE
    SET session_context.lead_id = lead_id

    greeting = "Hello! Welcome to our service. I'm here to help you. May I have your name to better assist you?"
END IF

' Save activity
SAVE "activities", activity

' Store session context
CACHE SET "session:" + session_id, session_context, 3600

' Send greeting
SEND MESSAGE greeting

' Check business hours
business_hours = GET "config", "business_hours"
current_hour = HOUR(NOW())

IF current_hour < business_hours.start OR current_hour > business_hours.end THEN
    SEND MESSAGE "Please note that our business hours are " + business_hours.start + " to " + business_hours.end + ". You can still leave a message and we'll get back to you as soon as possible."
END IF

' Set up session monitoring
SCHEDULE IN 300 SECONDS DO
    ' Check if session is still active after 5 minutes
    IF IS_ACTIVE(session_id) THEN
        ' Session still active, check if user needs help
        last_message_time = GET_LAST_MESSAGE_TIME(session_id)
        IF SECONDS_BETWEEN(last_message_time, NOW()) > 180 THEN
            SEND MESSAGE "I'm still here if you need any assistance. Just let me know how I can help!"
        END IF
    END IF
END SCHEDULE

' Log session start for analytics
LOG "session_start", {
    "session_id": session_id,
    "user_id": user_id,
    "channel": channel,
    "user_type": user IS NOT NULL ? "existing" : "new",
    "timestamp": NOW()
}

' Return session context
RETURN session_context
