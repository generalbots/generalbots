# CRM / Contacts API 🟡 BETA

> **Contact, account, lead, opportunity, and deal management with full pipeline visibility**

---

## Base URL

```
/api/crm
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header.

---

## Contacts

### List Contacts

**`GET /api/crm/contacts`**

Returns all contacts with optional filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `search` | string | No | Search by name or email |
| `account_id` | string | No | Filter by account |
| `tag` | string | No | Filter by tag |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "contacts": [
    {
      "id": "contact_001",
      "name": "João Silva",
      "email": "joao@example.com",
      "phone": "+5511999999999",
      "company": "Acme Corp",
      "account_id": "acc_001",
      "role": "CTO",
      "tags": ["vip", "enterprise"],
      "created_at": "2026-01-15T10:00:00Z",
      "last_activity_at": "2026-06-04T10:00:00Z"
    }
  ],
  "total": 150,
  "page": 1
}
```

---

### Create Contact

**`POST /api/crm/contacts`**

Creates a new contact record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Full name |
| `email` | string | No | Email address |
| `phone` | string | No | Phone number |
| `company` | string | No | Company name |
| `account_id` | string | No | Linked account ID |
| `role` | string | No | Job title/role |
| `tags` | string[] | No | Tags |
| `custom_fields` | object | No | Custom field values |

**Response:**
```json
{
  "id": "contact_002",
  "name": "Ana Costa",
  "email": "ana@startup.com",
  "phone": "+5511888888888",
  "company": "Startup Inc",
  "account_id": null,
  "role": "CEO",
  "tags": [],
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Contact

**`GET /api/crm/contacts/:id`**

Returns full details of a contact.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Contact identifier |

**Response:**
```json
{
  "id": "contact_001",
  "name": "João Silva",
  "email": "joao@example.com",
  "phone": "+5511999999999",
  "company": "Acme Corp",
  "account_id": "acc_001",
  "role": "CTO",
  "tags": ["vip", "enterprise"],
  "custom_fields": {
    "linkedin": "https://linkedin.com/in/joaosilva",
    "budget_range": "50k-100k"
  },
  "activities": [
    {
      "id": "act_001",
      "type": "email",
      "subject": "Proposta comercial",
      "date": "2026-06-03T14:00:00Z"
    }
  ],
  "notes": "Decisor principal para ferramentas de TI",
  "created_at": "2026-01-15T10:00:00Z",
  "updated_at": "2026-06-04T10:00:00Z"
}
```

---

### Update Contact

**`PUT /api/crm/contacts/:id`**

Updates an existing contact.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Contact identifier |
| `name` | string | No | Full name |
| `email` | string | No | Email |
| `phone` | string | No | Phone |
| `company` | string | No | Company |
| `account_id` | string | No | Account ID |
| `role` | string | No | Job title |
| `tags` | string[] | No | Tags |
| `custom_fields` | object | No | Custom fields |

**Response:**
```json
{
  "id": "contact_001",
  "name": "João Silva",
  "updated_at": "2026-06-04T10:15:00Z"
}
```

---

### Delete Contact

**`DELETE /api/crm/contacts/:id`**

Deletes a contact record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Contact identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "contact_002"
}
```

---

## Accounts

### List Accounts

**`GET /api/crm/accounts`**

