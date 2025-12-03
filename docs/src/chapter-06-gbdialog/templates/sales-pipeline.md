# Sales Pipeline Template

The sales pipeline template provides a complete CRM (Customer Relationship Management) system for managing deals, tracking opportunities through sales stages, and generating revenue forecasts.

## Topic: Sales Pipeline & Deal Management

This template is perfect for:
- Sales teams tracking deals
- Revenue forecasting
- Pipeline management
- Win/loss analysis
- Sales performance reporting

## The Code

```basic
ADD TOOL "create-deal"
ADD TOOL "update-stage"
ADD TOOL "list-deals"
ADD TOOL "deal-details"
ADD TOOL "pipeline-report"
ADD TOOL "forecast-revenue"

USE KB "sales-pipeline.gbkb"

SET CONTEXT "sales pipeline" AS "You are a sales assistant helping manage the sales pipeline. Help with creating new deals, updating deal stages, viewing pipeline status, generating sales forecasts, and analyzing win/loss rates."

CLEAR SUGGESTIONS

ADD SUGGESTION "newdeal" AS "Create a new deal"
ADD SUGGESTION "pipeline" AS "Show my pipeline"
ADD SUGGESTION "update" AS "Update a deal stage"
ADD SUGGESTION "forecast" AS "View sales forecast"
ADD SUGGESTION "report" AS "Generate pipeline report"

BEGIN TALK
**Sales Pipeline Manager**

I can help you with:
• Create new deals and opportunities
• View and manage your pipeline
• Update deal stages
• Generate sales forecasts
• Pipeline analytics and reports
• Track win/loss rates

Select an option or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are a sales pipeline assistant.

Pipeline stages:
- Lead: Initial contact, not qualified
- Qualified: Budget, authority, need, timeline confirmed
- Proposal: Quote sent
- Negotiation: Active discussions
- Closed Won: Successfully closed
- Closed Lost: Lost or no decision

Always encourage sales reps and provide actionable insights.
Confirm changes before saving.
Use currency format for amounts.
END SYSTEM PROMPT
```

## Sample Dialogs

These conversations show how the sales pipeline template works in real-world scenarios.

### Dialog 1: Creating a New Deal

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💼</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Sales Pipeline</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Sales Pipeline</div>
      <p><strong>Sales Pipeline Manager</strong></p>
      <p>I can help you with:</p>
      <p>• Create new deals and opportunities<br>• View and manage your pipeline<br>• Update deal stages<br>• Generate sales forecasts</p>
      <div class="wa-time">09:00</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to add a new deal</p>
      <div class="wa-time">09:01 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Great! Let's create a new deal. 🎯</p>
      <p>What's the company name?</p>
      <div class="wa-time">09:01</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Acme Corporation</p>
      <div class="wa-time">09:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>Got it. What's the deal value?</p>
      <div class="wa-time">09:02</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>$50,000</p>
      <div class="wa-time">09:02 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>What's the expected close date?</p>
      <div class="wa-time">09:02</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>End of March</p>
      <div class="wa-time">09:03 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Deal Created!</strong></p>
      <p>🏢 <strong>Company:</strong> Acme Corporation</p>
      <p>💰 <strong>Value:</strong> $50,000</p>
      <p>📅 <strong>Expected Close:</strong> March 31, 2025</p>
      <p>📊 <strong>Stage:</strong> Lead</p>
      <p>🎫 <strong>Deal ID:</strong> DEAL-2025-0142</p>
      <p>Good luck! 🍀 Keep me updated on the progress.</p>
      <div class="wa-time">09:03</div>
    </div>
  </div>
</div>

### Dialog 2: Viewing Pipeline

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💼</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Sales Pipeline</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Show my pipeline</p>
      <div class="wa-time">14:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Your Sales Pipeline</strong></p>
      <p></p>
      <p><strong>🔵 Lead</strong> (3 deals - $85,000)</p>
      <p>• Acme Corp - $50,000</p>
      <p>• TechStart Inc - $25,000</p>
      <p>• Global Ltd - $10,000</p>
      <p></p>
      <p><strong>🟡 Qualified</strong> (2 deals - $120,000)</p>
      <p>• MegaCorp - $80,000</p>
      <p>• Innovate Co - $40,000</p>
      <p></p>
      <p><strong>🟠 Proposal</strong> (1 deal - $65,000)</p>
      <p>• Enterprise Solutions - $65,000</p>
      <p></p>
      <p><strong>🔴 Negotiation</strong> (1 deal - $150,000)</p>
      <p>• BigClient Inc - $150,000 ⭐</p>
      <p></p>
      <p><strong>Total Pipeline:</strong> $420,000</p>
      <p><strong>Weighted Forecast:</strong> $178,500</p>
      <div class="wa-time">14:00</div>
    </div>
  </div>
