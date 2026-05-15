' =============================================================================
' Account Management Dialog
' Dynamics CRM-style Account Entity Management
' General Bots CRM Template
' =============================================================================
' This dialog provides comprehensive account (company) management similar to
' Microsoft Dynamics CRM including:
' - Account creation and updates
' - Account hierarchy (parent/child relationships)
' - Contact associations
' - Activity timeline
' - Account scoring and health
' =============================================================================

PARAM action AS TEXT

SELECT CASE UCASE(action)
    CASE "CREATE"
        CALL create_account
    CASE "UPDATE"
        CALL update_account
    CASE "VIEW"
        CALL view_account
    CASE "LIST"
        CALL list_accounts
    CASE "SEARCH"
        CALL search_accounts
    CASE "HIERARCHY"
        CALL account_hierarchy
    CASE "CONTACTS"
        CALL account_contacts
    CASE "ACTIVITIES"
        CALL account_activities
    CASE "HEALTH"
        CALL account_health
    CASE ELSE
        TALK "Account Management"
        TALK "Available actions: Create, Update, View, List, Search, Hierarchy, Contacts, Activities, Health"
        HEAR selected_action AS TEXT WITH "What would you like to do?"
        CALL "account-management.bas", selected_action
END SELECT

' -----------------------------------------------------------------------------
' CREATE ACCOUNT
' -----------------------------------------------------------------------------
SUB create_account
    TALK "Create New Account"

    HEAR account_name AS TEXT WITH "Company name:"
    HEAR account_type AS TEXT WITH "Account type (Customer, Partner, Vendor, Competitor, Other):" DEFAULT "Customer"
    HEAR industry AS TEXT WITH "Industry:"
    HEAR phone AS TEXT WITH "Main phone:" DEFAULT ""
    HEAR website AS TEXT WITH "Website:" DEFAULT ""
    HEAR email AS TEXT WITH "Primary email:" DEFAULT ""

    TALK "Address Information"
    HEAR street AS TEXT WITH "Street address:" DEFAULT ""
    HEAR city AS TEXT WITH "City:" DEFAULT ""
    HEAR state AS TEXT WITH "State/Province:" DEFAULT ""
    HEAR postal_code AS TEXT WITH "Postal code:" DEFAULT ""
    HEAR country AS TEXT WITH "Country:" DEFAULT ""

    TALK "Business Information"
    HEAR employees AS INTEGER WITH "Number of employees:" DEFAULT 0
    HEAR revenue AS MONEY WITH "Annual revenue:" DEFAULT 0
    HEAR description AS TEXT WITH "Description:" DEFAULT ""

    HEAR parent_account AS TEXT WITH "Parent account (leave empty if none):" DEFAULT ""

    account_id = "ACC-" + FORMAT(NOW(), "YYYYMMDDHHmmss") + "-" + RANDOM(1000, 9999)
    created_by = GET SESSION "user_email"

    account = {
        "id": account_id,
        "name": account_name,
        "type": account_type,
        "industry": industry,
        "phone": phone,
        "website": website,
        "email": email,
        "street": street,
        "city": city,
        "state": state,
        "postal_code": postal_code,
        "country": country,
        "employees": employees,
        "annual_revenue": revenue,
        "description": description,
        "parent_account_id": parent_account,
        "owner_id": created_by,
        "created_by": created_by,
        "created_at": NOW(),
        "modified_at": NOW(),
        "status": "Active",
        "health_score": 100
    }

    SAVE "accounts.csv", account

    LOG ACTIVITY account_id, "Account", "Created", "Account created: " + account_name, created_by

    RECORD METRIC "crm_accounts" WITH action="created", type=account_type

    TALK "Account created successfully"
    TALK "Account ID: " + account_id
    TALK "Name: " + account_name
    TALK "Type: " + account_type

    HEAR add_contact AS BOOLEAN WITH "Would you like to add a primary contact?"
    IF add_contact THEN
        SET BOT MEMORY "current_account_id", account_id
        CALL "contact-management.bas", "CREATE"
    END IF
END SUB

