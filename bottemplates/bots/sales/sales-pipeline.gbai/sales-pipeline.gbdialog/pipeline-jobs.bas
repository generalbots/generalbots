PARAM jobname AS STRING DESCRIPTION "Name of the scheduled job to execute"

DESCRIPTION "Scheduled jobs for pipeline maintenance, reminders, and reporting. Run automatically based on configured schedules."

' ============================================================================
' DAILY PIPELINE REPORT - Runs at 8:00 AM every day
' ============================================================================

IF jobname = "daily report" THEN
    SET SCHEDULE "0 8 * * *"

    ' Get pipeline summary
    let deals = FIND "deals.csv"
    let opendeals = FILTER deals, "stage!=Closed Won AND stage!=Closed Lost"

    ' Calculate metrics
    let totalvalue = AGGREGATE "SUM", opendeals, "dealvalue"
    let weightedvalue = AGGREGATE "SUM", opendeals, "weightedvalue"
    let dealcount = AGGREGATE "COUNT", opendeals, "dealid"

    ' Count by stage
    let leads = FILTER opendeals, "stage=Lead"
    let qualified = FILTER opendeals, "stage=Qualified"
    let proposals = FILTER opendeals, "stage=Proposal"
    let negotiations = FILTER opendeals, "stage=Negotiation"

    let leadcount = AGGREGATE "COUNT", leads, "dealid"
    let qualifiedcount = AGGREGATE "COUNT", qualified, "dealid"
    let proposalcount = AGGREGATE "COUNT", proposals, "dealid"
    let negotiationcount = AGGREGATE "COUNT", negotiations, "dealid"

    ' Build report
    let reportdate = FORMAT TODAY() AS "MMMM DD, YYYY"
    let subject = "Daily Pipeline Report - " + reportdate

    let message = "Good morning!\n\n"
    message = message + "Here is your daily pipeline summary:\n\n"
    message = message + "PIPELINE OVERVIEW\n"
    message = message + "================\n"
    message = message + "Total Open Deals: " + dealcount + "\n"
    message = message + "Total Pipeline Value: $" + FORMAT totalvalue AS "#,##0" + "\n"
    message = message + "Weighted Value: $" + FORMAT weightedvalue AS "#,##0" + "\n\n"
    message = message + "BY STAGE\n"
    message = message + "========\n"
    message = message + "Lead: " + leadcount + " deals\n"
    message = message + "Qualified: " + qualifiedcount + " deals\n"
    message = message + "Proposal: " + proposalcount + " deals\n"
    message = message + "Negotiation: " + negotiationcount + " deals\n\n"
    message = message + "Have a great selling day!\n"
    message = message + "- Sales Pipeline Bot"

    SEND MAIL "sales-team@company.com", subject, message

    TALK "Daily pipeline report sent"
END IF

' ============================================================================
' STALE DEAL ALERTS - Runs at 9:00 AM every day
' ============================================================================

IF jobname = "stale alerts" THEN
    SET SCHEDULE "0 9 * * *"

    ' Find deals not updated in 7+ days
    let cutoffdate = FORMAT DATEADD(TODAY(), -7, "day") AS "YYYY-MM-DD"
    let deals = FIND "deals.csv"
    let staledeals = FILTER deals, "updatedat<" + cutoffdate + " AND stage!=Closed Won AND stage!=Closed Lost"

    let stalecount = AGGREGATE "COUNT", staledeals, "dealid"

    IF stalecount > 0 THEN
        FOR EACH deal IN staledeals
            let stalealert = "Your deal '" + deal.dealname + "' has not been updated in over 7 days.\n"
            stalealert = stalealert + "Deal ID: " + deal.dealid + "\n"
            stalealert = stalealert + "Current Stage: " + deal.stage + "\n"
            stalealert = stalealert + "Value: $" + FORMAT deal.dealvalue AS "#,##0" + "\n\n"
            stalealert = stalealert + "Please review and update this deal or mark it as closed."

            SEND MAIL deal.owneremail, "Action Required: Stale Deal Alert", stalealert
        NEXT deal

        TALK "Sent " + stalecount + " stale deal alerts"
    ELSE
        TALK "No stale deals found"
    END IF
END IF

