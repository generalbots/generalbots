PARAM searchterm AS STRING LIKE "john" DESCRIPTION "Name, email, company, or phone to search for"
PARAM searchby AS STRING LIKE "all" DESCRIPTION "Optional: Filter by field - all, name, email, company, phone"

DESCRIPTION "Searches the contact directory by name, email, company, or phone number. Returns matching contacts with their details."

' Validate search term
IF searchterm = "" THEN
    TALK "What would you like to search for? You can enter a name, email, company, or phone number."
    HEAR searchterm AS STRING
END IF

IF searchterm = "" THEN
    TALK "I need a search term to find contacts."
    RETURN
END IF

' Set default search scope
IF searchby = "" THEN
    searchby = "all"
END IF

TALK "🔍 Searching contacts for: **" + searchterm + "**..."
TALK ""

' Search based on field
let results = []

IF searchby = "all" OR searchby = "name" THEN
    let nameresults = FIND "contacts.csv", "fullname LIKE " + searchterm
    results = MERGE results, nameresults
END IF

IF searchby = "all" OR searchby = "email" THEN
    let emailresults = FIND "contacts.csv", "email LIKE " + searchterm
    results = MERGE results, emailresults
END IF

IF searchby = "all" OR searchby = "company" THEN
    let companyresults = FIND "contacts.csv", "companyname LIKE " + searchterm
    results = MERGE results, companyresults
END IF

IF searchby = "all" OR searchby = "phone" THEN
    let phoneresults = FIND "contacts.csv", "phone LIKE " + searchterm
    results = MERGE results, phoneresults
END IF

' Count results
let resultcount = AGGREGATE "COUNT", results, "contactid"

IF resultcount = 0 THEN
    TALK "❌ No contacts found matching **" + searchterm + "**"
    TALK ""
    TALK "💡 **Tips:**"
    TALK "• Try a partial name or email"
    TALK "• Check spelling"
    TALK "• Search by company name"
    TALK ""
    TALK "Would you like to add a new contact instead?"
    RETURN
END IF

' Display results
IF resultcount = 1 THEN
    TALK "✅ Found 1 contact:"
ELSE
    TALK "✅ Found " + resultcount + " contacts:"
END IF

TALK ""

' Show each result
FOR EACH contact IN results
    TALK "---"
    TALK "👤 **" + contact.fullname + "**"
    TALK "📧 " + contact.email

    IF contact.phone != "" THEN
        TALK "📱 " + contact.phone
    END IF

    IF contact.companyname != "" THEN
        TALK "🏢 " + contact.companyname
    END IF

    IF contact.jobtitle != "" THEN
        TALK "💼 " + contact.jobtitle
    END IF

    IF contact.tags != "" THEN
        TALK "🏷️ " + contact.tags
    END IF

    TALK "📋 ID: " + contact.contactid
    TALK ""
NEXT contact

' Store first result in memory for quick reference
IF resultcount > 0 THEN
    let firstcontact = FIRST results
    SET BOT MEMORY "last_contact", firstcontact.contactid
    SET BOT MEMORY "last_search", searchterm
END IF

TALK "---"
TALK "💡 To view more details or update a contact, just ask!"
