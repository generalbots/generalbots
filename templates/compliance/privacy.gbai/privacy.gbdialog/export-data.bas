' ============================================================================
' Privacy Template: Data Portability/Export Request
' LGPD Art. 18 V / GDPR Art. 20 - Right to Data Portability
' ============================================================================
' This dialog enables users to export their data in portable formats
' Supports JSON, CSV, and XML export for interoperability

TALK "📦 **Data Portability Request**"
TALK "You have the right to receive your personal data in a structured, commonly used, and machine-readable format."
TALK ""

' Verify user identity
TALK "First, I need to verify your identity."
HEAR email AS EMAIL WITH "Please enter your registered email address:"

user = FIND "users" WHERE email = email
IF user IS NULL THEN
    TALK "❌ No account found with that email address."
    TALK "Please check and try again, or contact support."
    EXIT
END IF

' Send verification code
code = GENERATE CODE 6
SET SESSION "export_verification_code", code
SET SESSION "export_email", email

SEND MAIL email, "Data Export Request - Verification Code", "
Your verification code is: " + code + "

This code expires in 15 minutes.

If you did not request this data export, please ignore this email.

Pragmatismo Privacy Team
"

HEAR entered_code AS TEXT WITH "📧 Enter the verification code sent to your email:"

IF entered_code <> code THEN
    TALK "❌ Invalid verification code. Please start over."
    EXIT
END IF

TALK "✅ Identity verified!"
TALK ""

' Ask for export format
TALK "**Choose your export format:**"
TALK ""
TALK "1️⃣ **JSON** - Best for importing into other systems"
TALK "2️⃣ **CSV** - Best for spreadsheets (Excel, Google Sheets)"
TALK "3️⃣ **XML** - Universal interchange format"
TALK "4️⃣ **All formats** - Get all three formats in a ZIP file"

HEAR format_choice WITH "Enter your choice (1-4):"

SELECT CASE format_choice
    CASE "1", "json", "JSON"
        export_format = "json"
        format_name = "JSON"
    CASE "2", "csv", "CSV"
        export_format = "csv"
        format_name = "CSV"
    CASE "3", "xml", "XML"
        export_format = "xml"
        format_name = "XML"
    CASE "4", "all", "ALL"
        export_format = "all"
        format_name = "All Formats (ZIP)"
    CASE ELSE
        export_format = "json"
        format_name = "JSON"
        TALK "Defaulting to JSON format."
END SELECT

TALK ""
TALK "**Select data categories to export:**"
TALK ""
TALK "1️⃣ Everything (complete data export)"
TALK "2️⃣ Profile information only"
TALK "3️⃣ Conversations and messages"
TALK "4️⃣ Files and documents"
TALK "5️⃣ Activity history"
TALK "6️⃣ Custom selection"

HEAR data_choice WITH "Enter your choice (1-6):"

' Define what data to export based on choice
SELECT CASE data_choice
    CASE "1"
        include_profile = TRUE
        include_conversations = TRUE
        include_files = TRUE
        include_activity = TRUE
        include_consents = TRUE
        data_scope = "complete"

    CASE "2"
        include_profile = TRUE
        include_conversations = FALSE
        include_files = FALSE
        include_activity = FALSE
        include_consents = TRUE
        data_scope = "profile"

    CASE "3"
        include_profile = FALSE
        include_conversations = TRUE
        include_files = FALSE
        include_activity = FALSE
        include_consents = FALSE
        data_scope = "conversations"

    CASE "4"
        include_profile = FALSE
        include_conversations = FALSE
        include_files = TRUE
        include_activity = FALSE
        include_consents = FALSE
        data_scope = "files"

    CASE "5"
        include_profile = FALSE
        include_conversations = FALSE
        include_files = FALSE
        include_activity = TRUE
        include_consents = FALSE
        data_scope = "activity"

    CASE "6"
        TALK "Select categories (yes/no for each):"
        HEAR include_profile AS BOOLEAN WITH "Include profile information?"
        HEAR include_conversations AS BOOLEAN WITH "Include conversations?"
        HEAR include_files AS BOOLEAN WITH "Include files metadata?"
        HEAR include_activity AS BOOLEAN WITH "Include activity logs?"
        HEAR include_consents AS BOOLEAN WITH "Include consent records?"
        data_scope = "custom"

    CASE ELSE
        include_profile = TRUE
        include_conversations = TRUE
        include_files = TRUE
        include_activity = TRUE
        include_consents = TRUE
        data_scope = "complete"
END SELECT

TALK ""
TALK "🔄 Preparing your data export... This may take a few minutes."
TALK ""

' Gather the data
export_data = {}
request_id = "EXP-" + FORMAT(NOW(), "YYYYMMDD-HHmmss") + "-" + user.id

' Export metadata
export_data.metadata = {
    "export_id": request_id,
    "export_date": NOW(),
    "format": format_name,
    "data_scope": data_scope,
    "legal_basis": "LGPD Art. 18 V / GDPR Art. 20",
    "data_controller": "Your Organization Name",
    "contact": "privacy@company.com"
}

' Gather profile data
IF include_profile THEN
    profile = FIND "users" WHERE id = user.id
    export_data.profile = {
        "name": profile.name,
        "email": profile.email,
        "phone": profile.phone,
        "address": profile.address,
        "created_at": profile.created_at,
        "last_login": profile.last_login,
        "timezone": profile.timezone,
        "language": profile.language,
        "preferences": profile.preferences
    }
    TALK "✓ Profile data collected"
END IF

