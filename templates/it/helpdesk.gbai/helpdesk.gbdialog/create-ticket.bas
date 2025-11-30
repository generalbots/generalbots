PARAM description AS STRING LIKE "My computer won't turn on" DESCRIPTION "Description of the IT issue or problem"
PARAM category AS STRING LIKE "hardware" DESCRIPTION "Optional: Category - hardware, software, network, email, account, other"
PARAM priority AS STRING LIKE "medium" DESCRIPTION "Optional: Priority level - critical, high, medium, low"

DESCRIPTION "Creates a new IT support ticket. Gathers information about the issue and creates the ticket record."

' Validate description
IF description = "" THEN
    TALK "Please describe the issue you're experiencing."
    description = HEAR
END IF

IF description = "" THEN
    TALK "I need a description to create a ticket."
    RETURN
END IF

' Get user info
let useremail = GET "session.user_email"
let username = GET "session.user_name"

IF useremail = "" THEN
    TALK "What is your email address?"
    useremail = HEAR
END IF

IF username = "" THEN
    TALK "And your name?"
    username = HEAR
END IF

' Set defaults
IF category = "" THEN
    category = "other"
END IF

IF priority = "" THEN
    priority = "medium"
END IF

' Generate ticket number
let ticketnumber = "TKT" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

' Determine SLA hours based on priority
let slahours = 48
IF priority = "critical" THEN
    slahours = 4
ELSE IF priority = "high" THEN
    slahours = 24
ELSE IF priority = "low" THEN
    slahours = 72
END IF

' Save ticket
let status = "new"
let createdat = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"
let assignedteam = "general-support"

IF category = "network" THEN
    assignedteam = "network-team"
ELSE IF category = "hardware" THEN
    assignedteam = "desktop-support"
ELSE IF category = "email" THEN
    assignedteam = "messaging-team"
END IF

SAVE "tickets.csv", ticketnumber, description, category, priority, status, useremail, username, assignedteam, createdat

' Store in bot memory
SET BOT MEMORY "last_ticket", ticketnumber

' Send confirmation email
let subject = "Ticket Created: " + ticketnumber
let message = "Hello " + username + ",\n\n"
message = message + "Your support ticket has been created.\n\n"
message = message + "Ticket Number: " + ticketnumber + "\n"
message = message + "Category: " + category + "\n"
message = message + "Priority: " + priority + "\n"
message = message + "Expected Response: Within " + slahours + " hours\n\n"
message = message + "Issue:\n" + description + "\n\n"
message = message + "Best regards,\nIT Helpdesk"

SEND MAIL useremail, subject, message

' Notify support team
let teamsubject = "[" + priority + "] New Ticket: " + ticketnumber
let teammessage = "New ticket from " + username + " (" + useremail + ")\n\n"
teammessage = teammessage + "Category: " + category + "\n"
teammessage = teammessage + "Priority: " + priority + "\n\n"
teammessage = teammessage + "Description:\n" + description

SEND MAIL assignedteam + "@company.com", teamsubject, teammessage

' Respond to user
TALK "✅ **Ticket Created Successfully!**"
TALK ""
TALK "**Ticket Number:** " + ticketnumber
TALK "**Category:** " + category
TALK "**Priority:** " + priority
TALK "**Assigned Team:** " + assignedteam
TALK ""
TALK "**Expected Response:** Within " + slahours + " hours"
TALK ""
TALK "📧 A confirmation email has been sent to " + useremail
TALK ""
TALK "You can check your ticket status anytime by saying **check ticket " + ticketnumber + "**"
