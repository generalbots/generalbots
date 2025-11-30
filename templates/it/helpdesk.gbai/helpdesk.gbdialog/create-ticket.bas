PARAM description AS STRING LIKE "My computer won't turn on" DESCRIPTION "Description of the IT issue or problem"
PARAM category AS STRING LIKE "hardware" DESCRIPTION "Category: hardware, software, network, email, account, other" OPTIONAL
PARAM priority AS STRING LIKE "medium" DESCRIPTION "Priority level: critical, high, medium, low" OPTIONAL

DESCRIPTION "Create a new IT support ticket with issue details and priority assignment"

useremail = GET "session.user_email"
username = GET "session.user_name"

IF NOT category THEN
    category = "other"
END IF

IF NOT priority THEN
    priority = "medium"
END IF

ticketnumber = "TKT" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))

slahours = 48
IF priority = "critical" THEN
    slahours = 4
ELSE IF priority = "high" THEN
    slahours = 24
ELSE IF priority = "low" THEN
    slahours = 72
END IF

assignedteam = "general-support"
IF category = "network" THEN
    assignedteam = "network-team"
ELSE IF category = "hardware" THEN
    assignedteam = "desktop-support"
ELSE IF category = "email" THEN
    assignedteam = "messaging-team"
ELSE IF category = "account" THEN
    assignedteam = "identity-team"
END IF

WITH ticket
    number = ticketnumber
    desc = description
    cat = category
    prio = priority
    status = "new"
    userEmail = useremail
    userName = username
    team = assignedteam
    created = NOW()
END WITH

SAVE "tickets.csv", ticket

SET BOT MEMORY "last_ticket", ticketnumber

subject = "Ticket Created: " + ticketnumber
message = "Hello " + username + ",\n\nYour support ticket has been created.\n\nTicket: " + ticketnumber + "\nCategory: " + category + "\nPriority: " + priority + "\nExpected Response: Within " + slahours + " hours\n\nIssue:\n" + description

SEND EMAIL useremail, subject, message

teamsubject = "[" + priority + "] New Ticket: " + ticketnumber
teammessage = "New ticket from " + username + " (" + useremail + ")\n\nCategory: " + category + "\nPriority: " + priority + "\n\nDescription:\n" + description

SEND EMAIL assignedteam + "@company.com", teamsubject, teammessage

TALK "Ticket created: " + ticketnumber
TALK "Category: " + category
TALK "Priority: " + priority
TALK "Assigned Team: " + assignedteam
TALK "Expected Response: Within " + slahours + " hours"

RETURN ticketnumber