' -----------------------------------------------------------------------------
' UPDATE ACCOUNT
' -----------------------------------------------------------------------------
SUB update_account
    HEAR account_id AS TEXT WITH "Enter Account ID or search by name:"

    IF LEFT(account_id, 4) <> "ACC-" THEN
        accounts = FIND "accounts" WHERE name LIKE "%" + account_id + "%"
        IF COUNT(accounts) = 0 THEN
            TALK "No accounts found matching: " + account_id
            EXIT SUB
        ELSEIF COUNT(accounts) = 1 THEN
            account = FIRST(accounts)
            account_id = account.id
        ELSE
            TALK "Multiple accounts found:"
            FOR EACH acc IN accounts
                TALK acc.id + " - " + acc.name
            NEXT
            HEAR account_id AS TEXT WITH "Enter the Account ID:"
        END IF
    END IF

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found: " + account_id
        EXIT SUB
    END IF

    TALK "Updating: " + account.name
    TALK "Press Enter to keep current value"

    HEAR new_name AS TEXT WITH "Company name [" + account.name + "]:" DEFAULT account.name
    HEAR new_type AS TEXT WITH "Account type [" + account.type + "]:" DEFAULT account.type
    HEAR new_industry AS TEXT WITH "Industry [" + account.industry + "]:" DEFAULT account.industry
    HEAR new_phone AS TEXT WITH "Phone [" + account.phone + "]:" DEFAULT account.phone
    HEAR new_website AS TEXT WITH "Website [" + account.website + "]:" DEFAULT account.website
    HEAR new_email AS TEXT WITH "Email [" + account.email + "]:" DEFAULT account.email
    HEAR new_employees AS INTEGER WITH "Employees [" + account.employees + "]:" DEFAULT account.employees
    HEAR new_revenue AS MONEY WITH "Annual revenue [" + account.annual_revenue + "]:" DEFAULT account.annual_revenue
    HEAR new_status AS TEXT WITH "Status [" + account.status + "] (Active, Inactive, On Hold):" DEFAULT account.status

    UPDATE "accounts" SET
        name = new_name,
        type = new_type,
        industry = new_industry,
        phone = new_phone,
        website = new_website,
        email = new_email,
        employees = new_employees,
        annual_revenue = new_revenue,
        status = new_status,
        modified_at = NOW(),
        modified_by = GET SESSION "user_email"
    WHERE id = account_id

    LOG ACTIVITY account_id, "Account", "Updated", "Account updated", GET SESSION "user_email"

    TALK "Account updated successfully"
END SUB

' -----------------------------------------------------------------------------
' VIEW ACCOUNT (360-degree view)
' -----------------------------------------------------------------------------
SUB view_account
    HEAR account_id AS TEXT WITH "Enter Account ID:"

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found"
        EXIT SUB
    END IF

    TALK "Account Details"
    TALK "Name: " + account.name
    TALK "ID: " + account.id
    TALK "Type: " + account.type
    TALK "Industry: " + account.industry
    TALK "Status: " + account.status
    TALK ""
    TALK "Contact Information"
    TALK "Phone: " + account.phone
    TALK "Email: " + account.email
    TALK "Website: " + account.website
    TALK ""
    TALK "Address"
    TALK account.street
    TALK account.city + ", " + account.state + " " + account.postal_code
    TALK account.country
    TALK ""
    TALK "Business Information"
    TALK "Employees: " + FORMAT(account.employees, "#,###")
    TALK "Annual Revenue: " + FORMAT(account.annual_revenue, "$#,###")
    TALK "Health Score: " + account.health_score + "/100"
    TALK ""
    TALK "System Information"
    TALK "Owner: " + account.owner_id
    TALK "Created: " + FORMAT(account.created_at, "YYYY-MM-DD")
    TALK "Modified: " + FORMAT(account.modified_at, "YYYY-MM-DD")

    contacts = FIND "contacts" WHERE account_id = account_id
    opportunities = FIND "opportunities" WHERE account_id = account_id
    cases = FIND "cases" WHERE account_id = account_id

    TALK ""
    TALK "Related Records"
    TALK "Contacts: " + COUNT(contacts)
    TALK "Opportunities: " + COUNT(opportunities)
    TALK "Cases: " + COUNT(cases)

    open_opportunities = FILTER(opportunities, "status <> 'Closed Won' AND status <> 'Closed Lost'")
    total_pipeline = SUM(open_opportunities, "estimated_value")
    TALK "Pipeline Value: " + FORMAT(total_pipeline, "$#,###")

    won_opportunities = FILTER(opportunities, "status = 'Closed Won'")
    total_revenue = SUM(won_opportunities, "actual_value")
    TALK "Lifetime Revenue: " + FORMAT(total_revenue, "$#,###")