' Gather conversations
IF include_conversations THEN
    messages = FIND "messages" WHERE user_id = user.id ORDER BY created_at
    sessions = FIND "sessions" WHERE user_id = user.id

    export_data.conversations = {
        "total_sessions": COUNT(sessions),
        "total_messages": COUNT(messages),
        "sessions": sessions,
        "messages": messages
    }
    TALK "✓ Conversation data collected (" + COUNT(messages) + " messages)"
END IF

' Gather files metadata
IF include_files THEN
    files = FIND "user_files" WHERE user_id = user.id

    file_list = []
    FOR EACH file IN files
        file_info = {
            "filename": file.name,
            "size": file.size,
            "type": file.mime_type,
            "uploaded_at": file.created_at,
            "last_accessed": file.last_accessed,
            "path": file.path
        }
        APPEND file_list, file_info
    NEXT

    export_data.files = {
        "total_files": COUNT(files),
        "total_size": SUM(files, "size"),
        "file_list": file_list
    }
    TALK "✓ Files metadata collected (" + COUNT(files) + " files)"
END IF

' Gather activity logs
IF include_activity THEN
    activity = FIND "activity_logs" WHERE user_id = user.id ORDER BY timestamp DESC LIMIT 10000

    export_data.activity = {
        "total_events": COUNT(activity),
        "events": activity
    }
    TALK "✓ Activity logs collected (" + COUNT(activity) + " events)"
END IF

' Gather consent records
IF include_consents THEN
    consents = FIND "user_consents" WHERE user_id = user.id

    export_data.consents = {
        "consent_records": consents,
        "current_preferences": {
            "marketing_emails": user.marketing_consent,
            "analytics": user.analytics_consent,
            "third_party_sharing": user.sharing_consent
        }
    }
    TALK "✓ Consent records collected"
END IF

TALK ""
TALK "📁 Generating export files..."

' Generate export files based on format
timestamp = FORMAT(NOW(), "YYYYMMDD_HHmmss")
base_filename = "data_export_" + timestamp

SELECT CASE export_format
    CASE "json"
        filename = base_filename + ".json"
        WRITE filename, JSON(export_data)

    CASE "csv"
        ' Generate multiple CSV files for different data types
        IF include_profile THEN
            WRITE base_filename + "_profile.csv", CSV(export_data.profile)
        END IF
        IF include_conversations THEN
            WRITE base_filename + "_messages.csv", CSV(export_data.conversations.messages)
        END IF
        IF include_files THEN
            WRITE base_filename + "_files.csv", CSV(export_data.files.file_list)
        END IF
        IF include_activity THEN
            WRITE base_filename + "_activity.csv", CSV(export_data.activity.events)
        END IF
        ' Create ZIP of all CSVs
        filename = base_filename + "_csv.zip"
        COMPRESS filename, base_filename + "_*.csv"

    CASE "xml"
        filename = base_filename + ".xml"
        WRITE filename, XML(export_data)

    CASE "all"
        ' Generate all formats
        WRITE base_filename + ".json", JSON(export_data)
        WRITE base_filename + ".xml", XML(export_data)

        IF include_profile THEN
            WRITE base_filename + "_profile.csv", CSV(export_data.profile)
        END IF
        IF include_conversations THEN
            WRITE base_filename + "_messages.csv", CSV(export_data.conversations.messages)
        END IF
        IF include_files THEN
            WRITE base_filename + "_files.csv", CSV(export_data.files.file_list)
        END IF

        filename = base_filename + "_complete.zip"
        COMPRESS filename, base_filename + ".*"
END SELECT

' Upload to secure storage
secure_path = "/secure/exports/" + user.id + "/"
UPLOAD filename TO secure_path

' Generate download link (expires in 7 days)
download_link = GENERATE SECURE LINK secure_path + filename EXPIRES 7 DAYS

' Log the export request for compliance
INSERT INTO "privacy_requests" VALUES {
    "id": request_id,
    "user_id": user.id,
    "request_type": "data_portability",
    "data_scope": data_scope,
    "format": format_name,
    "requested_at": NOW(),
    "completed_at": NOW(),
    "status": "completed",
    "legal_basis": "LGPD Art. 18 V / GDPR Art. 20"
}

' Send email with download link
SEND MAIL email, "Your Data Export is Ready - " + request_id, "
Dear " + user.name + ",

Your data export request has been completed.

**Export Details:**
- Request ID: " + request_id + "
- Format: " + format_name + "
- Data Included: " + data_scope + "
- Generated: " + FORMAT(NOW(), "DD/MM/YYYY HH:mm") + "

**Download Your Data:**
" + download_link + "

⚠️ This link expires in 7 days for security purposes.

**What's Included:**
" + IF(include_profile, "✓ Profile information\n", "") + IF(include_conversations, "✓ Conversation history\n", "") + IF(include_files, "✓ Files metadata\n", "") + IF(include_activity, "✓ Activity logs\n", "") + IF(include_consents, "✓ Consent records\n", "") + "

**Your Rights Under LGPD/GDPR:**
- Import this data to another service provider
- Request data deletion after export
- Request additional data categories
- File a complaint with data protection authorities

If you have questions, contact privacy@company.com

Pragmatismo Privacy Team
"

TALK ""
TALK "✅ **Export Complete!**"
TALK ""
TALK "📧 A download link has been sent to: " + email
TALK ""
TALK "**Export Details:**"
TALK "• Request ID: " + request_id
TALK "• Format: " + format_name
TALK "• Link expires in: 7 days"
TALK ""
TALK "You can use this data to:"
TALK "• Import into another service"
TALK "• Keep a personal backup"
TALK "• Review what data we hold"
TALK ""
TALK "🔒 Need anything else?"
TALK "• Say **'delete my data'** to request deletion"
TALK "• Say **'privacy settings'** to manage consents"
TALK "• Say **'help'** for other options"
