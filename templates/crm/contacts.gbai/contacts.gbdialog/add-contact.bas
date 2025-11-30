PARAM firstname AS STRING LIKE "John" DESCRIPTION "First name of the contact"
PARAM lastname AS STRING LIKE "Smith" DESCRIPTION "Last name of the contact"
PARAM email AS STRING LIKE "john.smith@company.com" DESCRIPTION "Email address of the contact"
PARAM phone AS STRING LIKE "+1-555-123-4567" DESCRIPTION "Phone number of the contact"
PARAM companyname AS STRING LIKE "Acme Corporation" DESCRIPTION "Company or organization name"
PARAM jobtitle AS STRING LIKE "Sales Manager" DESCRIPTION "Job title or role"
PARAM tags AS STRING LIKE "customer,vip" DESCRIPTION "Optional comma-separated tags"
PARAM notes AS STRING LIKE "Met at conference" DESCRIPTION "Optional notes about the contact"

DESCRIPTION "Adds a new contact to the directory. Collects contact information and saves to the contacts database."

' Validate required fields
IF firstname = "" THEN
    TALK "What is the contact's first name?"
    HEAR firstname AS STRING
END IF

IF lastname = "" THEN
    TALK "What is the contact's last name?"
    HEAR lastname AS STRING
END IF

IF email = "" THEN
    TALK "What is the contact's email address?"
    HEAR email AS EMAIL
END IF

' Generate contact ID
let contactid = "CON-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

' Set timestamps
let createdat = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"
let createdby = GET "session.user_email"

' Build full name
let fullname = firstname + " " + lastname

' Save the contact
SAVE "contacts.csv", contactid, firstname, lastname, fullname, email, phone, companyname, jobtitle, tags, notes, createdby, createdat

' Store in bot memory
SET BOT MEMORY "last_contact", contactid

' If company provided, check if it exists and add if not
IF companyname != "" THEN
    let existingcompany = FIND "companies.csv", "name=" + companyname
    let companycount = AGGREGATE "COUNT", existingcompany, "id"

    IF companycount = 0 THEN
        let companyid = "COMP-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)
        SAVE "companies.csv", companyid, companyname, createdat
        TALK "📝 Note: Company '" + companyname + "' was also added to the directory."
    END IF
END IF

' Log activity
let activity = "Contact created: " + fullname
SAVE "contact_activities.csv", contactid, activity, createdby, createdat

' Respond to user
TALK "✅ **Contact Added Successfully!**"
TALK ""
TALK "**Contact Details:**"
TALK "📋 **ID:** " + contactid
TALK "👤 **Name:** " + fullname
TALK "📧 **Email:** " + email

IF phone != "" THEN
    TALK "📱 **Phone:** " + phone
END IF

IF companyname != "" THEN
    TALK "🏢 **Company:** " + companyname
END IF

IF jobtitle != "" THEN
    TALK "💼 **Title:** " + jobtitle
END IF

IF tags != "" THEN
    TALK "🏷️ **Tags:** " + tags
END IF

TALK ""
TALK "You can find this contact anytime by searching for their name or email."
