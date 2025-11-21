REM General Bots: SEND EMAIL Keyword - Universal Email Sending
REM Free email sending using SMTP or email APIs
REM Can be used by ANY template that needs to send emails

PARAM to_email AS string LIKE "user@example.com"
PARAM subject AS string LIKE "Important Message"
PARAM body AS string LIKE "Hello, this is the email body content."
PARAM from_email AS string LIKE "noreply@generalbots.ai"

DESCRIPTION "Send an email to any recipient with subject and body"

REM Validate inputs
IF NOT to_email OR to_email = "" THEN
    TALK "❌ Recipient email is required"
    RETURN NULL
END IF

IF NOT subject OR subject = "" THEN
    subject = "Message from General Bots"
END IF

IF NOT body OR body = "" THEN
    body = "This is an automated message."
END IF

IF NOT from_email OR from_email = "" THEN
    from_email = "noreply@generalbots.ai"
END IF

TALK "📧 Preparing to send email..."
TALK "To: " + to_email
TALK "Subject: " + subject

REM Create email object
email_data = NEW OBJECT
email_data.to = to_email
email_data.from = from_email
email_data.subject = subject
email_data.body = body
email_data.timestamp = NOW()
email_data.status = "pending"

REM In production, this would integrate with:
REM 1. SMTP server (Gmail, SendGrid, etc.)
REM 2. Email API service (Mailgun, SendGrid, etc.)
REM 3. Microsoft Graph API for Office 365

REM For now, save to queue for processing
SAVE "email_queue.csv", email_data.timestamp, email_data.from, email_data.to, email_data.subject, email_data.body, email_data.status

TALK "✅ Email queued successfully!"
TALK ""
TALK "📊 Email Details:"
TALK "From: " + from_email
TALK "To: " + to_email
TALK "Subject: " + subject
TALK "Time: " + email_data.timestamp
TALK ""
TALK "⚙️ Status: Queued for delivery"
TALK ""
TALK "💡 Setup Guide:"
TALK "To enable actual email sending, configure SMTP in .gbot settings:"
TALK "1. SMTP_HOST (e.g., smtp.gmail.com)"
TALK "2. SMTP_PORT (e.g., 587)"
TALK "3. SMTP_USER (your email)"
TALK "4. SMTP_PASSWORD (your password or app password)"

REM Return email data
RETURN email_data