</div>

### Dialog 3: Update Deal Stage

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💼</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Sales Pipeline</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Move Acme Corp to Qualified</p>
      <div class="wa-time">16:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🎯 <strong>BANT Qualification Checklist:</strong></p>
      <p>Before moving to Qualified, please confirm:</p>
      <p>✅ <strong>Budget:</strong> Do they have budget approved?</p>
      <p>✅ <strong>Authority:</strong> Are you speaking with a decision maker?</p>
      <p>✅ <strong>Need:</strong> Is there a clear business need?</p>
      <p>✅ <strong>Timeline:</strong> Do they have a timeline for purchase?</p>
      <p>Have all these been confirmed?</p>
      <div class="wa-time">16:30</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Yes, all confirmed</p>
      <div class="wa-time">16:31 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>Deal Updated!</strong></p>
      <p>🏢 <strong>Acme Corporation</strong></p>
      <p>📊 Lead → <strong>Qualified</strong></p>
      <p>💰 $50,000</p>
      <p></p>
      <p>Great progress! 🎉 Next step: Send a proposal.</p>
      <p>Win probability increased to 40%.</p>
      <div class="wa-time">16:31</div>
    </div>
  </div>
</div>

### Dialog 4: Revenue Forecast

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">💼</div>
    <div class="wa-header-info">
      <div class="wa-header-name">Sales Pipeline</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>What's my forecast for this quarter?</p>
      <div class="wa-time">10:00 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📈 <strong>Q1 2025 Revenue Forecast</strong></p>
      <p></p>
      <p><strong>By Stage:</strong></p>
      <p>• Negotiation (80%): $120,000</p>
      <p>• Proposal (50%): $32,500</p>
      <p>• Qualified (40%): $48,000</p>
      <p>• Lead (20%): $17,000</p>
      <p></p>
      <p><strong>Weighted Forecast:</strong> $217,500</p>
      <p><strong>Best Case:</strong> $420,000</p>
      <p><strong>Committed:</strong> $150,000</p>
      <p></p>
      <p><strong>Quota:</strong> $200,000</p>
      <p><strong>Attainment:</strong> 109% (forecasted)</p>
      <p></p>
      <p>🎯 You're on track to exceed quota! Focus on closing the BigClient deal to lock in your numbers.</p>
      <div class="wa-time">10:00</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register pipeline management tools |
| `USE KB` | Load sales methodology knowledge base |
| `SET CONTEXT` | Define sales assistant behavior |
| `ADD SUGGESTION` | Create quick action buttons |
| `BEGIN TALK` | Welcome message with options |
| `BEGIN SYSTEM PROMPT` | Sales stage definitions and guidelines |

## Pipeline Stages

| Stage | Win Probability | Description |
|-------|-----------------|-------------|
| Lead | 20% | Initial contact, not yet qualified |
| Qualified | 40% | BANT criteria confirmed |
| Proposal | 50% | Quote or proposal sent |
| Negotiation | 80% | Active deal discussions |
| Closed Won | 100% | Deal successfully closed |
| Closed Lost | 0% | Deal lost or abandoned |

## Template Structure

```
sales-pipeline.gbai/
├── sales-pipeline.gbdialog/
│   ├── start.bas           # Main entry point
│   ├── create-deal.bas     # New deal creation
│   ├── update-stage.bas    # Stage progression
│   ├── list-deals.bas      # Pipeline view
│   ├── deal-details.bas    # Individual deal info
│   ├── pipeline-report.bas # Analytics reports
│   └── forecast-revenue.bas # Revenue forecasting
├── sales-pipeline.gbdrive/
│   └── templates/          # Proposal templates
├── sales-pipeline.gbkb/
│   └── sales-methodology.md # Sales best practices
└── sales-pipeline.gbot/
    └── config.csv          # Bot configuration
```

## Create Deal Tool: create-deal.bas

