' =============================================================================
' HIPAA Medical Privacy Portal - Main Dialog
' General Bots Template for Healthcare Data Protection
' =============================================================================
' This template helps healthcare organizations comply with:
' - HIPAA (Health Insurance Portability and Accountability Act)
' - HITECH Act (Health Information Technology for Economic and Clinical Health)
' - State-specific healthcare privacy regulations
' =============================================================================

TALK "🏥 Welcome to the HIPAA Privacy Portal"
TALK "I can help you manage your Protected Health Information (PHI) rights."
TALK ""

TALK "Under HIPAA, you have the following rights:"
TALK ""
TALK "1️⃣ **Access Your Medical Records** - Request copies of your health information"
TALK "2️⃣ **Request Amendments** - Correct errors in your medical records"
TALK "3️⃣ **Accounting of Disclosures** - See who has accessed your PHI"
TALK "4️⃣ **Request Restrictions** - Limit how we use or share your information"
TALK "5️⃣ **Confidential Communications** - Choose how we contact you"
TALK "6️⃣ **File a Privacy Complaint** - Report a privacy concern"
TALK "7️⃣ **Revoke Authorization** - Withdraw previous consent for PHI disclosure"

HEAR choice AS "What would you like to do? (1-7 or describe your request)"

SELECT CASE choice
    CASE "1", "access", "records", "medical records", "view", "copy"
        CALL "access-phi.bas"

    CASE "2", "amend", "amendment", "correct", "correction", "fix", "error"
        CALL "request-amendment.bas"

    CASE "3", "accounting", "disclosures", "who accessed", "access log"
        CALL "accounting-disclosures.bas"

    CASE "4", "restrict", "restriction", "limit", "limitations"
        CALL "request-restrictions.bas"

    CASE "5", "communications", "contact", "how to contact", "confidential"
        CALL "confidential-communications.bas"

    CASE "6", "complaint", "report", "privacy concern", "violation"
        CALL "file-complaint.bas"

    CASE "7", "revoke", "withdraw", "cancel authorization"
        CALL "revoke-authorization.bas"

    CASE ELSE
        ' Use LLM to understand medical privacy requests
        SET CONTEXT "You are a HIPAA compliance assistant. Classify the user's request into one of these categories: access_records, amendment, disclosures, restrictions, communications, complaint, revoke. Only respond with the category name."

        intent = LLM "Classify this healthcare privacy request: " + choice

        SELECT CASE intent
            CASE "access_records"
                CALL "access-phi.bas"
            CASE "amendment"
                CALL "request-amendment.bas"
            CASE "disclosures"
                CALL "accounting-disclosures.bas"
            CASE "restrictions"
                CALL "request-restrictions.bas"
            CASE "communications"
                CALL "confidential-communications.bas"
            CASE "complaint"
                CALL "file-complaint.bas"
            CASE "revoke"
                CALL "revoke-authorization.bas"
            CASE ELSE
                TALK "I'm not sure I understood your request."
                TALK "Please select a number from 1-7 or contact our Privacy Officer directly."
                TALK ""
                TALK "📞 Privacy Officer: privacy@healthcare.org"
                TALK "📧 Email: hipaa-requests@healthcare.org"
                CALL "start.bas"
        END SELECT
END SELECT

' Log all interactions for HIPAA audit trail
INSERT INTO "hipaa_audit_log" VALUES {
    "timestamp": NOW(),
    "session_id": GET SESSION "id",
    "action": "privacy_portal_access",
    "choice": choice,
    "ip_address": GET SESSION "client_ip",
    "user_agent": GET SESSION "user_agent"
}
