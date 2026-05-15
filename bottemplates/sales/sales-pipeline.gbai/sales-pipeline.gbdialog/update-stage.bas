PARAM dealid AS STRING LIKE "DEAL-20250115-1234" DESCRIPTION "The deal ID to update"
PARAM newstage AS STRING LIKE "Qualified" DESCRIPTION "New stage: Lead, Qualified, Proposal, Negotiation, Closed Won, Closed Lost"
PARAM reason AS STRING LIKE "Budget confirmed" DESCRIPTION "Optional reason for stage change"

DESCRIPTION "Updates the stage of an existing deal in the pipeline. Automatically updates probability and logs the activity."

' Validate deal ID
IF dealid = "" THEN
    TALK "Which deal would you like to update? Please provide the deal ID."
    HEAR dealid AS STRING
END IF

IF dealid = "" THEN
    let lastdeal = GET BOT MEMORY("last_deal")
    IF lastdeal != "" THEN
        TALK "Would you like to update the last deal: " + lastdeal + "?"
        HEAR confirm AS BOOLEAN
        IF confirm THEN
            dealid = lastdeal
        ELSE
            TALK "Please provide a deal ID to update."
            RETURN
        END IF
    ELSE
        TALK "I need a deal ID to update."
        RETURN
    END IF
END IF

' Validate new stage
IF newstage = "" THEN
    TALK "What stage should this deal move to?"
    TALK "Options: Lead, Qualified, Proposal, Negotiation, Closed Won, Closed Lost"
    HEAR newstage AS STRING
END IF

' Normalize stage name
newstage = UCASE(LEFT(newstage, 1)) + LCASE(MID(newstage, 2))

' Validate stage is valid
let validstages = "Lead,Qualified,Proposal,Negotiation,Closed Won,Closed Lost"
IF INSTR(validstages, newstage) = 0 THEN
    TALK "Invalid stage. Please use: Lead, Qualified, Proposal, Negotiation, Closed Won, or Closed Lost"
    RETURN
END IF

' Determine probability based on stage
let probability = 10

IF newstage = "Lead" THEN
    probability = 10
ELSE IF newstage = "Qualified" THEN
    probability = 25
ELSE IF newstage = "Proposal" THEN
    probability = 50
ELSE IF newstage = "Negotiation" THEN
    probability = 75
ELSE IF newstage = "Closed Won" THEN
    probability = 100
ELSE IF newstage = "Closed Lost" THEN
    probability = 0
END IF

' Get current timestamp
let updatedat = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"
let useremail = GET "session.user_email"

' Update the deal
UPDATE "deals.csv", "dealid=" + dealid, stage, probability, updatedat

' Log activity
let activity = "Stage changed to " + newstage
IF reason != "" THEN
    activity = activity + " - " + reason
END IF

SAVE "deal_activities.csv", dealid, activity, useremail, updatedat

' Store last updated deal
SET BOT MEMORY "last_deal", dealid

' Respond based on stage
IF newstage = "Closed Won" THEN
    TALK "🎉 **Congratulations! Deal Won!**"
    TALK ""
    TALK "**Deal " + dealid + "** has been marked as **Closed Won**!"
    TALK "Great work closing this deal!"
    TALK ""
    TALK "📊 Probability: 100%"
ELSE IF newstage = "Closed Lost" THEN
    TALK "📋 **Deal Closed Lost**"
    TALK ""
    TALK "**Deal " + dealid + "** has been marked as **Closed Lost**."
    IF reason != "" THEN
        TALK "📝 Reason: " + reason
    END IF
    TALK ""
    TALK "Don't worry - analyze what happened and apply the learnings to future deals!"
ELSE
    TALK "✅ **Deal Stage Updated!**"
    TALK ""
    TALK "**Deal " + dealid + "** moved to **" + newstage + "**"
    TALK "📊 New Probability: " + probability + "%"
    IF reason != "" THEN
        TALK "📝 Note: " + reason
    END IF
    TALK ""
    TALK "Keep the momentum going!"
END IF

' Suggest next actions based on stage
IF newstage = "Qualified" THEN
    TALK ""
    TALK "💡 **Next Steps:**"
    TALK "• Prepare a tailored proposal"
    TALK "• Schedule a demo or presentation"
    TALK "• Identify key stakeholders"
ELSE IF newstage = "Proposal" THEN
    TALK ""
    TALK "💡 **Next Steps:**"
    TALK "• Follow up within 48 hours"
    TALK "• Address any questions or concerns"
    TALK "• Identify decision timeline"
ELSE IF newstage = "Negotiation" THEN
    TALK ""
    TALK "💡 **Next Steps:**"
    TALK "• Clarify any contract terms"
    TALK "• Prepare for potential objections"
    TALK "• Confirm decision makers and process"
END IF
