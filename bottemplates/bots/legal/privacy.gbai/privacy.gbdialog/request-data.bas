' ============================================================================
' Privacy Template: Data Access Request (Subject Access Request - SAR)
' LGPD Art. 18 / GDPR Art. 15 - Right of Access
' ============================================================================
' This dialog handles user requests to access their personal data
' Companies can install this template for LGPD/GDPR compliance

TALK "Data Access Request"
TALK "You have the right to access all personal data we hold about you."
TALK ""

' Verify user identity
TALK "First, I need to verify your identity for security purposes."
HEAR email AS EMAIL WITH "Please provide your registered email address:"

' Check if email exists in system
user = FIND "users" WHERE email = email
IF user IS NULL THEN
    TALK "We couldn't find an account with that email address."
    TALK "Please check the email and try again, or contact support."
    EXIT
END IF

' Send verification code
code = GENERATE CODE 6
SET SESSION "verification_code", code
SET SESSION "verified_email", email

SEND MAIL email, "Data Access Request - Verification Code", "
Your verification code is: " + code + "

This code expires in 15 minutes.

If you did not request this, please ignore this email.
"

HEAR entered_code AS TEXT WITH "We sent a verification code to your email. Please enter it:"

IF entered_code <> code THEN
    TALK "Invalid verification code. Please start over."
    EXIT
END IF

TALK "Identity verified successfully!"
TALK ""

' Gather all user data
TALK "Gathering your personal data... This may take a moment."
TALK ""

' Get user profile data
profile = FIND "users" WHERE email = email
sessions = FIND "sessions" WHERE user_id = profile.id
messages = FIND "messages" WHERE user_id = profile.id
files = FIND "user_files" WHERE user_id = profile.id
consents = FIND "user_consents" WHERE user_id = profile.id
audit_logs = FIND "audit_logs" WHERE user_id = profile.id

' Build comprehensive report
report_data = {
    "request_date": NOW(),
    "request_type": "Subject Access Request (SAR)",
    "legal_basis": "LGPD Art. 18 / GDPR Art. 15",
    "profile": {
        "name": profile.name,
        "email": profile.email,
        "phone": profile.phone,
        "created_at": profile.created_at,
        "last_login": profile.last_login,
        "preferences": profile.preferences
    },
    "sessions": {
        "total_count": COUNT(sessions),
        "active_sessions": FILTER(sessions, "status = 'active'"),
        "session_history": sessions
    },
    "communications": {
        "total_messages": COUNT(messages),
        "messages": messages
    },
    "files": {
        "total_files": COUNT(files),
        "file_list": MAP(files, "name, size, created_at")
    },
    "consents": consents,
    "activity_log": audit_logs
}

' Generate PDF report
report_filename = "data_access_report_" + FORMAT(NOW(), "YYYYMMDD_HHmmss") + ".pdf"
GENERATE PDF report_filename WITH TEMPLATE "data_access_report" DATA report_data

' Upload to user's secure area
UPLOAD report_filename TO "/secure/reports/" + profile.id + "/"

' Send report via email
SEND MAIL email, "Your Data Access Request - Complete Report", "
Dear " + profile.name + ",

As requested, please find attached a complete report of all personal data we hold about you.

This report includes:
- Your profile information
- Session history
- Communication records
- Files you have uploaded
- Consent records
- Activity logs

Report generated: " + FORMAT(NOW(), "DD/MM/YYYY HH:mm") + "

Your rights under LGPD/GDPR:
- Right to rectification (Art. 18 III LGPD / Art. 16 GDPR)
- Right to erasure (Art. 18 VI LGPD / Art. 17 GDPR)
- Right to data portability (Art. 18 V LGPD / Art. 20 GDPR)
- Right to object to processing (Art. 18 IV LGPD / Art. 21 GDPR)

To exercise any of these rights, please contact us or use our privacy portal.

Best regards,
Privacy & Compliance Team
", ATTACHMENT report_filename

' Log the request for compliance audit
INSERT INTO "privacy_requests" VALUES {
    "user_id": profile.id,
    "request_type": "data_access",
    "requested_at": NOW(),
    "completed_at": NOW(),
    "status": "completed",
    "legal_basis": "LGPD Art. 18 / GDPR Art. 15"
}

TALK "Request Complete!"
TALK ""
TALK "We have sent a comprehensive report to: " + email
TALK ""
TALK "The report includes:"
TALK "- Your profile information"
TALK "- " + COUNT(sessions) + " session records"
TALK "- " + COUNT(messages) + " message records"
TALK "- " + COUNT(files) + " files"
TALK "- Consent history"
TALK "- Activity logs"
TALK ""
TALK "You can also download the report from your account settings."
TALK ""
TALK "Your other privacy rights:"
TALK "- Say 'correct my data' to update your information"
TALK "- Say 'delete my data' to request data erasure"
TALK "- Say 'export my data' for portable format"
TALK "- Say 'privacy settings' to manage consents"