END SUB

' -----------------------------------------------------------------------------
' LIST ACCOUNTS
' -----------------------------------------------------------------------------
SUB list_accounts
    TALK "List Accounts"
    TALK "Filter by:"
    TALK "1. All Active"
    TALK "2. By Type"
    TALK "3. By Industry"
    TALK "4. By Owner"
    TALK "5. Recently Modified"

    HEAR filter_choice AS INTEGER

    SELECT CASE filter_choice
        CASE 1
            accounts = FIND "accounts" WHERE status = "Active" ORDER BY name
        CASE 2
            HEAR filter_type AS TEXT WITH "Account type (Customer, Partner, Vendor, Competitor):"
            accounts = FIND "accounts" WHERE type = filter_type AND status = "Active" ORDER BY name
        CASE 3
            HEAR filter_industry AS TEXT WITH "Industry:"
            accounts = FIND "accounts" WHERE industry = filter_industry AND status = "Active" ORDER BY name
        CASE 4
            HEAR filter_owner AS TEXT WITH "Owner email:"
            accounts = FIND "accounts" WHERE owner_id = filter_owner AND status = "Active" ORDER BY name
        CASE 5
            accounts = FIND "accounts" WHERE modified_at >= DATEADD(NOW(), -7, "day") ORDER BY modified_at DESC
        CASE ELSE
            accounts = FIND "accounts" WHERE status = "Active" ORDER BY name LIMIT 20
    END SELECT

    IF COUNT(accounts) = 0 THEN
        TALK "No accounts found"
        EXIT SUB
    END IF

    TALK "Found " + COUNT(accounts) + " accounts:"
    TALK ""

    FOR EACH acc IN accounts
        TALK acc.id + " | " + acc.name + " | " + acc.type + " | " + acc.industry
    NEXT
END SUB

' -----------------------------------------------------------------------------
' SEARCH ACCOUNTS
' -----------------------------------------------------------------------------
SUB search_accounts
    HEAR search_term AS TEXT WITH "Search accounts (name, email, phone, or website):"

    accounts = FIND "accounts" WHERE
        name LIKE "%" + search_term + "%" OR
        email LIKE "%" + search_term + "%" OR
        phone LIKE "%" + search_term + "%" OR
        website LIKE "%" + search_term + "%"

    IF COUNT(accounts) = 0 THEN
        TALK "No accounts found for: " + search_term
        EXIT SUB
    END IF

    TALK "Found " + COUNT(accounts) + " matching accounts:"
    FOR EACH acc IN accounts
        TALK acc.id + " - " + acc.name + " (" + acc.type + ")"
        TALK "  Phone: " + acc.phone + " | Email: " + acc.email
    NEXT
END SUB

' -----------------------------------------------------------------------------
' ACCOUNT HIERARCHY
' -----------------------------------------------------------------------------
SUB account_hierarchy
    HEAR account_id AS TEXT WITH "Enter Account ID to view hierarchy:"

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found"
        EXIT SUB
    END IF

    TALK "Account Hierarchy for: " + account.name
    TALK ""

    IF account.parent_account_id <> "" THEN
        parent = FIND "accounts" WHERE id = account.parent_account_id
        IF parent IS NOT NULL THEN
            TALK "Parent Account:"
            TALK "  " + parent.name + " (" + parent.id + ")"
        END IF
    END IF

    TALK ""
    TALK "Current Account:"
    TALK "  " + account.name + " (" + account.id + ")"

    children = FIND "accounts" WHERE parent_account_id = account_id
    IF COUNT(children) > 0 THEN
        TALK ""
        TALK "Child Accounts:"
        FOR EACH child IN children
            TALK "  - " + child.name + " (" + child.id + ")"
        NEXT
    END IF
END SUB

