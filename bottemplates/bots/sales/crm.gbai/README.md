# Sales CRM Template (crm.gbai)

A comprehensive General Bots template for sales customer relationship management with lead tracking, opportunity management, and sales pipeline automation.

## Overview

The CRM template provides a full-featured sales CRM system with conversational AI capabilities. It enables sales teams to manage leads, track opportunities through the pipeline, generate quotes, send proposals, and forecast revenue—all through natural conversation or automated workflows.

## Features

- **Lead Management** - Capture, qualify, convert, and nurture leads
- **Opportunity Pipeline** - Track deals through customizable stages
- **Account Management** - Manage customer accounts and contacts
- **Activity Tracking** - Log calls, emails, meetings, and tasks
- **Quote Generation** - Create and send professional quotes
- **Proposal Automation** - Generate and deliver sales proposals
- **Sales Forecasting** - Pipeline analysis and revenue projections
- **Email Integration** - Receive and process emails automatically
- **Sentiment Analysis** - AI-powered customer sentiment tracking
- **Data Enrichment** - Automatic lead data enhancement

## Package Structure

```
crm.gbai/
├── README.md
├── crm.gbdialog/
│   ├── lead-management.bas          # Lead lifecycle management
│   ├── opportunity-management.bas   # Opportunity pipeline
│   ├── account-management.bas       # Account/company management
│   ├── activity-tracking.bas        # Activity logging
│   ├── case-management.bas          # Support case handling
│   ├── analyze-customer-sentiment.bas # AI sentiment analysis
│   ├── data-enrichment.bas          # Lead data enhancement
│   ├── send-proposal.bas            # Proposal generation
│   ├── create-lead-from-draft.bas   # Email to lead conversion
│   ├── crm-jobs.bas                 # Scheduled background jobs
│   ├── tables.bas                   # Database schema definitions
│   └── ... (additional scripts)
└── crm.gbot/
    └── config.csv                   # Bot configuration
```

## Scripts

| File | Description |
|------|-------------|
| `lead-management.bas` | Complete lead lifecycle: capture, qualify, convert, follow-up, nurture |
| `opportunity-management.bas` | Pipeline stages, quotes, products, forecasting |
| `account-management.bas` | Account and contact management |
| `activity-tracking.bas` | Log and track all sales activities |
| `case-management.bas` | Customer support case handling |
| `analyze-customer-sentiment.bas` | AI-powered sentiment analysis |
| `data-enrichment.bas` | Enrich leads with external data |
| `send-proposal.bas` | Generate and send proposals |
| `on-receive-email.bas` | Process incoming emails |
| `crm-jobs.bas` | Scheduled automation tasks |
| `tables.bas` | CRM database schema |

## Data Schema

### Leads Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | GUID | Unique identifier |
| `name` | String | Lead name |
| `email` | Email | Email address |
| `phone` | Phone | Phone number |
| `company` | String | Company name |
| `source` | String | Lead source |
| `status` | String | new, qualified, hot, warm, cold, converted |
| `score` | Integer | Lead qualification score (0-100) |
| `assigned_to` | String | Sales rep ID |
| `created_at` | DateTime | Creation timestamp |

### Opportunities Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | GUID | Unique identifier |
| `name` | String | Opportunity name |
| `account_id` | GUID | Related account |
| `contact_id` | GUID | Primary contact |
| `amount` | Decimal | Deal value |
| `stage` | String | Pipeline stage |
| `probability` | Integer | Win probability (0-100) |
| `close_date` | Date | Expected close date |
| `owner_id` | String | Sales rep ID |
| `lead_source` | String | Original lead source |

### Accounts Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | GUID | Unique identifier |
| `name` | String | Company name |
| `type` | String | prospect, customer, partner |
| `industry` | String | Industry vertical |
| `owner_id` | String | Account owner |
| `created_from_lead` | GUID | Original lead ID |

### Contacts Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | GUID | Unique identifier |
| `account_id` | GUID | Parent account |
| `name` | String | Full name |
| `email` | Email | Email address |
| `phone` | Phone | Phone number |
| `title` | String | Job title |
| `primary_contact` | Boolean | Primary contact flag |

### Activities Table

| Field | Type | Description |
|-------|------|-------------|
| `id` | GUID | Unique identifier |
| `type` | String | call, email, meeting, task |
| `subject` | String | Activity subject |
| `lead_id` | GUID | Related lead |
| `opportunity_id` | GUID | Related opportunity |
| `created_at` | DateTime | Activity timestamp |

## Pipeline Stages

| Stage | Probability | Description |
|-------|-------------|-------------|
| `qualification` | 10% | Initial qualification |
| `needs_analysis` | 20% | Understanding requirements |
| `value_proposition` | 50% | Presenting solution |
| `decision_makers` | 60% | Engaging decision makers |
| `proposal` | 75% | Proposal sent |
| `negotiation` | 90% | Terms negotiation |
| `closed_won` | 100% | Deal closed - won |
| `closed_lost` | 0% | Deal closed - lost |

