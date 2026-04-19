# Lead Scoring Keywords

General Bots includes native lead scoring capabilities through BASIC keywords, enabling automated lead qualification, AI-enhanced scoring, and CRM integration directly from conversational flows.

<img src="../assets/gb-decorative-header.svg" alt="General Bots" style="max-height: 100px; width: 100%; object-fit: contain;">

## Overview

Lead scoring assigns numeric values to prospects based on their attributes and behaviors. Higher scores indicate greater sales readiness. General Bots provides both rule-based and AI-enhanced scoring approaches.

## SCORE LEAD

Calculate a lead score based on profile and behavior data using configurable rules.

### Syntax

```basic
score = SCORE LEAD lead_data
```

### Example

```basic
lead_data = NEW OBJECT
lead_data.email = "john@company.com"
lead_data.name = "John Smith"
lead_data.company = "Acme Corp"
lead_data.job_title = "VP of Engineering"
lead_data.industry = "Technology"
lead_data.company_size = "Enterprise"

score = SCORE LEAD lead_data

TALK "Score: " + score.score
TALK "Grade: " + score.grade
TALK "Status: " + score.status
TALK "Top recommendation: " + score.recommendations[0]
```

### Return Object

The `SCORE LEAD` keyword returns an object containing:

| Property | Type | Description |
|----------|------|-------------|
| `score` | Integer | Numeric score (0-100) |
| `grade` | String | Letter grade (A, B, C, D, F) |
| `status` | String | hot, warm, cold, or unqualified |
| `breakdown` | Object | Score components by category |
| `recommendations` | Array | Suggested next actions |

### Score Breakdown

```basic
score = SCORE LEAD lead_data

TALK "Demographic score: " + score.breakdown.demographic
TALK "Firmographic score: " + score.breakdown.firmographic
TALK "Behavioral score: " + score.breakdown.behavioral
TALK "Engagement score: " + score.breakdown.engagement
```

## AI SCORE LEAD

Use AI/LLM-enhanced scoring for more nuanced lead evaluation.

```basic
score = AI SCORE LEAD lead_data

TALK "AI Score: " + score.score
TALK "Confidence: " + score.breakdown.ai_confidence
TALK "Reasoning: " + score.breakdown.ai_reasoning
```

AI scoring considers factors that rule-based scoring might miss, such as company news, market conditions, and subtle signals in communication patterns.

### When to Use AI Scoring

AI scoring works best for complex B2B scenarios where context matters significantly. Rule-based scoring is faster and sufficient for high-volume B2C leads with clear qualification criteria.

```basic
' Use AI for enterprise leads, rules for SMB
IF lead_data.company_size = "Enterprise" THEN
    score = AI SCORE LEAD lead_data
ELSE
    score = SCORE LEAD lead_data
END IF
```

## GET LEAD SCORE

Retrieve an existing lead score from the database.

```basic
score = GET LEAD SCORE "lead-id"
TALK "Current score: " + score.score
TALK "Last updated: " + score.updated_at
```

## QUALIFY LEAD

Check if a lead meets the qualification threshold for sales handoff.

### Default Threshold (70)

```basic
result = QUALIFY LEAD "lead-id"
IF result.qualified THEN
    TALK "Lead is qualified: " + result.status
    ' Notify sales team
    SEND MAIL TO "sales@company.com" SUBJECT "New Qualified Lead" BODY result
ELSE
    TALK "Lead needs more nurturing. Score: " + result.score
END IF
```

### Custom Threshold

```basic
' Enterprise deals require higher qualification
result = QUALIFY LEAD "lead-id", 85

IF result.qualified THEN
    TALK "Enterprise lead qualified for sales"
END IF
```

### Qualification Result

| Property | Type | Description |
|----------|------|-------------|
| `qualified` | Boolean | Meets threshold |
| `score` | Integer | Current score |
| `threshold` | Integer | Applied threshold |
| `status` | String | Current lead status |
| `gap` | Integer | Points needed if not qualified |

## UPDATE LEAD SCORE

Manually adjust a lead's score based on specific actions or behaviors.

### Add Points

```basic
' Lead attended webinar
new_score = UPDATE LEAD SCORE "lead-id", 10, "Attended product webinar"
TALK "Score updated to: " + new_score.score
```

### Deduct Points

```basic
' Lead unsubscribed from newsletter
new_score = UPDATE LEAD SCORE "lead-id", -15, "Unsubscribed from email"
```

### Behavioral Scoring

```basic
ON "webinar:attended"
    UPDATE LEAD SCORE params.lead_id, 15, "Webinar attendance"
END ON

ON "pricing:viewed"
    UPDATE LEAD SCORE params.lead_id, 20, "Viewed pricing page"
END ON

ON "demo:requested"
    UPDATE LEAD SCORE params.lead_id, 30, "Requested demo"
END ON

ON "email:bounced"
    UPDATE LEAD SCORE params.lead_id, -25, "Email bounced"
END ON
```

