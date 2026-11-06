' Privacy Template - Data Deletion Request (Right to be Forgotten)
' LGPD Art. 18, GDPR Art. 17, HIPAA (where applicable)
' This dialog handles user requests to delete their personal data

TALK "Data Deletion Request"
TALK "I can help you exercise your right to have your personal data deleted."
TALK "This is also known as the 'Right to be Forgotten' under LGPD and GDPR."

' Authenticate the user first
TALK "For security purposes, I need to verify your identity before proceeding."
HEAR email AS EMAIL WITH "Please enter your registered email address:"

' Verify email exists in system
user = FIND "users.csv" WHERE email = email
IF user IS NULL THEN
    TALK "I couldn't find an account with that email address."
    TALK "Please check the email and try again, or contact support@company.com"
    EXIT
END IF

' Send verification code
verification_code = RANDOM(100000, 999999)
SET BOT MEMORY "verification_" + email, verification_code
SET BOT MEMORY "verification_expiry_" + email, NOW() + 15 * 60

SEND MAIL email, "Data Deletion Verification Code", "
Your verification code is: " + verification_code + "

This code expires in 15 minutes.

If you did not request data deletion, please ignore this email and contact support immediately.

Pragmatismo Privacy Team
"

HEAR entered_code AS INTEGER WITH "I've sent a verification code to your email. Please enter it here:"

stored_code = GET BOT MEMORY "verification_" + email
IF entered_code <> stored_code THEN
    TALK "Invalid verification code. Please try again."
    EXIT
END IF

TALK "Identity verified."
TALK ""
TALK "What data would you like to delete?"
TALK ""
TALK "1. All my personal data (complete account deletion)"
TALK "2. Conversation history only"
TALK "3. Files and documents only"
TALK "4. Activity logs and analytics"
TALK "5. Specific data categories (I'll choose)"
TALK "6. Cancel this request"

HEAR deletion_choice AS INTEGER WITH "Please enter your choice (1-6):"

SELECT CASE deletion_choice
    CASE 1
        deletion_type = "complete"
        TALK "Complete Account Deletion"
        TALK "This will permanently delete:"
        TALK "- Your user profile and account"
        TALK "- All conversation history"
        TALK "- All uploaded files and documents"
        TALK "- All activity logs"
        TALK "- All preferences and settings"
        TALK ""
        TALK "This action cannot be undone."

    CASE 2
        deletion_type = "conversations"
        TALK "This will delete all your conversation history with our bots."

    CASE 3
        deletion_type = "files"
        TALK "This will delete all files and documents you've uploaded."

    CASE 4
        deletion_type = "logs"
        TALK "This will delete all activity logs and analytics data associated with you."

    CASE 5
        deletion_type = "selective"
        TALK "Please specify which data categories you want deleted:"
        HEAR categories WITH "Enter categories separated by commas (e.g., 'email history, phone number, address'):"

    CASE 6
        TALK "Request cancelled. No data has been deleted."
        EXIT

    CASE ELSE
        TALK "Invalid choice. Please start over."
        EXIT
END SELECT

' Explain data retention exceptions
TALK ""
TALK "Legal Notice:"
TALK "Some data may be retained for legal compliance purposes:"
TALK "- Financial records (tax requirements)"
TALK "- Legal dispute documentation"
TALK "- Fraud prevention records"
TALK "- Regulatory compliance data"
TALK ""
TALK "Retained data will be minimized and protected according to law."

HEAR reason WITH "Please briefly explain why you're requesting deletion (optional, press Enter to skip):"

HEAR confirmation WITH "Type 'DELETE MY DATA' to confirm this irreversible action:"

IF confirmation <> "DELETE MY DATA" THEN
    TALK "Confirmation not received. Request cancelled for your protection."
    EXIT
END IF

' Log the deletion request
request_id = "DEL-" + FORMAT(NOW(), "YYYYMMDD") + "-" + RANDOM(10000, 99999)
request_date = NOW()

' Create deletion request record
INSERT INTO "deletion_requests.csv", request_id, email, deletion_type, categories, reason, request_date, "pending"

' Process the deletion based on type
SELECT CASE deletion_type
    CASE "complete"
        ' Delete from all tables
        DELETE FROM "messages" WHERE user_email = email
        DELETE FROM "files" WHERE owner_email = email
        DELETE FROM "activity_logs" WHERE user_email = email
        DELETE FROM "user_preferences" WHERE email = email
        DELETE FROM "sessions" WHERE user_email = email

        ' Anonymize required retention records
        UPDATE "audit_logs" SET user_email = "DELETED_USER_" + request_id WHERE user_email = email

        ' Mark user for deletion (actual deletion after retention period)
        UPDATE "users" SET status = "pending_deletion", deletion_request_id = request_id WHERE email = email

    CASE "conversations"
        DELETE FROM "messages" WHERE user_email = email
        DELETE FROM "sessions" WHERE user_email = email

    CASE "files"
        ' Get file list for physical deletion
        files = FIND "files" WHERE owner_email = email
        FOR EACH file IN files
            DELETE FILE file.path
        NEXT
        DELETE FROM "files" WHERE owner_email = email

    CASE "logs"
        DELETE FROM "activity_logs" WHERE user_email = email
        ' Anonymize audit logs (keep for compliance but remove PII)
        UPDATE "audit_logs" SET user_email = "ANONYMIZED" WHERE user_email = email

    CASE "selective"
        ' Process specific categories
        TALK "Processing selective deletion for: " + categories
        ' Custom handling based on categories specified
        INSERT INTO "manual_deletion_queue", request_id, email, categories, request_date
END SELECT

' Update request status
UPDATE "deletion_requests" SET status = "completed", completion_date = NOW() WHERE request_id = request_id

' Send confirmation email
SEND MAIL email, "Data Deletion Request Confirmed - " + request_id, "
Dear User,

Your data deletion request has been received and processed.

Request Details:
- Request ID: " + request_id + "
- Request Date: " + FORMAT(request_date, "YYYY-MM-DD HH:mm") + "
- Deletion Type: " + deletion_type + "
- Status: Completed

What happens next:
" + IF(deletion_type = "complete", "
- Your account will be fully deleted within 30 days
- You will receive a final confirmation email
- Some data may be retained for legal compliance (anonymized)
", "
- The specified data has been deleted from our systems
- Some backups may take up to 30 days to purge
") + "

Your Rights:
- You can request a copy of any retained data
- You can file a complaint with your data protection authority
- Contact us at privacy@company.com for questions

Under LGPD (Brazil) and GDPR (EU), you have the right to:
- Request confirmation of this deletion
- Lodge a complaint with supervisory authorities
- Seek judicial remedy if unsatisfied

Thank you for trusting us with your data.

Pragmatismo Privacy Team
Request Reference: " + request_id + "
"

TALK ""
TALK "Request Completed"
TALK ""
TALK "Your deletion request has been processed."
TALK "Request ID: " + request_id
TALK ""
TALK "A confirmation email has been sent to " + email
TALK ""
TALK "If you have questions, contact privacy@company.com"
TALK "Reference your Request ID in any communications."