' -----------------------------------------------------------------------------
' ACCOUNT CONTACTS
' -----------------------------------------------------------------------------
SUB account_contacts
    HEAR account_id AS TEXT WITH "Enter Account ID:"

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found"
        EXIT SUB
    END IF

    contacts = FIND "contacts" WHERE account_id = account_id ORDER BY is_primary DESC, last_name

    TALK "Contacts for: " + account.name
    TALK "Total: " + COUNT(contacts)
    TALK ""

    IF COUNT(contacts) = 0 THEN
        TALK "No contacts associated with this account"
        HEAR add_new AS BOOLEAN WITH "Would you like to add a contact?"
        IF add_new THEN
            SET BOT MEMORY "current_account_id", account_id
            CALL "contact-management.bas", "CREATE"
        END IF
        EXIT SUB
    END IF

    FOR EACH contact IN contacts
        primary_marker = ""
        IF contact.is_primary THEN
            primary_marker = " [PRIMARY]"
        END IF
        TALK contact.first_name + " " + contact.last_name + primary_marker
        TALK "  Title: " + contact.job_title
        TALK "  Email: " + contact.email
        TALK "  Phone: " + contact.phone
        TALK ""
    NEXT
END SUB

' -----------------------------------------------------------------------------
' ACCOUNT ACTIVITIES
' -----------------------------------------------------------------------------
SUB account_activities
    HEAR account_id AS TEXT WITH "Enter Account ID:"

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found"
        EXIT SUB
    END IF

    activities = FIND "activities" WHERE related_to = account_id ORDER BY activity_date DESC LIMIT 20

    TALK "Recent Activities for: " + account.name
    TALK ""

    IF COUNT(activities) = 0 THEN
        TALK "No activities recorded"
        EXIT SUB
    END IF

    FOR EACH activity IN activities
        TALK FORMAT(activity.activity_date, "YYYY-MM-DD HH:mm") + " | " + activity.activity_type
        TALK "  " + activity.subject
        TALK "  By: " + activity.created_by
        TALK ""
    NEXT
END SUB

' -----------------------------------------------------------------------------
' ACCOUNT HEALTH SCORE
' -----------------------------------------------------------------------------
SUB account_health
    HEAR account_id AS TEXT WITH "Enter Account ID:"

    account = FIND "accounts" WHERE id = account_id
    IF account IS NULL THEN
        TALK "Account not found"
        EXIT SUB
    END IF

    contacts = FIND "contacts" WHERE account_id = account_id
    opportunities = FIND "opportunities" WHERE account_id = account_id
    activities = FIND "activities" WHERE related_to = account_id AND activity_date >= DATEADD(NOW(), -90, "day")
    cases = FIND "cases" WHERE account_id = account_id AND status = "Open"

    health_score = 100
    health_factors = []

    IF COUNT(contacts) = 0 THEN
        health_score = health_score - 20
        PUSH health_factors, "No contacts (-20)"
    END IF

    recent_activities = FILTER(activities, "activity_date >= " + DATEADD(NOW(), -30, "day"))
    IF COUNT(recent_activities) = 0 THEN
        health_score = health_score - 15
        PUSH health_factors, "No recent activity (-15)"
    END IF

    IF COUNT(cases) > 3 THEN
        health_score = health_score - 10
        PUSH health_factors, "Multiple open cases (-10)"
    END IF

    open_opps = FILTER(opportunities, "status <> 'Closed Won' AND status <> 'Closed Lost'")
    IF COUNT(open_opps) > 0 THEN
        health_score = health_score + 10
        PUSH health_factors, "Active opportunities (+10)"
    END IF

    won_opps = FILTER(opportunities, "status = 'Closed Won' AND close_date >= " + DATEADD(NOW(), -365, "day"))
    IF COUNT(won_opps) > 0 THEN
        health_score = health_score + 15
        PUSH health_factors, "Recent closed deals (+15)"
    END IF

    IF health_score > 100 THEN health_score = 100
    IF health_score < 0 THEN health_score = 0

    UPDATE "accounts" SET health_score = health_score, modified_at = NOW() WHERE id = account_id

    TALK "Account Health Assessment"
    TALK "Account: " + account.name
    TALK ""
    TALK "Health Score: " + health_score + "/100"
    TALK ""
    TALK "Factors:"
    FOR EACH factor IN health_factors
        TALK "  - " + factor
    NEXT
    TALK ""
    TALK "Statistics:"
    TALK "  Contacts: " + COUNT(contacts)
    TALK "  Activities (90 days): " + COUNT(activities)
    TALK "  Open Cases: " + COUNT(cases)
    TALK "  Open Opportunities: " + COUNT(open_opps)
END SUB