## Complete Lead Nurturing Flow

```basic
' lead-nurturing.bas
PARAM email AS string
PARAM name AS string
PARAM company AS string
PARAM source AS string

DESCRIPTION "Process and score new leads"

' Build lead profile
WITH lead
    .email = email
    .name = name
    .company = company
    .source = source
    .created_at = NOW()
END WITH

' Initial scoring
score = SCORE LEAD lead

' Store lead
INSERT "leads", lead
SET BOT MEMORY "lead_" + email + "_score", score.score

' Route based on score
IF score.status = "hot" THEN
    ' Immediate sales notification
    SEND MAIL TO "sales@company.com" SUBJECT "Hot Lead: " + name BODY score
    SEND TEMPLATE "hot-lead-welcome", "email", email, #{name: name}
    
ELSEIF score.status = "warm" THEN
    ' Schedule nurture sequence
    SEND TEMPLATE "welcome", "email", email, #{name: name}
    SET SCHEDULE DATEADD(NOW(), 3, "day"), "nurture-day-3.bas"
    
ELSE
    ' Cold lead - educational content
    SEND TEMPLATE "educational", "email", email, #{name: name}
END IF

TALK "Lead " + name + " processed with score " + score.score + " (" + score.status + ")"
```

## Lead Scoring Configuration

Configure scoring weights in your bot's `config.csv`:

```csv
key,value
lead-score-job-title-weight,20
lead-score-company-size-weight,15
lead-score-industry-weight,10
lead-score-engagement-weight,25
lead-score-behavioral-weight,30
lead-score-qualification-threshold,70
```

### Title-Based Scoring

| Job Title Pattern | Points |
|-------------------|--------|
| C-Level (CEO, CTO, CFO) | 25 |
| VP / Vice President | 20 |
| Director | 15 |
| Manager | 10 |
| Individual Contributor | 5 |

### Company Size Scoring

| Company Size | Points |
|--------------|--------|
| Enterprise (1000+) | 20 |
| Mid-Market (100-999) | 15 |
| SMB (10-99) | 10 |
| Small (1-9) | 5 |

### Behavioral Actions

| Action | Typical Points |
|--------|---------------|
| Demo request | +30 |
| Pricing page view | +20 |
| Case study download | +15 |
| Webinar attendance | +15 |
| Blog subscription | +10 |
| Email open | +2 |
| Email click | +5 |
| Unsubscribe | -15 |
| Email bounce | -25 |

## Scheduled Score Decay

Implement score decay for inactive leads:

```basic
' score-decay.bas
SET SCHEDULE "every day at 2am"

' Find leads with no activity in 30 days
stale_leads = FIND "leads", "last_activity < DATEADD(NOW(), -30, 'day') AND score > 20"

FOR EACH lead IN stale_leads
    UPDATE LEAD SCORE lead.id, -5, "Inactivity decay"
NEXT lead

TALK "Processed " + LEN(stale_leads) + " stale leads"
```

## Integration with CRM

Push qualified leads to external CRM systems:

```basic
result = QUALIFY LEAD lead_id

IF result.qualified THEN
    ' Push to Salesforce
    crm_payload = NEW OBJECT
    crm_payload.email = lead.email
    crm_payload.name = lead.name
    crm_payload.score = result.score
    crm_payload.status = "Qualified"
    
    POST "https://api.salesforce.com/leads", crm_payload
    
    ' Mark as synced
    UPDATE "leads", "id = " + lead_id, #{crm_synced: true, synced_at: NOW()}
END IF
```

## Best Practices

**Start with simple rules.** Begin with basic demographic and firmographic scoring, then add behavioral triggers as you gather data.

**Align scoring with sales.** Work with your sales team to define what makes a "qualified" lead. Their input ensures scores reflect actual sales readiness.

**Review and adjust regularly.** Analyze conversion rates by score range monthly. Adjust weights if high-scoring leads aren't converting.

**Combine rule-based and AI scoring.** Use rule-based scoring for speed and consistency, AI scoring for complex enterprise deals requiring nuanced evaluation.

**Implement score decay.** Leads that go cold should have their scores decrease over time to keep the pipeline accurate.

**Track score history.** Store score changes with timestamps and reasons for audit trails and analysis.

```basic
' Log all score changes
ON "lead:score:changed"
    INSERT "score_history", #{
        lead_id: params.lead_id,
        old_score: params.old_score,
        new_score: params.new_score,
        reason: params.reason,
        changed_at: NOW()
    }
END ON
```

## See Also

- [SEND TEMPLATE](./keywords.md) - Nurture campaign emails
- [SET SCHEDULE](./keyword-set-schedule.md) - Automated scoring jobs
- [ON Keyword](./keyword-on.md) - Event-driven score updates
- [GET / POST](./keywords-http.md) - CRM integration