## Lead Management

### Lead Capture

```basic
' Capture a new lead
WITH new_lead
    id = FORMAT(GUID())
    name = lead_data.name
    email = lead_data.email
    phone = lead_data.phone
    company = lead_data.company
    source = lead_data.source
    status = "new"
    score = 0
    created_at = NOW()
    assigned_to = user_id
END WITH

SAVE "leads.csv", new_lead
```

### Lead Qualification (BANT)

The qualification process scores leads based on:

- **Budget** - Revenue range (up to 30 points)
- **Authority** - Employee count/company size (up to 25 points)
- **Need** - Timeline urgency (up to 30 points)
- **Timeline** - Budget allocation (up to 25 points)

Scoring thresholds:
- **Hot Lead** (≥70): Immediate follow-up
- **Warm Lead** (50-69): 24-hour response
- **Cold Lead** (30-49): Nurture campaign
- **Unqualified** (<30): Archive

### Lead Conversion

When converting a lead, the system automatically creates:
1. Account record
2. Contact record
3. Opportunity record

```basic
' Convert qualified lead
IF lead.status = "hot" OR lead.status = "warm" THEN
    ' Create account
    SAVE "accounts.csv", account
    
    ' Create contact
    SAVE "contacts.csv", contact
    
    ' Create opportunity
    SAVE "opportunities.csv", opportunity
    
    ' Update lead status
    UPDATE "leads.csv" SET status = "converted"
END IF
```

### Lead Nurturing

Automated nurture campaigns based on lead age:

| Days | Content |
|------|---------|
| 3 | 5 Tips to Improve Your Business |
| 7 | Case Study: How We Helped Similar Companies |
| 14 | Free Consultation Offer |
| 30 | Special Limited Time Offer |

## Opportunity Management

### Creating Opportunities

```
User: Create a new opportunity for Acme Corp
Bot: What should we call this opportunity?
User: Enterprise License Deal
Bot: What is the estimated value of this deal?
User: $50,000
Bot: When do you expect to close this deal?
User: End of Q2
Bot: ✅ Opportunity created: Enterprise License Deal
     Account: Acme Corp
     Value: $50,000
     Close Date: June 30, 2024
```

### Updating Pipeline Stage

```
User: Update the Acme deal to proposal stage
Bot: Current stage: value_proposition
     
     Select new stage:
     1. Qualification (10%)
     2. Needs Analysis (20%)
     3. Value Proposition (50%)
     4. Decision Makers (60%)
     5. Proposal (75%)
     6. Negotiation (90%)
     7. Closed Won (100%)
     8. Closed Lost (0%)

User: 5
Bot: ✅ Stage updated to proposal (75%)
```

### Quote Generation

```basic
' Generate quote from opportunity
quote_content = "QUOTATION\n"
quote_content = quote_content + "Quote #: " + quote.quote_number + "\n"
quote_content = quote_content + "To: " + account.name + "\n\n"
quote_content = quote_content + "ITEMS:\n"

FOR EACH item IN products DO
    quote_content = quote_content + item.product_name + " x " + item.quantity + 
                    " @ $" + item.unit_price + " = $" + item.total + "\n"
END FOR

quote_content = quote_content + "\nSubtotal: $" + quote.subtotal + "\n"
quote_content = quote_content + "Tax (10%): $" + quote.tax_amount + "\n"
quote_content = quote_content + "TOTAL: $" + quote.total + "\n"
```

### Sales Forecasting

```
User: Show me the sales forecast
Bot: 📊 SALES FORECAST
     ==============
     Total Pipeline: $425,000
     Weighted Pipeline: $187,500
     
     Quarterly Forecast:
     Q1: $45,000
     Q2: $82,500
     Q3: $35,000
     Q4: $25,000
```

## Configuration

Configure in `crm.gbot/config.csv`:

| Parameter | Description | Example |
|-----------|-------------|---------|
| `Company Name` | Your company name | `Acme Sales` |
| `Currency` | Default currency | `USD` |
| `Tax Rate` | Default tax percentage | `10` |
| `Quote Validity Days` | Quote expiration | `30` |
| `Pipeline Stages` | Custom stage definitions | (JSON) |
| `Lead Sources` | Available lead sources | `web,referral,event` |
| `Admin Email` | Notifications email | `sales@company.com` |

## Email Integration

### Receiving Emails

```basic
' on-receive-email.bas
email_from = GET "email.from"
email_subject = GET "email.subject"
email_body = GET "email.body"

' Check if from existing contact
contact = FIND "contacts.csv", "email = '" + email_from + "'"

IF contact THEN
    ' Log activity against contact
    WITH activity
        type = "email"
        subject = email_subject
        contact_id = contact.id
    END WITH
    SAVE "activities.csv", activity
ELSE
    ' Create new lead from email
    CALL "create-lead-from-draft.bas"
END IF
```

