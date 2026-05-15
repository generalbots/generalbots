' ============================================================================
' Privacy Template: Consent Management
' LGPD Art. 8 / GDPR Art. 7 - Consent Management
' ============================================================================
' This dialog allows users to view, grant, and revoke their consents
' Essential for LGPD/GDPR compliance with granular consent tracking

TALK "🔐 **Consent Management Center**"
TALK "Here you can view and manage all your data processing consents."
TALK ""

' Verify user identity first
HEAR email AS EMAIL WITH "Please enter your registered email address:"

user = FIND "users" WHERE email = email
IF user IS NULL THEN
    TALK "⚠️ We couldn't find an account with that email."
    TALK "Please check the email address and try again."
    EXIT
END IF

' Send quick verification
code = GENERATE CODE 6
SET SESSION "consent_verify_code", code
SET SESSION "consent_verify_email", email

SEND MAIL email, "Consent Management - Verification", "
Your verification code is: " + code + "

This code expires in 10 minutes.

Pragmatismo Privacy Team
"

HEAR entered_code AS TEXT WITH "📧 Enter the verification code sent to your email:"

IF entered_code <> code THEN
    TALK "❌ Invalid code. Please try again."
    EXIT
END IF

TALK "✅ Identity verified!"
TALK ""

' Load current consents
consents = FIND "user_consents" WHERE user_id = user.id

' Define consent categories
consent_categories = [
    {
        "id": "essential",
        "name": "Essential Services",
        "description": "Required for basic service functionality",
        "required": TRUE,
        "legal_basis": "Contract performance"
    },
    {
        "id": "analytics",
        "name": "Analytics & Improvement",
        "description": "Help us improve our services through usage analysis",
        "required": FALSE,
        "legal_basis": "Legitimate interest / Consent"
    },
    {
        "id": "marketing",
        "name": "Marketing Communications",
        "description": "Receive news, updates, and promotional content",
        "required": FALSE,
        "legal_basis": "Consent"
    },
    {
        "id": "personalization",
        "name": "Personalization",
        "description": "Customize your experience based on preferences",
        "required": FALSE,
        "legal_basis": "Consent"
    },
    {
        "id": "third_party",
        "name": "Third-Party Sharing",
        "description": "Share data with trusted partners for enhanced services",
        "required": FALSE,
        "legal_basis": "Consent"
    },
    {
        "id": "ai_training",
        "name": "AI Model Training",
        "description": "Use anonymized data to improve AI capabilities",
        "required": FALSE,
        "legal_basis": "Consent"
    }
]

TALK "📋 **Your Current Consents:**"
TALK ""

FOR EACH category IN consent_categories
    current_consent = FILTER(consents, "category = '" + category.id + "'")
    IF current_consent IS NOT NULL THEN
        status = current_consent.granted ? "✅ Granted" : "❌ Denied"
        granted_date = FORMAT(current_consent.updated_at, "DD/MM/YYYY")
    ELSE
        status = "⚪ Not Set"
        granted_date = "N/A"
    END IF

    required_tag = category.required ? " (Required)" : ""
    TALK category.name + required_tag + ": " + status
    TALK "   └─ " + category.description
    TALK "   └─ Legal basis: " + category.legal_basis
    TALK "   └─ Last updated: " + granted_date
    TALK ""
NEXT

TALK "**What would you like to do?**"
TALK ""
TALK "1️⃣ Grant a consent"
TALK "2️⃣ Revoke a consent"
TALK "3️⃣ Revoke ALL optional consents"
TALK "4️⃣ Grant ALL consents"
TALK "5️⃣ View consent history"
TALK "6️⃣ Download consent record"
TALK "7️⃣ Exit"

HEAR action AS INTEGER WITH "Enter your choice (1-7):"

