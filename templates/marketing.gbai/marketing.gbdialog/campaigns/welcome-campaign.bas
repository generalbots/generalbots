' General Bots - Welcome Campaign Template
' Automated drip sequence for new leads/subscribers

PARAM lead_email AS string LIKE "user@example.com"
PARAM lead_name AS string LIKE "John Doe"
PARAM lead_source AS string LIKE "website"

DESCRIPTION "Welcome campaign drip sequence for new leads"

' Validate input
IF ISEMPTY(lead_email) THEN
    THROW "Email is required for welcome campaign"
END IF

IF ISEMPTY(lead_name) THEN
    lead_name = "Friend"
END IF

TALK "Starting welcome campaign for: " + lead_email

' Step 1: Immediate Welcome Email
WITH variables
    .name = lead_name
    .source = lead_source
    .date = TODAY()
END WITH

result = SEND TEMPLATE "welcome-email-1",