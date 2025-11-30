PARAM dealname AS STRING LIKE "Acme Corp Enterprise License" DESCRIPTION "Name of the deal or opportunity"
PARAM companyname AS STRING LIKE "Acme Corporation" DESCRIPTION "Company or account name"
PARAM contactemail AS STRING LIKE "john@acme.com" DESCRIPTION "Primary contact email"
PARAM dealvalue AS NUMBER LIKE 50000 DESCRIPTION "Estimated deal value in dollars"
PARAM stage AS STRING LIKE "Lead" DESCRIPTION "Initial stage: Lead, Qualified, Proposal, Negotiation"
PARAM closedate AS STRING LIKE "2025-03-30" DESCRIPTION "Expected close date"
PARAM notes AS STRING LIKE "Met at trade show" DESCRIPTION "Optional notes about the deal"

DESCRIPTION "Creates a new sales deal in the pipeline. Collects deal information and saves to the deals database."

' Validate required fields
IF dealname = "" THEN
    TALK "What is the name of this deal?"
    HEAR dealname AS STRING
END IF

IF companyname = "" THEN
    TALK "What company is this deal with?"
    HEAR companyname AS STRING
END IF

IF contactemail = "" THEN
    TALK "What is the primary contact's email?"
    HEAR contactemail AS EMAIL
END IF

IF dealvalue = 0 THEN
    TALK "What is the estimated deal value?"
    HEAR dealvalue AS NUMBER
END IF

' Set defaults for optional fields
IF stage = "" THEN
    stage = "Lead"
END IF

IF closedate = "" THEN
    closedate = FORMAT DATEADD(TODAY(), 30, "day") AS "YYYY-MM-DD"
END IF

' Generate deal ID
let dealid = "DEAL-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)

' Set timestamps and owner
let createdat = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"
let ownerid = GET "session.user_id"
let owneremail = GET "session.user_email"
let probability = 10

' Set probability based on stage
IF stage = "Qualified" THEN
    probability = 25
ELSE IF stage = "Proposal" THEN
    probability = 50
ELSE IF stage = "Negotiation" THEN
    probability = 75
END IF

' Calculate weighted value
let weightedvalue = dealvalue * probability / 100

' Save the deal
SAVE "deals.csv", dealid, dealname, companyname, contactemail, dealvalue, stage, closedate, probability, weightedvalue, notes, ownerid, owneremail, createdat

' Store in bot memory
SET BOT MEMORY "last_deal", dealid

' Log activity
let activity = "Deal created: " + dealname
SAVE "deal_activities.csv", dealid, activity, owneremail, createdat

' Respond to user
TALK "✅ **Deal Created Successfully!**"
TALK ""
TALK "**Deal Details:**"
TALK "📋 **ID:** " + dealid
TALK "💼 **Name:** " + dealname
TALK "🏢 **Company:** " + companyname
TALK "📧 **Contact:** " + contactemail
TALK "💰 **Value:** $" + FORMAT dealvalue AS "#,##0"
TALK "📊 **Stage:** " + stage + " (" + probability + "% probability)"
TALK "📅 **Expected Close:** " + closedate
TALK "💵 **Weighted Value:** $" + FORMAT weightedvalue AS "#,##0"
TALK ""
TALK "📝 You can update this deal anytime by saying **update deal " + dealid + "**"