```basic
PARAM company AS STRING LIKE "Acme Corp" DESCRIPTION "Company or account name"
PARAM value AS NUMBER LIKE 50000 DESCRIPTION "Deal value in dollars"
PARAM close_date AS DATE LIKE "2025-03-31" DESCRIPTION "Expected close date"
PARAM contact AS STRING DESCRIPTION "Primary contact name" OPTIONAL
PARAM notes AS STRING DESCRIPTION "Deal notes" OPTIONAL

DESCRIPTION "Create a new deal in the sales pipeline"

' Generate deal ID
dealId = "DEAL-" + FORMAT(NOW(), "YYYY") + "-" + FORMAT(RANDOM(1000, 9999))

' Get sales rep info
salesRep = USERNAME
salesRepEmail = FROM

' Create deal record
WITH deal
    id = dealId
    company = company
    value = value
    expected_close = close_date
    contact_name = contact
    notes = notes
    stage = "lead"
    probability = 20
    owner = salesRep
    owner_email = salesRepEmail
    created_at = NOW()
    updated_at = NOW()
END WITH

SAVE "deals.csv", deal

' Log activity
WITH activity
    deal_id = dealId
    type = "created"
    description = "Deal created with value $" + FORMAT(value, "#,##0")
    user = salesRep
    timestamp = NOW()
END WITH

SAVE "deal_activities.csv", activity

TALK "✅ **Deal Created!**"
TALK "🏢 **Company:** " + company
TALK "💰 **Value:** $" + FORMAT(value, "#,##0")
TALK "📅 **Expected Close:** " + FORMAT(close_date, "MMMM DD, YYYY")
TALK "📊 **Stage:** Lead"
TALK "🎫 **Deal ID:** " + dealId
TALK ""
TALK "Good luck! 🍀"

RETURN dealId
```

## Update Stage Tool: update-stage.bas

```basic
PARAM deal_id AS STRING LIKE "DEAL-2025-0142" DESCRIPTION "Deal ID or company name"
PARAM new_stage AS STRING LIKE "qualified" DESCRIPTION "New stage: lead, qualified, proposal, negotiation, closed_won, closed_lost"
PARAM reason AS STRING DESCRIPTION "Reason for stage change" OPTIONAL

DESCRIPTION "Update the stage of a deal in the pipeline"

' Find deal
deal = FIND "deals.csv", "id = '" + deal_id + "' OR LOWER(company) LIKE '%" + LOWER(deal_id) + "%'"

IF NOT deal THEN
    TALK "Deal not found. Please check the deal ID or company name."
    RETURN NULL
END IF

old_stage = deal.stage
new_stage_lower = LOWER(new_stage)

' Set probability based on stage
SELECT CASE new_stage_lower
    CASE "lead"
        probability = 20
    CASE "qualified"
        probability = 40
    CASE "proposal"
        probability = 50
    CASE "negotiation"
        probability = 80
    CASE "closed_won"
        probability = 100
    CASE "closed_lost"
        probability = 0
END SELECT

' Update deal
deal.stage = new_stage_lower
deal.probability = probability
deal.updated_at = NOW()

IF new_stage_lower = "closed_won" THEN
    deal.closed_date = NOW()
    deal.closed_value = deal.value
ELSE IF new_stage_lower = "closed_lost" THEN
    deal.closed_date = NOW()
    deal.lost_reason = reason
END IF

UPDATE "deals.csv", deal

' Log activity
WITH activity
    deal_id = deal.id
    type = "stage_change"
    description = "Stage changed: " + old_stage + " → " + new_stage_lower
    user = USERNAME
    timestamp = NOW()
END WITH

SAVE "deal_activities.csv", activity

' Format stage names
old_stage_display = PROPER(REPLACE(old_stage, "_", " "))
new_stage_display = PROPER(REPLACE(new_stage_lower, "_", " "))

TALK "✅ **Deal Updated!**"
TALK "🏢 **" + deal.company + "**"
TALK "📊 " + old_stage_display + " → **" + new_stage_display + "**"
TALK "💰 $" + FORMAT(deal.value, "#,##0")

IF new_stage_lower = "closed_won" THEN
    TALK ""
    TALK "🎉 Congratulations on closing the deal!"
ELSE IF new_stage_lower = "closed_lost" THEN
    TALK ""
    TALK "📝 Deal marked as lost. Keep pushing on the other opportunities!"
ELSE
    TALK ""
    TALK "Win probability: " + probability + "%"
END IF

RETURN deal.id
```

## Forecast Revenue Tool: forecast-revenue.bas