SELECT CASE action
    CASE 1
        ' Grant consent
        TALK "Which consent would you like to grant?"
        TALK "Available options: analytics, marketing, personalization, third_party, ai_training"
        HEAR grant_category WITH "Enter consent category:"

        ' Validate category
        valid_categories = ["analytics", "marketing", "personalization", "third_party", "ai_training"]
        IF NOT CONTAINS(valid_categories, grant_category) THEN
            TALK "❌ Invalid category. Please try again."
            EXIT
        END IF

        ' Record consent with full audit trail
        consent_record = {
            "user_id": user.id,
            "category": grant_category,
            "granted": TRUE,
            "granted_at": NOW(),
            "updated_at": NOW(),
            "ip_address": GET SESSION "client_ip",
            "user_agent": GET SESSION "user_agent",
            "consent_version": "2.0",
            "method": "explicit_dialog"
        }

        ' Check if exists and update, otherwise insert
        existing = FIND "user_consents" WHERE user_id = user.id AND category = grant_category
        IF existing IS NOT NULL THEN
            UPDATE "user_consents" SET granted = TRUE, updated_at = NOW(), method = "explicit_dialog" WHERE id = existing.id
        ELSE
            INSERT INTO "user_consents" VALUES consent_record
        END IF

        ' Log to consent history
        INSERT INTO "consent_history" VALUES {
            "user_id": user.id,
            "category": grant_category,
            "action": "granted",
            "timestamp": NOW(),
            "ip_address": GET SESSION "client_ip"
        }

        TALK "✅ Consent for **" + grant_category + "** has been granted."
        TALK "You can revoke this consent at any time."

    CASE 2
        ' Revoke consent
        TALK "Which consent would you like to revoke?"
        TALK "Note: Essential services consent cannot be revoked while using the service."
        HEAR revoke_category WITH "Enter consent category:"

        IF revoke_category = "essential" THEN
            TALK "⚠️ Essential consent is required for service operation."
            TALK "To revoke it, you must delete your account."
            EXIT
        END IF

        UPDATE "user_consents" SET granted = FALSE, updated_at = NOW(), method = "explicit_revoke" WHERE user_id = user.id AND category = revoke_category

        INSERT INTO "consent_history" VALUES {
            "user_id": user.id,
            "category": revoke_category,
            "action": "revoked",
            "timestamp": NOW(),
            "ip_address": GET SESSION "client_ip"
        }

        TALK "✅ Consent for **" + revoke_category + "** has been revoked."
        TALK "This change takes effect immediately."

        ' Notify relevant systems
        WEBHOOK POST "/internal/consent-changed" WITH {
            "user_id": user.id,
            "category": revoke_category,
            "action": "revoked"
        }

    CASE 3
        ' Revoke all optional
        TALK "⚠️ This will revoke ALL optional consents:"
        TALK "• Analytics & Improvement"
        TALK "• Marketing Communications"
        TALK "• Personalization"
        TALK "• Third-Party Sharing"
        TALK "• AI Model Training"

        HEAR confirm WITH "Type 'REVOKE ALL' to confirm:"

        IF confirm <> "REVOKE ALL" THEN
            TALK "Operation cancelled."
            EXIT
        END IF

        UPDATE "user_consents" SET granted = FALSE, updated_at = NOW() WHERE user_id = user.id AND category <> "essential"

        INSERT INTO "consent_history" VALUES {
            "user_id": user.id,
            "category": "ALL_OPTIONAL",
            "action": "bulk_revoked",
            "timestamp": NOW(),
            "ip_address": GET SESSION "client_ip"
        }

        TALK "✅ All optional consents have been revoked."

    CASE 4
        ' Grant all
        TALK "This will grant consent for all categories."
        TALK "You can revoke individual consents at any time."

        HEAR confirm WITH "Type 'GRANT ALL' to confirm:"

        IF confirm <> "GRANT ALL" THEN
            TALK "Operation cancelled."
            EXIT
        END IF

        FOR EACH category IN consent_categories
            existing = FIND "user_consents" WHERE user_id = user.id AND category = category.id
            IF existing IS NOT NULL THEN
                UPDATE "user_consents" SET granted = TRUE, updated_at = NOW() WHERE id = existing.id
            ELSE
                INSERT INTO "user_consents" VALUES {
                    "user_id": user.id,
                    "category": category.id,
                    "granted": TRUE,
                    "granted_at": NOW(),
                    "updated_at": NOW(),
                    "method": "bulk_grant"
                }
            END IF
        NEXT

        INSERT INTO "consent_history" VALUES {
            "user_id": user.id,
            "category": "ALL",
            "action": "bulk_granted",
            "timestamp": NOW()
        }

        TALK "✅ All consents have been granted."

    CASE 5
        ' View history
        TALK "📜 **Your Consent History:**"
        TALK ""

        history = FIND "consent_history" WHERE user_id = user.id ORDER BY timestamp DESC LIMIT 20

        IF COUNT(history) = 0 THEN
            TALK "No consent history found."
        ELSE
            FOR EACH record IN history
                action_icon = record.action CONTAINS "grant" ? "✅" : "❌"
                TALK action_icon + " " + FORMAT(record.timestamp, "DD/MM/YYYY HH:mm") + " - " + record.category + " " + record.action
            NEXT
        END IF

    CASE 6
        ' Download consent record
        TALK "📥 Generating your consent record..."

        consent_report = {
            "generated_at": NOW(),
            "user_email": email,
            "current_consents": consents,
            "consent_history": FIND "consent_history" WHERE user_id = user.id,
            "legal_notice": "This document serves as proof of consent status under LGPD/GDPR"
        }

        filename = "consent_record_" + FORMAT(NOW(), "YYYYMMDD") + ".pdf"
        GENERATE PDF filename WITH TEMPLATE "consent_report" DATA consent_report

        SEND MAIL email, "Your Consent Record", "
Dear User,

Please find attached your complete consent record as requested.

This document includes:
- Current consent status for all categories
- Complete consent history with timestamps
- Legal basis for each processing activity

Keep this document for your records.

Pragmatismo Privacy Team
        ", ATTACHMENT filename

        TALK "✅ Consent record has been sent to " + email

    CASE 7
        TALK "Thank you for managing your privacy preferences."
        TALK "You can return here anytime to update your consents."
        EXIT

    CASE ELSE
        TALK "Invalid choice. Please try again."
END SELECT

TALK ""
TALK "🔒 **Privacy Reminder:**"
TALK "• Your consents are stored securely"
TALK "• Changes take effect immediately"
TALK "• You can modify consents anytime"
TALK "• Contact privacy@company.com for questions"
