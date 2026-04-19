PARAM to AS EMAIL LIKE "client@company.com" DESCRIPTION "Email address to send proposal to"
PARAM template AS STRING LIKE "proposal-template.docx" DESCRIPTION "Proposal template file to use"
PARAM opportunity AS STRING LIKE "OPP-12345" DESCRIPTION "Opportunity ID to link proposal to"

DESCRIPTION "Generate and send a proposal document based on opportunity and conversation history"

company = QUERY "SELECT Company FROM Opportunities WHERE Id = ${opportunity}"

IF NOT company THEN
    TALK "Could not find opportunity. Please provide a valid opportunity ID."
    RETURN NULL
END IF

doc = FILL template

subject = REWRITE "Based on this ${history}, generate a subject for a proposal email to ${company}"
contents = REWRITE "Based on this ${history}, and ${subject}, generate the email body for ${to}, signed by ${user}, including key points from our proposal"

proposalpath = ".gbdrive/Proposals/${company}-proposal.docx"

CALL "/files/upload", proposalpath, doc
CALL "/files/permissions", proposalpath, "sales-team", "edit"

WITH activity
    opportunityId = opportunity
    type = "email_sent"
    subject = subject
    description = "Proposal sent to " + company
    date = NOW()
END WITH

CALL "/crm/activities/create", activity

CALL "/comm/email/send", to, subject, contents, doc

WITH proposalLog
    timestamp = NOW()
    opp = opportunity
    companyName = company
    recipient = to
    templateUsed = template
    status = "sent"
END WITH

SAVE "proposal_log.csv", proposalLog

SET BOT MEMORY "last_proposal", opportunity

TALK "Proposal sent to " + to
TALK "Company: " + company
TALK "Template: " + template
TALK "Opportunity: " + opportunity

RETURN opportunity