Returns all accounts.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `search` | string | No | Search by company name |
| `industry` | string | No | Filter by industry |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "accounts": [
    {
      "id": "acc_001",
      "name": "Acme Corp",
      "industry": "Technology",
      "size": "enterprise",
      "website": "https://acme.com",
      "revenue": 5000000,
      "contacts_count": 12,
      "active_deals": 3,
      "created_at": "2026-01-10T10:00:00Z"
    }
  ],
  "total": 45
}
```

---

### Create Account

**`POST /api/crm/accounts`**

Creates a new account.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Company name |
| `industry` | string | No | Industry |
| `size` | string | No | `startup`, `smb`, `mid-market`, `enterprise` |
| `website` | string | No | Company website |
| `revenue` | number | No | Annual revenue |
| `address` | object | No | Address details |
| `custom_fields` | object | No | Custom fields |

**Response:**
```json
{
  "id": "acc_002",
  "name": "Startup Inc",
  "industry": "SaaS",
  "size": "startup",
  "website": "https://startup.com",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Account

**`GET /api/crm/accounts/:id`**

Returns account details including linked contacts and deals.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Account identifier |

**Response:**
```json
{
  "id": "acc_001",
  "name": "Acme Corp",
  "industry": "Technology",
  "size": "enterprise",
  "website": "https://acme.com",
  "revenue": 5000000,
  "contacts": [
    { "id": "contact_001", "name": "João Silva", "role": "CTO" },
    { "id": "contact_003", "name": "Maria Lima", "role": "VP Sales" }
  ],
  "deals": [
    { "id": "deal_001", "name": "Enterprise License", "value": 120000, "stage": "proposal" }
  ],
  "total_deal_value": 120000,
  "created_at": "2026-01-10T10:00:00Z"
}
```

---

### Delete Account

**`DELETE /api/crm/accounts/:id`**

Deletes an account. Contacts are unlinked, not deleted.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Account identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "acc_002",
  "unlinked_contacts": 3
}
```

---

## Leads

### List Leads

**`GET /api/crm/leads`**

Returns all leads with optional filtering.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `stage` | string | No | Filter by pipeline stage |
| `owner_id` | string | No | Filter by lead owner |
| `source` | string | No | Filter by lead source |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "leads": [
    {
      "id": "lead_001",
      "name": "Pedro Almeida",
      "email": "pedro@enterprise.com",
      "company": "Enterprise Ltd",
      "stage": "qualified",
      "source": "website",
      "score": 85,
      "owner": {
        "id": "agent_001",
        "name": "Maria Santos"
      },
      "estimated_value": 50000,
      "created_at": "2026-06-01T10:00:00Z",
      "last_activity_at": "2026-06-03T14:00:00Z"
    }
  ],
  "total": 32
}
```

---

### Create Lead

**`POST /api/crm/leads`**

Creates a new lead.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Lead name |
| `email` | string | No | Email |
| `phone` | string | No | Phone |
| `company` | string | No | Company |
| `source` | string | No | Lead source: `website`, `referral`, `campaign`, `cold_call`, `social` |
| `owner_id` | string | No | Assigned owner |
| `estimated_value` | number | No | Estimated deal value |
| `notes` | string | No | Notes about the lead |

**Response:**
```json
{
  "id": "lead_002",
  "name": "Fernanda Rocha",
  "company": "Tech Solutions",
  "stage": "new",
  "source": "campaign",
  "score": 45,
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Lead

**`GET /api/crm/leads/:id`**

Returns lead details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Lead identifier |

**Response:**
```json
{
  "id": "lead_001",
  "name": "Pedro Almeida",
  "email": "pedro@enterprise.com",
  "company": "Enterprise Ltd",
  "stage": "qualified",
  "source": "website",
  "score": 85,
  "owner": { "id": "agent_001", "name": "Maria Santos" },
  "estimated_value": 50000,
  "activities": [
    { "type": "form_submission", "date": "2026-06-01T10:00:00Z", "details": "Demo request" },
    { "type": "call", "date": "2026-06-02T14:00:00Z", "details": "Qualification call completed" }
  ],
  "score_history": [45, 60, 85],
  "created_at": "2026-06-01T10:00:00Z"
}
```

---

### Update Lead

**`PUT /api/crm/leads/:id`**

Updates lead information.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Lead identifier |
| `name` | string | No | Name |
| `email` | string | No | Email |
| `company` | string | No | Company |
| `owner_id` | string | No | Owner |
| `estimated_value` | number | No | Value |
| `notes` | string | No | Notes |

**Response:**
```json
{
  "id": "lead_001",
  "name": "Pedro Almeida",
  "updated_at": "2026-06-04T10:15:00Z"
}
```

---

### Delete Lead

**`DELETE /api/crm/leads/:id`**

Deletes a lead.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Lead identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "lead_002"
}
```

---

### Update Lead Stage

**`PUT /api/crm/leads/:id/stage`**

Moves a lead to a different pipeline stage.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Lead identifier |
| `stage` | string | Yes | Target stage: `new`, `contacted`, `qualified`, `proposal`, `negotiation`, `won`, `lost` |

**Response:**
```json
{
  "id": "lead_001",
  "previous_stage": "qualified",
  "current_stage": "proposal",
  "moved_at": "2026-06-04T10:20:00Z"
}
```

---

### Convert Lead

**`POST /api/crm/leads/:id/convert`**

Converts a lead into a contact, account, and/or opportunity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Lead identifier |
| `create_account` | boolean | No | Create account from lead (default: true) |
| `create_opportunity` | boolean | No | Create opportunity (default: true) |
| `account_name` | string | No | Custom account name |
| `opportunity_name` | string | No | Custom opportunity name |
| `opportunity_value` | number | No | Initial opportunity value |