```basic
PARAM period AS STRING LIKE "this quarter" DESCRIPTION "Forecast period: this month, this quarter, this year"

DESCRIPTION "Generate revenue forecast based on pipeline and probabilities"

' Determine date range
IF INSTR(LOWER(period), "month") > 0 THEN
    start_date = DATE(YEAR(NOW()), MONTH(NOW()), 1)
    end_date = DATEADD(DATEADD(start_date, 1, "month"), -1, "day")
    period_name = FORMAT(NOW(), "MMMM YYYY")
ELSE IF INSTR(LOWER(period), "quarter") > 0 THEN
    quarter = INT((MONTH(NOW()) - 1) / 3) + 1
    start_date = DATE(YEAR(NOW()), (quarter - 1) * 3 + 1, 1)
    end_date = DATEADD(DATEADD(start_date, 3, "month"), -1, "day")
    period_name = "Q" + quarter + " " + YEAR(NOW())
ELSE
    start_date = DATE(YEAR(NOW()), 1, 1)
    end_date = DATE(YEAR(NOW()), 12, 31)
    period_name = YEAR(NOW())
END IF

' Get deals closing in period
deals = FIND "deals.csv", "expected_close >= '" + FORMAT(start_date, "YYYY-MM-DD") + "' AND expected_close <= '" + FORMAT(end_date, "YYYY-MM-DD") + "' AND stage NOT IN ('closed_won', 'closed_lost')"

' Calculate forecasts by stage
weighted_total = 0
best_case = 0
committed = 0

stages = ["negotiation", "proposal", "qualified", "lead"]
stage_totals = []

FOR EACH stage IN stages
    stage_deals = FILTER(deals, "stage = '" + stage + "'")
    stage_value = 0
    stage_weighted = 0
    
    FOR EACH deal IN stage_deals
        stage_value = stage_value + deal.value
        stage_weighted = stage_weighted + (deal.value * deal.probability / 100)
    NEXT
    
    best_case = best_case + stage_value
    weighted_total = weighted_total + stage_weighted
    
    IF stage = "negotiation" THEN
        committed = committed + stage_weighted
    END IF
    
    stage_totals[stage] = {value: stage_value, weighted: stage_weighted, prob: deals[1].probability}
NEXT

' Get closed won in period
closed = FIND "deals.csv", "closed_date >= '" + FORMAT(start_date, "YYYY-MM-DD") + "' AND stage = 'closed_won'"
closed_value = 0
FOR EACH deal IN closed
    closed_value = closed_value + deal.closed_value
NEXT

' Get quota
quota = GET BOT MEMORY("quota_" + USERNAME)
IF NOT quota THEN quota = 200000

attainment = ((closed_value + weighted_total) / quota) * 100

TALK "📈 **" + period_name + " Revenue Forecast**"
TALK ""
TALK "**By Stage:**"
TALK "• Negotiation (80%): $" + FORMAT(stage_totals["negotiation"].weighted, "#,##0")
TALK "• Proposal (50%): $" + FORMAT(stage_totals["proposal"].weighted, "#,##0")
TALK "• Qualified (40%): $" + FORMAT(stage_totals["qualified"].weighted, "#,##0")
TALK "• Lead (20%): $" + FORMAT(stage_totals["lead"].weighted, "#,##0")
TALK ""
TALK "**Weighted Forecast:** $" + FORMAT(weighted_total, "#,##0")
TALK "**Best Case:** $" + FORMAT(best_case, "#,##0")
TALK "**Committed:** $" + FORMAT(committed, "#,##0")
TALK "**Already Closed:** $" + FORMAT(closed_value, "#,##0")
TALK ""
TALK "**Quota:** $" + FORMAT(quota, "#,##0")
TALK "**Attainment:** " + FORMAT(attainment, "#,##0") + "% (forecasted)"

IF attainment >= 100 THEN
    TALK ""
    TALK "🎯 You're on track to exceed quota!"
ELSE IF attainment >= 80 THEN
    TALK ""
    TALK "📊 You're close! Focus on advancing your top deals."
ELSE
    TALK ""
    TALK "⚠️ You need more pipeline coverage. Time to prospect!"
END IF

RETURN {weighted: weighted_total, best_case: best_case, attainment: attainment}
```

## Customization Ideas

### Add Deal Scoring

```basic
' Calculate deal score based on various factors
score = 0

' Company size score
IF deal.company_size > 1000 THEN
    score = score + 20
ELSE IF deal.company_size > 100 THEN
    score = score + 10
END IF

' Budget confirmed
IF deal.budget_confirmed THEN
    score = score + 25
END IF

' Decision maker engaged
IF deal.decision_maker THEN
    score = score + 25
END IF

' Timeline urgency
IF DATEDIFF(deal.expected_close, NOW(), "days") < 30 THEN
    score = score + 20
END IF

' Competitor involved
IF deal.competitor THEN
    score = score - 10
END IF

deal.score = score
TALK "Deal Score: " + score + "/100"
```

