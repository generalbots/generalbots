PARAM firstname AS STRING LIKE "John" DESCRIPTION "First name of the contact"
PARAM lastname AS STRING LIKE "Smith" DESCRIPTION "Last name of the contact"
PARAM email AS EMAIL LIKE "john.smith@company.com" DESCRIPTION "Email address"
PARAM phone AS PHONE LIKE "+1-555-123-4567" DESCRIPTION "Phone number"
PARAM companyname AS STRING LIKE "Acme Corporation" DESCRIPTION "Company or organization"
PARAM jobtitle AS STRING LIKE "Sales Manager" DESCRIPTION "Job title or role"
PARAM tags AS STRING LIKE "customer,vip" DESCRIPTION "Comma-separated tags" OPTIONAL
PARAM notes AS STRING LIKE "Met at conference" DESCRIPTION "Notes about the contact" OPTIONAL

DESCRIPTION "Add a new contact to the directory with contact information"

contactid = "CON-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
createdat = FORMAT(NOW(), "YYYY-MM-DD HH:mm:ss")
createdby = GET "session.user_email"
fullname = firstname + " " + lastname

SAVE "contacts.csv", contactid, firstname, lastname, fullname, email, phone, companyname, jobtitle, tags, notes, createdby, createdat

SET BOT MEMORY "last_contact", contactid

IF companyname THEN
    existingcompany = FIND "companies.csv", "name=" + companyname
    companycount = AGGREGATE "COUNT", existingcompany, "id"

    IF companycount = 0 THEN
        companyid = "COMP-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
        SAVE "companies.csv", companyid, companyname, createdat
    END IF
END IF

WITH activity
    contactid = contactid
    action = "Contact created: " + fullname
    createdby = createdby
    createdat = createdat
END WITH

SAVE "contact_activities.csv", activity

TALK "Contact added: " + fullname
TALK "ID: " + contactid
TALK "Email: " + email

IF phone THEN
    TALK "Phone: " + phone
END IF

IF companyname THEN
    TALK "Company: " + companyname
END IF

IF jobtitle THEN
    TALK "Title: " + jobtitle
END IF

RETURN contactid