**Response:**
```json
{
  "lead_id": "lead_001",
  "converted": true,
  "contact_id": "contact_010",
  "account_id": "acc_010",
  "opportunity_id": "opp_010",
  "converted_at": "2026-06-04T10:30:00Z"
}
```

---

## Opportunities

### List Opportunities

**`GET /api/crm/opportunities`**

Returns all opportunities.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `stage` | string | No | Filter by stage |
| `owner_id` | string | No | Filter by owner |
| `account_id` | string | No | Filter by account |
| `min_value` | number | No | Minimum value filter |
| `max_value` | number | No | Maximum value filter |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "opportunities": [
    {
      "id": "opp_001",
      "name": "Enterprise License Acme",
      "value": 120000,
      "stage": "proposal",
      "probability": 70,
      "account_id": "acc_001",
      "account_name": "Acme Corp",
      "owner": { "id": "agent_001", "name": "Maria Santos" },
      "expected_close_date": "2026-07-15T00:00:00Z",
      "created_at": "2026-06-01T10:00:00Z"
    }
  ],
  "total": 18,
  "total_value": 850000,
  "weighted_value": 595000
}
```

---

### Create Opportunity

**`POST /api/crm/opportunities`**

Creates a new opportunity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `name` | string | Yes | Opportunity name |
| `value` | number | Yes | Deal value |
| `stage` | string | No | Initial stage (default: `qualification`) |
| `account_id` | string | No | Linked account |
| `owner_id` | string | No | Owner |
| `probability` | integer | No | Win probability 0-100 |
| `expected_close_date` | string | ISO 8601 | Expected close date |
| `notes` | string | No | Notes |

**Response:**
```json
{
  "id": "opp_002",
  "name": "SaaS Subscription Startup",
  "value": 24000,
  "stage": "qualification",
  "probability": 30,
  "expected_close_date": "2026-08-01T00:00:00Z",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Opportunity

**`GET /api/crm/opportunities/:id`**

Returns opportunity details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Opportunity identifier |

**Response:**
```json
{
  "id": "opp_001",
  "name": "Enterprise License Acme",
  "value": 120000,
  "stage": "proposal",
  "probability": 70,
  "account": { "id": "acc_001", "name": "Acme Corp" },
  "owner": { "id": "agent_001", "name": "Maria Santos" },
  "expected_close_date": "2026-07-15T00:00:00Z",
  "contacts": [
    { "id": "contact_001", "name": "João Silva", "role": "Decision Maker" }
  ],
  "activities": [
    { "type": "meeting", "date": "2026-06-03T14:00:00Z", "summary": "Demo apresentada" }
  ],
  "stage_history": [
    { "stage": "qualification", "entered_at": "2026-06-01T10:00:00Z" },
    { "stage": "proposal", "entered_at": "2026-06-03T16:00:00Z" }
  ],
  "created_at": "2026-06-01T10:00:00Z"
}
```

---

### Update Opportunity

**`PUT /api/crm/opportunities/:id`**

Updates opportunity details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Opportunity identifier |
| `name` | string | No | Name |
| `value` | number | No | Value |
| `stage` | string | No | Stage |
| `probability` | integer | No | Probability |
| `expected_close_date` | string | ISO 8601 | Close date |
| `owner_id` | string | No | Owner |

**Response:**
```json
{
  "id": "opp_001",
  "value": 135000,
  "probability": 80,
  "updated_at": "2026-06-04T10:15:00Z"
}
```

---

### Delete Opportunity

**`DELETE /api/crm/opportunities/:id`**

Deletes an opportunity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Opportunity identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "opp_002"
}
```

---

### Close Opportunity

**`POST /api/crm/opportunities/:id/close`**

Marks an opportunity as won or lost.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Opportunity identifier |
| `outcome` | string | Yes | `won` or `lost` |
| `actual_value` | number | No | Final deal value |
| `reason` | string | No | Reason (required if lost) |
| `closed_at` | string | ISO 8601 | Close date (default: now) |

**Response:**
```json
{
  "id": "opp_001",
  "stage": "won",
  "actual_value": 135000,
  "closed_at": "2026-06-04T10:30:00Z",
  "won": true,
  "cycle_days": 45
}
```

---

## Deals

### List Deals

**`GET /api/crm/deals`**

Returns all deals (closed-won opportunities with financial details).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | `pending`, `completed`, `cancelled` |
| `from_date` | string | ISO 8601 | Filter from date |
| `to_date` | string | ISO 8601 | Filter to date |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Results per page (default: 20) |

**Response:**
```json
{
  "deals": [
    {
      "id": "deal_001",
      "opportunity_id": "opp_001",
      "name": "Enterprise License Acme",
      "value": 135000,
      "status": "completed",
      "account_name": "Acme Corp",
      "closed_at": "2026-06-04T10:30:00Z",
      "payment_terms": "Net 30",
      "invoice_id": "INV-2026-001"
    }
  ],
  "total": 28,
  "total_value": 1250000
}
```

---

### Create Deal

**`POST /api/crm/deals`**

Creates a new deal record.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `opportunity_id` | string | Yes | Linked opportunity |
| `value` | number | Yes | Deal value |
| `payment_terms` | string | No | Payment terms |
| `notes` | string | No | Deal notes |

**Response:**
```json
{
  "id": "deal_002",
  "opportunity_id": "opp_002",
  "value": 24000,
  "status": "pending",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Deal

**`GET /api/crm/deals/:id`**

Returns deal details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Deal identifier |

**Response:**
```json
{
  "id": "deal_001",
  "opportunity": {
    "id": "opp_001",
    "name": "Enterprise License Acme",
    "account_name": "Acme Corp"
  },
  "value": 135000,
  "status": "completed",
  "payment_terms": "Net 30",
  "invoice_id": "INV-2026-001",
  "closed_at": "2026-06-04T10:30:00Z"
}
```

---

### Update Deal

**`PUT /api/crm/deals/:id`**

Updates deal details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Deal identifier |
| `value` | number | No | Deal value |
| `status` | string | No | Status |
| `payment_terms` | string | No | Payment terms |

**Response:**
```json
{
  "id": "deal_001",
  "value": 140000,
  "updated_at": "2026-06-04T10:15:00Z"
}
```

---

### Delete Deal

**`DELETE /api/crm/deals/:id`**

Deletes a deal.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Deal identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "deal_002"
}
```

---

## Activities

### List Activities

**`GET /api/crm/activities`**

Returns all activities across CRM records.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | No | Filter: `call`, `email`, `meeting`, `task`, `note` |
| `contact_id` | string | No | Filter by contact |
| `account_id` | string | No | Filter by account |
| `owner_id` | string | No | Filter by owner |
| `from_date` | string | ISO 8601 | Start date |
| `to_date` | string | ISO 8601 | End date |
| `page` | integer | No | Page number (default: 1) |

**Response:**
```json
{
  "activities": [
    {
      "id": "act_001",
      "type": "call",
      "subject": "Qualification call",
      "contact_id": "contact_001",
      "contact_name": "João Silva",
      "account_name": "Acme Corp",
      "owner_id": "agent_001",
      "duration_seconds": 900,
      "outcome": "interested",
      "scheduled_at": "2026-06-03T14:00:00Z",
      "completed_at": "2026-06-03T14:15:00Z"
    }
  ],
  "total": 89
}
```

---

### Create Activity

**`POST /api/crm/activities`**

Creates a new activity.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `type` | string | Yes | `call`, `email`, `meeting`, `task`, `note` |
| `subject` | string | Yes | Activity subject |
| `description` | string | No | Description |
| `contact_id` | string | No | Related contact |
| `account_id` | string | No | Related account |
| `opportunity_id` | string | No | Related opportunity |
| `scheduled_at` | string | ISO 8601 | Scheduled date/time |
| `due_at` | string | ISO 8601 | Due date (for tasks) |
| `duration_seconds` | integer | No | Duration |
| `outcome` | string | No | Outcome |

**Response:**
```json
{
  "id": "act_002",
  "type": "meeting",
  "subject": "Demo técnica",
  "contact_id": "contact_001",
  "scheduled_at": "2026-06-05T10:00:00Z",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

## Pipeline

### Get Pipeline Stages

**`GET /api/crm/pipeline/stages`**

Returns the configured pipeline stages.

**Response:**
```json
{
  "stages": [
    { "id": "stage_001", "name": "qualification", "order": 1, "probability": 10, "color": "#6B7280" },
    { "id": "stage_002", "name": "needs_analysis", "order": 2, "probability": 25, "color": "#3B82F6" },
    { "id": "stage_003", "name": "proposal", "order": 3, "probability": 50, "color": "#8B5CF6" },
    { "id": "stage_004", "name": "negotiation", "order": 4, "probability": 75, "color": "#F59E0B" },
    { "id": "stage_005", "name": "closed_won", "order": 5, "probability": 100, "color": "#10B981" },
    { "id": "stage_006", "name": "closed_lost", "order": 6, "probability": 0, "color": "#EF4444" }
  ]
}
```

---

## Statistics

### Get CRM Statistics

**`GET /api/crm/stats`**

Returns aggregated CRM statistics.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | `today`, `week`, `month`, `quarter`, `year` (default: `month`) |
| `owner_id` | string | No | Filter by owner |

**Response:**
```json
{
  "period": "month",
  "contacts": {
    "total": 150,
    "new_this_period": 23,
    "by_source": {
      "website": 8,
      "referral": 10,
      "campaign": 5
    }
  },
  "leads": {
    "total": 32,
    "new_this_period": 12,
    "converted": 8,
    "conversion_rate": 0.25,
    "average_score": 65
  },
  "opportunities": {
    "total": 18,
    "total_value": 850000,
    "weighted_value": 595000,
    "won": 5,
    "won_value": 320000,
    "lost": 3,
    "lost_value": 95000,
    "win_rate": 0.625,
    "average_cycle_days": 42
  },
  "pipeline": {
    "by_stage": [
      { "stage": "qualification", "count": 5, "value": 175000 },
      { "stage": "proposal", "count": 6, "value": 420000 },
      { "stage": "negotiation", "count": 4, "value": 250000 }
    ]
  }
}
```

---

## External Sync

### List Sync Accounts

**`GET /api/crm/sync/accounts`**

Returns accounts configured for external system synchronization.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | No | Filter by provider: `salesforce`, `hubspot`, `pipedrive` |

**Response:**
```json
{
  "sync_accounts": [
    {
      "id": "sync_001",
      "provider": "salesforce",
      "name": "Salesforce Production",
      "status": "active",
      "last_sync_at": "2026-06-04T06:00:00Z",
      "records_synced": 1250,
      "error_count": 0
    }
  ]
}
```

---

### Create Sync Account

**`POST /api/crm/sync/accounts`**

Creates a new external system sync configuration.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `provider` | string | Yes | Provider name |
| `name` | string | Yes | Configuration name |
| `credentials` | object | Yes | API credentials (encrypted at rest) |
| `sync_direction` | string | No | `push`, `pull`, `bidirectional` (default: `bidirectional`) |
| `field_mappings` | object | No | Custom field mappings |

**Response:**
```json
{
  "id": "sync_002",
  "provider": "hubspot",
  "name": "Hubspot CRM",
  "status": "active",
  "sync_direction": "bidirectional",
  "created_at": "2026-06-04T10:00:00Z"
}
```

---

### Get Sync Account

**`GET /api/crm/sync/accounts/:id`**

Returns sync account details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Sync account identifier |

**Response:**
```json
{
  "id": "sync_001",
  "provider": "salesforce",
  "name": "Salesforce Production",
  "status": "active",
  "sync_direction": "bidirectional",
  "last_sync_at": "2026-06-04T06:00:00Z",
  "next_sync_at": "2026-06-04T12:00:00Z",
  "records_synced": 1250,
  "error_count": 0,
  "field_mappings": {
    "email": "Email",
    "phone": "Phone",
    "company": "Account.Name"
  }
}
```

---

### Delete Sync Account

**`DELETE /api/crm/sync/accounts/:id`**

Deletes a sync account and stops synchronization.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Sync account identifier |

**Response:**
```json
{
  "deleted": true,
  "id": "sync_002",
  "pending_syncs_cancelled": 3
}
```

---

### Trigger Manual Sync

**`POST /api/crm/sync/accounts/:id/sync`**

Triggers an immediate sync for the specified account.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | path | Yes | Sync account identifier |
| `direction` | string | No | Override sync direction for this run |
| `force` | boolean | No | Force full sync (default: incremental) |

**Response:**
```json
{
  "sync_id": "sync_run_001",
  "sync_account_id": "sync_001",
  "status": "started",
  "direction": "bidirectional",
  "started_at": "2026-06-04T10:00:00Z",
  "estimated_duration_seconds": 120
}
```

---

## See Also

- [Contacts API](users-api.md) — user management and authentication
- [Sheets API](sheets-api.md) — spreadsheet operations for bulk data
- [Tasks API](tasks-api.md) — task management linked to CRM activities
- [Analytics API](analytics-api.md) — reporting and dashboards