### Sending Proposals

```basic
' send-proposal.bas
proposal = GENERATE FROM TEMPLATE "proposal_template.docx" WITH {
    "company": account.name,
    "contact": contact.name,
    "products": opportunity_products,
    "total": quote.total,
    "valid_until": quote.valid_until
}

SEND EMAIL contact.email, "Proposal: " + opportunity.name, 
    "Please find attached our proposal.", ATTACHMENT proposal
```

## AI Features

### Customer Sentiment Analysis

```basic
' analyze-customer-sentiment.bas
SET CONTEXT "Analyze customer communication for sentiment and buying signals."

communications = FIND "activities.csv", "contact_id = '" + contact_id + "'"

analysis = LLM "Analyze these customer communications and provide:
               1. Overall sentiment (positive, neutral, negative)
               2. Buying signals detected
               3. Concerns or objections
               4. Recommended next action
               
               Communications: " + JSON(communications)

TALK analysis
```

### Data Enrichment

```basic
' data-enrichment.bas
' Enrich lead with external data
company_info = GET "https://api.enrichment.com/company/" + lead.company

IF company_info THEN
    lead.industry = company_info.industry
    lead.employee_count = company_info.employees
    lead.revenue_range = company_info.revenue
    lead.linkedin_url = company_info.linkedin
    
    UPDATE "leads.csv", lead
END IF
```

## Scheduled Jobs

| Job | Schedule | Description |
|-----|----------|-------------|
| Lead nurturing | Daily | Send nurture emails to cold/warm leads |
| Follow-up reminders | Hourly | Alert reps of overdue follow-ups |
| Pipeline cleanup | Weekly | Archive stale opportunities |
| Forecast update | Daily | Recalculate sales forecast |
| Activity sync | Real-time | Sync emails and calendar events |

## Usage Examples

### Full Lead Lifecycle

```
User: I just got a lead from the website
Bot: Great! I'll help you capture it. What's their name?
User: John Smith from TechCorp
Bot: What's their email?
User: john@techcorp.com
Bot: ✅ Lead captured: John Smith (TechCorp)
     ID: LEAD-20240115-4821
     
     Would you like to qualify this lead now?

User: Yes
Bot: I'll ask a few questions to qualify them.
     What is TechCorp's annual revenue range?
     1. Under $1M
     2. $1M - $10M
     3. $10M - $50M
     4. Over $50M

User: 3
Bot: How many employees?
User: About 200
Bot: What's their timeline?
User: This quarter
Bot: Do they have budget allocated?
User: Yes

Bot: 🔥 Hot Lead! Score: 85/100
     This lead is a great fit. I've:
     - Notified sales@company.com
     - Created a follow-up task
     
     Would you like to convert to an opportunity?

User: Yes
Bot: ✅ Lead converted!
     - Account created: TechCorp
     - Contact created: John Smith
     - Opportunity created: Opportunity for TechCorp
     
     What stage should we start at?
```

## Best Practices

1. **Qualify early** - Use BANT scoring to prioritize leads
2. **Track everything** - Log all customer interactions
3. **Follow up promptly** - Hot leads within hours, warm within 24h
4. **Use automation** - Let nurture campaigns work cold leads
5. **Clean pipeline** - Archive stale opportunities regularly
6. **Forecast accurately** - Keep close dates and probabilities updated
7. **Segment leads** - Use tags and sources for better targeting

## Troubleshooting

| Issue | Cause | Solution |
|-------|-------|----------|
| Lead not found | Search criteria too strict | Use partial match |
| Stage not updating | Missing opportunity ID | Set opportunity in session |
| Quote not generating | Missing products | Add products to opportunity first |
| Email not sending | Missing contact email | Verify contact record |
| Forecast incorrect | Stale data | Update opportunity amounts |

## Related Templates

- `contacts.gbai` - Contact directory management
- `marketing.gbai` - Marketing automation and campaigns
- `analytics.gbai` - Sales analytics and reporting
- `reminder.gbai` - Follow-up reminders

## Use Cases

- **Inside Sales** - Lead qualification and opportunity management
- **Field Sales** - Account management and activity tracking
- **Sales Management** - Pipeline visibility and forecasting
- **Business Development** - Lead generation and nurturing
- **Customer Success** - Account health and expansion opportunities

## Integration Points

- **Email** - Inbound/outbound email tracking
- **Calendar** - Meeting scheduling
- **ERP** - Order and billing sync
- **Marketing Automation** - Lead handoff
- **Support Ticketing** - Case management

## License

AGPL-3.0 - Part of General Bots Open Source Platform.

---

**Pragmatismo** - General Bots