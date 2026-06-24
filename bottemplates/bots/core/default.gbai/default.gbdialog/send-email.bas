PARAM to_email AS EMAIL LIKE "user@example.com" DESCRIPTION "Recipient email address"
PARAM subject AS STRING LIKE "Important Message" DESCRIPTION "Email subject line"
PARAM body AS STRING LIKE "Hello, this is the email content." DESCRIPTION "Email body content"
PARAM from_email AS EMAIL LIKE "noreply@company.com" DESCRIPTION "Sender email address" OPTIONAL

DESCRIPTION "Send an email to any recipient with subject and body"

IF NOT from_email THEN
    from_email = "noreply@pragmatismo.com.br"
END IF

WITH email_data
    to = to_email
    from = from_email
    subject = subject
    body = body
    timestamp = NOW()
END WITH

SEND EMAIL to_email, subject, body

SAVE "email_log.csv", email_data

TALK "Email sent to " + to_email

RETURN email_data
