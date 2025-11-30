' =============================================================================
' Privacy Rights Center - LGPD/GDPR Compliance Dialog
' General Bots Template for Data Subject Rights Management
' =============================================================================
' This template helps organizations comply with:
' - LGPD (Lei Geral de Proteção de Dados - Brazil)
' - GDPR (General Data Protection Regulation - EU)
' - CCPA (California Consumer Privacy Act)
' =============================================================================

TALK "Welcome to the Privacy Rights Center. I can help you exercise your data protection rights."
TALK "As a data subject, you have the following rights under LGPD/GDPR:"

TALK "1. Right of Access - View all data we hold about you"
TALK "2. Right to Rectification - Correct inaccurate data"
TALK "3. Right to Erasure - Request deletion of your data"
TALK "4. Right to Portability - Export your data"
TALK "5. Right to Object - Opt-out of certain processing"
TALK "6. Consent Management - Review and update your consents"

HEAR choice AS TEXT WITH "What would you like to do? (1-6 or type your request)"

SELECT CASE choice
    CASE "1", "access", "view", "see my data"
        CALL "access-data.bas"

    CASE "2", "rectification", "correct", "update", "fix"
        CALL "rectify-data.bas"

    CASE "3", "erasure", "delete", "remove", "forget me"
        CALL "erase-data.bas"

    CASE "4", "portability", "export", "download"
        CALL "export-data.bas"

    CASE "5", "object", "opt-out", "stop processing"
        CALL "object-processing.bas"

    CASE "6", "consent", "cons