### Add Activity Tracking

```basic
ADD TOOL "log-activity"

PARAM deal_id AS STRING DESCRIPTION "Deal ID"
PARAM activity_type AS STRING LIKE "call" DESCRIPTION "Type: call, email, meeting, demo, proposal"
PARAM notes AS STRING DESCRIPTION "Activity notes"

WITH activity
    deal_id = deal_id
    type = activity_type
    notes = notes
    user = USERNAME
    timestamp = NOW()
END WITH

SAVE "deal_activities.csv", activity

' Update deal's last activity date
UPDATE "deals.csv" SET last_activity = NOW() WHERE id = deal_id

TALK "✅ Activity logged for deal " + deal_id
```

### Add Win/Loss Analysis

```basic
ADD TOOL "win-loss-report"

won = FIND "deals.csv", "stage = 'closed_won' AND closed_date >= '" + start_date + "'"
lost = FIND "deals.csv", "stage = 'closed_lost' AND closed_date >= '" + start_date + "'"

won_count = UBOUND(won)
lost_count = UBOUND(lost)
win_rate = (won_count / (won_count + lost_count)) * 100

won_value = 0
FOR EACH deal IN won
    won_value = won_value + deal.value
NEXT

TALK "📊 **Win/Loss Analysis**"
TALK ""
TALK "**Win Rate:** " + FORMAT(win_rate, "#0.0") + "%"
TALK "**Deals Won:** " + won_count + " ($" + FORMAT(won_value, "#,##0") + ")"
TALK "**Deals Lost:** " + lost_count
TALK ""
TALK "**Top Loss Reasons:**"
' Aggregate loss reasons...
```

### Add Email Integration

```basic
' Send proposal email from pipeline
ADD TOOL "send-proposal"

PARAM deal_id AS STRING DESCRIPTION "Deal to send proposal for"

deal = FIND "deals.csv", "id = '" + deal_id + "'"

' Generate proposal from template
proposal = FILL "proposal-template.docx", deal

' Send email
SEND MAIL deal.contact_email, "Proposal for " + deal.company, 
    "Please find attached our proposal.\n\nBest regards,\n" + USERNAME,
    proposal

' Update deal stage
deal.stage = "proposal"
deal.proposal_sent = NOW()
UPDATE "deals.csv", deal

TALK "📧 Proposal sent to " + deal.contact_email
TALK "Deal moved to Proposal stage."
```

## Best Practices

1. **Keep Deals Updated**: Update deal stages promptly for accurate forecasting
2. **Log Activities**: Track all customer interactions
3. **Use BANT**: Qualify deals properly before advancing
4. **Clean Pipeline**: Remove stale deals regularly
5. **Review Weekly**: Check pipeline health and forecasts weekly

## Related Templates

- [crm/contacts.bas](./contacts.md) - Contact management
- [marketing.bas](./marketing.md) - Lead generation
- [store.bas](./store.md) - E-commerce integration

---

<style>
.wa-chat{background-color:#e5ddd5;border-radius:8px;padding:20px 15px;margin:20px 0;max-width:600px;font-family:-apple-system,BlinkMacSystemFont,'Segoe UI',Roboto,Helvetica,Arial,sans-serif;font-size:14px}
.wa-chat::after{content:'';display:table;clear:both}
.wa-message{clear:both;margin-bottom:10px;max-width:85%;position:relative}
.wa-message.user{float:right}
.wa-message.user .wa-bubble{background-color:#dcf8c6;border-radius:8px 0 8px 8px;margin-left:40px}
.wa-message.bot{float:left}
.wa-message.bot .wa-bubble{background-color:#fff;border-radius:0 8px 8px 8px;margin-right:40px}
.wa-bubble{padding:8px 12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-bubble p{margin:0 0 4px 0;line-height:1.4;color:#303030}
.wa-bubble p:last-child{margin-bottom:0}
.wa-time{font-size:11px;color:#8696a0;text-align:right;margin-top:4px}
.wa-message.user .wa-time{color:#61a05e}
.wa-sender{font-size:12px;font-weight:600;color:#06cf9c;margin-bottom:2px}
.wa-status.read::after{content:'✓✓';color:#53bdeb;margin-left:4px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>