' ============================================================================
' CLOSE DATE REMINDERS - Runs at 8:30 AM every day
' ============================================================================

IF jobname = "close reminders" THEN
    SET SCHEDULE "30 8 * * *"

    ' Find deals closing in next 7 days
    let today = FORMAT TODAY() AS "YYYY-MM-DD"
    let nextweek = FORMAT DATEADD(TODAY(), 7, "day") AS "YYYY-MM-DD"
    let deals = FIND "deals.csv"
    let upcoming = FILTER deals, "closedate>=" + today + " AND closedate<=" + nextweek + " AND stage!=Closed Won AND stage!=Closed Lost"

    let upcomingcount = AGGREGATE "COUNT", upcoming, "dealid"

    IF upcomingcount > 0 THEN
        FOR EACH deal IN upcoming
            let daysuntil = DATEDIFF(TODAY(), deal.closedate, "day")
            let reminder = "Deal Closing Soon!\n\n"
            reminder = reminder + "Your deal '" + deal.dealname + "' is expected to close in " + daysuntil + " days.\n\n"
            reminder = reminder + "Deal Details:\n"
            reminder = reminder + "- Company: " + deal.companyname + "\n"
            reminder = reminder + "- Value: $" + FORMAT deal.dealvalue AS "#,##0" + "\n"
            reminder = reminder + "- Stage: " + deal.stage + "\n"
            reminder = reminder + "- Close Date: " + deal.closedate + "\n\n"
            reminder = reminder + "Make sure to follow up and push for the close!"

            SEND MAIL deal.owneremail, "Reminder: Deal Closing in " + daysuntil + " Days", reminder
        NEXT deal

        TALK "Sent " + upcomingcount + " close date reminders"
    ELSE
        TALK "No deals closing in the next 7 days"
    END IF
END IF

' ============================================================================
' WEEKLY FORECAST REPORT - Runs at 9:00 AM every Monday
' ============================================================================

IF jobname = "weekly forecast" THEN
    SET SCHEDULE "0 9 * * 1"

    ' Get this month's deals
    let monthstart = FORMAT DATEADD(TODAY(), 0, "month") AS "YYYY-MM-01"
    let monthend = FORMAT DATEADD(TODAY(), 1, "month") AS "YYYY-MM-01"
    let deals = FIND "deals.csv"
    let monthdeals = FILTER deals, "closedate>=" + monthstart + " AND closedate<" + monthend

    ' Calculate forecast
    let totalforecast = AGGREGATE "SUM", monthdeals, "weightedvalue"
    let wondeals = FILTER monthdeals, "stage=Closed Won"
    let closedvalue = AGGREGATE "SUM", wondeals, "dealvalue"

    ' Build forecast report
    let reportdate = FORMAT TODAY() AS "MMMM DD, YYYY"
    let monthname = FORMAT TODAY() AS "MMMM YYYY"
    let subject = "Weekly Sales Forecast - " + reportdate

    let message = "Weekly Forecast Report\n"
    message = message + "======================\n\n"
    message = message + "Month: " + monthname + "\n\n"
    message = message + "Forecast Summary:\n"
    message = message + "- Weighted Forecast: $" + FORMAT totalforecast AS "#,##0" + "\n"
    message = message + "- Already Closed: $" + FORMAT closedvalue AS "#,##0" + "\n"
    message = message + "- Remaining to Close: $" + FORMAT (totalforecast - closedvalue) AS "#,##0" + "\n\n"
    message = message + "Keep pushing to hit your targets!\n"
    message = message + "- Sales Pipeline Bot"

    SEND MAIL "sales-team@company.com", subject, message

    TALK "Weekly forecast report sent"
END IF

' ============================================================================
' SETUP SCHEDULES - Run once to configure all jobs
' ============================================================================

IF jobname = "setup schedules" THEN
    TALK "Configuring pipeline scheduled jobs..."
    TALK ""
    TALK "📅 **Scheduled Jobs:**"
    TALK "• Daily Pipeline Report: 8:00 AM daily"
    TALK "• Stale Deal Alerts: 9:00 AM daily"
    TALK "• Close Date Reminders: 8:30 AM daily"
    TALK "• Weekly Forecast: 9:00 AM Mondays"
    TALK ""
    TALK "✅ All pipeline schedules configured!"
END IF
