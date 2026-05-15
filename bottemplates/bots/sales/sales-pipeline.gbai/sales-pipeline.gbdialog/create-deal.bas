PARAM dealname AS STRING LIKE "Acme Corp Enterprise License" DESCRIPTION "Name of the deal or opportunity"
PARAM companyname AS STRING LIKE "Acme Corporation" DESCRIPTION "Company or account name"
PARAM contactemail AS EMAIL LIKE "john@acme.com" DESCRIPTION "Primary contact email"
PARAM dealvalue AS MONEY LIKE 50000 DESCRIPTION "Estimated deal value in dollars"
PARAM stage AS STRING LIKE "Lead" DESCRIPTION "Initial stage: Lead, Qualified, Proposal, Negotiation" OPTIONAL
PARAM closedate AS DATE LIKE "2025-03-30" DESCRIPTION "Expected close date" OPTIONAL
PARAM notes AS STRING LIKE "Met at trade show" DESCRIPTION "Notes about the deal" OPTIONAL

DESCRIPTION "Create a new sales deal in the pipeline with deal information and value tracking"

IF NOT stage THEN
    stage = "Lead"
END IF

IF NOT closedate THEN
    closedate = DATEADD(TODAY(), 30, "day")
END IF

dealid = "DEAL-" + FORMAT(NOW(), "YYYYMMDD") + "-" + FORMAT(RANDOM(1000, 9999))
createdat = FORMAT(NOW(), "YYYY-MM-DD HH:mm:ss")
ownerid = GET "session.user_id"
owneremail = GET "session.user_email"

probability = 10
IF stage = "Qualified" THEN
    probability = 25
ELSE IF stage = "Proposal" THEN
    probability = 50
ELSE IF stage = "Negotiation" THEN
    probability = 75
END IF

weightedvalue = dealvalue * probability / 100

WITH deal
    id = dealid
    name = dealname
    company = companyname
    contact = contactemail
    value = dealvalue
    currentStage = stage
    expectedClose = closedate
    prob = probability
    weighted = weightedvalue
    dealNotes = notes
    owner = ownerid
    ownerEmail = owneremail
    created = createdat
END WITH

SAVE "deals.csv", deal

SET BOT MEMORY "last_deal", dealid

WITH dealActivity
    dealId = dealid
    action = "Deal created: " + dealname
    user = owneremail
    timestamp = createdat
END WITH

SAVE "deal_activities.csv", dealActivity

TALK "Deal created: " + dealname
TALK "ID: " + dealid
TALK "Company: " + companyname
TALK "Value: $" + FORMAT(dealvalue, "#,##0")
TALK "Stage: " + stage + " (" + probability + "% probability)"
TALK "Expected Close: " + closedate
TALK "Weighted Value: $" + FORMAT(weightedvalue, "#,##0")

RETURN dealid
