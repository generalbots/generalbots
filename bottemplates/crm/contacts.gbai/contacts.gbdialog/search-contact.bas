PARAM searchterm AS STRING LIKE "john" DESCRIPTION "Name, email, company, or phone to search for"
PARAM searchby AS STRING LIKE "all" DESCRIPTION "Filter by field: all, name, email, company, phone"

DESCRIPTION "Search contact directory by name, email, company, or phone number"

IF NOT searchby THEN
    searchby = "all"
END IF

TALK "Searching contacts for: " + searchterm

results = []

IF searchby = "all" OR searchby = "name" THEN
    nameresults = FIND "contacts.csv", "fullname LIKE " + searchterm
    results = MERGE results, nameresults
END IF

IF searchby = "all" OR searchby = "email" THEN
    emailresults = FIND "contacts.csv", "email LIKE " + searchterm
    results = MERGE results, emailresults
END IF

IF searchby = "all" OR searchby = "company" THEN
    companyresults = FIND "contacts.csv", "companyname LIKE " + searchterm
    results = MERGE results, companyresults
END IF

IF searchby = "all" OR searchby = "phone" THEN
    phoneresults = FIND "contacts.csv", "phone LIKE " + searchterm
    results = MERGE results, phoneresults
END IF

resultcount = UBOUND(results)

IF resultcount = 0 THEN
    TALK "No contacts found matching: " + searchterm
    RETURN
END IF

TALK "Found " + resultcount + " contact(s):"

FOR EACH contact IN results
    TALK "---"
    TALK "**" + contact.fullname + "**"
    TALK contact.email

    IF contact.phone <> "" THEN
        TALK contact.phone
    END IF

    IF contact.companyname <> "" THEN
        TALK contact.companyname
    END IF

    IF contact.jobtitle <> "" THEN
        TALK contact.jobtitle
    END IF

    TALK "ID: " + contact.contactid
NEXT

IF resultcount > 0 THEN
    firstcontact = FIRST results
    SET BOT MEMORY "last_contact", firstcontact.contactid
    SET BOT MEMORY "last_search", searchterm
END IF

RETURN results
