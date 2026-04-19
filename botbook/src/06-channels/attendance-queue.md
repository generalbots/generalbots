# Attendance Queue Module

Human-attendant queue management for hybrid bot/human support workflows, plus CRM automations for follow-ups, collections, scheduling, and sales.

## Overview

The attendance queue module manages handoffs from bot to human agents, tracking conversation queues, attendant availability, and real-time assignment. It also provides automated CRM workflows that run without human intervention.

---

## Configuration

Create `attendant.csv` in your bot's `.gbai` folder:

```csv
id,name,channel,preferences,department
att-001,John Smith,whatsapp,sales,commercial
att-002,Jane Doe,web,support,customer-service
att-003,Bob Wilson,all,technical,engineering
att-004,Maria Santos,whatsapp,collections,finance
```

---

## Queue Status

| Status | Description |
|--------|-------------|
| `waiting` | User waiting for attendant |
| `assigned` | Attendant assigned, not yet active |
| `active` | Conversation in progress |
| `resolved` | Conversation completed |
| `abandoned` | User left before assignment |

## Attendant Status

| Status | Description |
|--------|-------------|
| `online` | Available for new conversations |
| `busy` | Currently handling conversations |
| `away` | Temporarily unavailable |
| `offline` | Not working |

---

## CRM Automations

The attendant module includes built-in CRM automations that handle common business workflows automatically.

### Follow-Up Automation

Automated follow-up sequences for leads and customers.

```basic
' follow-up.bas
' Automated follow-up workflow

SET SCHEDULE "follow-ups", "0 9 * * 1-5"

' Find leads needing follow-up
leads_1_day = FIND "leads", "status='new' AND DATEDIFF(NOW(), last_contact) = 1"
leads_3_day = FIND "leads", "status='contacted' AND DATEDIFF(NOW(), last_contact) = 3"
leads_7_day = FIND "leads", "status='contacted' AND DATEDIFF(NOW(), last_contact) = 7"

' 1-day follow-up: Thank you message
FOR EACH lead IN leads_1_day
    SEND TEMPLATE lead.phone, "follow_up_thanks", lead.name, lead.interest
    UPDATE "leads", "id=" + lead.id, "contacted", NOW()
    INSERT "activities", lead.id, "follow_up", "1-day thank you sent", NOW()
NEXT lead

' 3-day follow-up: Value proposition
FOR EACH lead IN leads_3_day
    SEND TEMPLATE lead.phone, "follow_up_value", lead.name, lead.interest
    UPDATE "leads", "id=" + lead.id, "nurturing", NOW()
    INSERT "activities", lead.id, "follow_up", "3-day value prop sent", NOW()
NEXT lead

' 7-day follow-up: Special offer
FOR EACH lead IN leads_7_day
    SEND TEMPLATE lead.phone, "follow_up_offer", lead.name, "10%"
    UPDATE "leads", "id=" + lead.id, "offer_sent", NOW()
    INSERT "activities", lead.id, "follow_up", "7-day offer sent", NOW()
    
    ' Alert sales team for hot leads
    IF lead.score >= 70 THEN
        attendant = FIND "attendants", "department='commercial' AND status='online'"
        IF attendant THEN
            SEND MAIL attendant.email, "Hot Lead Follow-up: " + lead.name, "Lead " + lead.name + " received 7-day offer. Score: " + lead.score
        END IF
    END IF
NEXT lead

PRINT "Follow-ups completed: " + UBOUND(leads_1_day) + " 1-day, " + UBOUND(leads_3_day) + " 3-day, " + UBOUND(leads_7_day) + " 7-day"
```

### Collections Automation (Cobranças)

Automated payment reminders and collection workflow.

```basic
' collections.bas
' Automated payment collection workflow

SET SCHEDULE "collections", "0 8 * * 1-5"

' Find overdue invoices by age
due_today = FIND "invoices", "status='pending' AND due_date = CURDATE()"
overdue_3 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 3"
overdue_7 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 7"
overdue_15 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 15"
overdue_30 = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) >= 30"

' Due today: Friendly reminder via WhatsApp
FOR EACH invoice IN due_today
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND TEMPLATE customer.phone, "payment_due_today", customer.name, invoice.id, invoice.amount
    INSERT "collection_log", invoice.id, "reminder_due_today", NOW()
NEXT invoice

' 3 days overdue: First collection notice
FOR EACH invoice IN overdue_3
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND TEMPLATE customer.phone, "payment_overdue_3", customer.name, invoice.id, invoice.amount
    SEND MAIL customer.email, "Pagamento Pendente - Fatura #" + invoice.id, "Sua fatura está vencida há 3 dias. Valor: R$ " + invoice.amount
    UPDATE "invoices", "id=" + invoice.id, "first_notice_sent", NOW()
    INSERT "collection_log", invoice.id, "first_notice", NOW()
NEXT invoice

' 7 days overdue: Second notice with urgency
FOR EACH invoice IN overdue_7
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND TEMPLATE customer.phone, "payment_overdue_7", customer.name, invoice.id, invoice.amount
    UPDATE "invoices", "id=" + invoice.id, "second_notice_sent", NOW()
    INSERT "collection_log", invoice.id, "second_notice", NOW()
    
    ' Notify collections team
    SEND MAIL "cobranca@empresa.com", "Cobrança 7 dias: " + customer.name, "Cliente: " + customer.name + "\nFatura: " + invoice.id + "\nValor: R$ " + invoice.amount
NEXT invoice

' 15 days overdue: Final notice before action
FOR EACH invoice IN overdue_15
    customer = FIND "customers", "id=" + invoice.customer_id
    late_fee = invoice.amount * 0.02
    interest = invoice.amount * 0.01 * 15
    total_due = invoice.amount + late_fee + interest
    
    SEND TEMPLATE customer.phone, "payment_final_notice", customer.name, invoice.id, total_due
    UPDATE "invoices", "id=" + invoice.id, late_fee, interest, total_due, "final_notice_sent", NOW()
    INSERT "collection_log", invoice.id, "final_notice", NOW()
    
    ' Assign to human attendant for follow-up call
    attendant = FIND "attendants", "department='finance' AND status='online'"
    IF attendant THEN
        INSERT "queue", invoice.customer_id, attendant.id, "collection_call", "high", NOW()
    END IF
NEXT invoice

' 30+ days overdue: Escalate to collections
FOR EACH invoice IN overdue_30
    IF invoice.status <> "collections" THEN
        customer = FIND "customers", "id=" + invoice.customer_id
        UPDATE "invoices", "id=" + invoice.id, "collections", NOW()
        UPDATE "customers", "id=" + customer.id, "suspended"
        
        SEND MAIL "juridico@empresa.com", "Inadimplência 30+ dias: " + customer.name, "Cliente enviado para cobrança jurídica.\n\nCliente: " + customer.name + "\nFatura: " + invoice.id + "\nValor total: R$ " + invoice.total_due
        INSERT "collection_log", invoice.id, "sent_to_collections", NOW()
    END IF
NEXT invoice

PRINT "Collections processed: " + UBOUND(due_today) + " due today, " + UBOUND(overdue_30) + " sent to collections"
```

### Scheduling Automation (Agendamentos)

Automated appointment scheduling and reminders.

```basic
' scheduling.bas
' Automated appointment scheduling and reminders

SET SCHEDULE "appointment-reminders", "0 7 * * *"

' Find appointments for today and tomorrow
today_appointments = FIND "appointments", "DATE(scheduled_at) = CURDATE() AND status='confirmed'"
tomorrow_appointments = FIND "appointments", "DATE(scheduled_at) = DATE_ADD(CURDATE(), INTERVAL 1 DAY) AND status='confirmed'"

' Send day-before reminders
FOR EACH appt IN tomorrow_appointments
    customer = FIND "customers", "id=" + appt.customer_id
    staff = FIND "staff", "id=" + appt.staff_id
    
    appt_time = FORMAT(appt.scheduled_at, "HH:mm")
    appt_date = FORMAT(appt.scheduled_at, "DD/MM/YYYY")
    
    SEND TEMPLATE customer.phone, "appointment_reminder_24h", customer.name, appt.service, appt_date, appt_time, staff.name
    UPDATE "appointments", "id=" + appt.id, "reminder_24h_sent", NOW()
NEXT appt

' Send same-day reminders (2 hours before)
FOR EACH appt IN today_appointments
    IF DATEDIFF_HOURS(appt.scheduled_at, NOW()) <= 2 AND appt.reminder_2h_sent IS NULL THEN
        customer = FIND "customers", "id=" + appt.customer_id
        staff = FIND "staff", "id=" + appt.staff_id
        
        appt_time = FORMAT(appt.scheduled_at, "HH:mm")
        
        SEND TEMPLATE customer.phone, "appointment_reminder_2h", customer.name, appt.service, appt_time
        UPDATE "appointments", "id=" + appt.id, "reminder_2h_sent", NOW()
        
        ' Notify staff
        SEND TEMPLATE staff.phone, "staff_appointment_alert", staff.name, customer.name, appt.service, appt_time
    END IF
NEXT appt

' Check for no-shows (30 min past appointment time)
past_appointments = FIND "appointments", "scheduled_at < DATE_SUB(NOW(), INTERVAL 30 MINUTE) AND status='confirmed'"
FOR EACH appt IN past_appointments
    customer = FIND "customers", "id=" + appt.customer_id
    UPDATE "appointments", "id=" + appt.id, "no_show"
    INSERT "activities", appt.customer_id, "no_show", "Missed appointment: " + appt.service, NOW()
    
    ' Send reschedule offer
    SEND TEMPLATE customer.phone, "missed_appointment", customer.name, appt.service
NEXT appt

PRINT "Reminders sent: " + UBOUND(tomorrow_appointments) + " for tomorrow, " + UBOUND(today_appointments) + " for today"
```

### Sales Automation (Vendas)

Automated sales pipeline and lead scoring.

```basic
' sales-automation.bas
' Automated sales pipeline management

SET SCHEDULE "sales-automation", "0 8,14,18 * * 1-5"

' Score and prioritize leads
new_leads = FIND "leads", "score IS NULL OR score = 0"
FOR EACH lead IN new_leads
    score = 0
    
    ' Score based on source
    IF lead.source = "website" THEN score = score + 20
    IF lead.source = "referral" THEN score = score + 30
    IF lead.source = "campaign" THEN score = score + 15
    
    ' Score based on company size
    IF lead.company_size = "enterprise" THEN score = score + 25
    IF lead.company_size = "mid-market" THEN score = score + 20
    IF lead.company_size = "small" THEN score = score + 10
    
    ' Score based on engagement
    page_views = FIND "analytics", "lead_id=" + lead.id + " AND type='page_view'"
    score = score + MIN(UBOUND(page_views) * 2, 20)
    
    ' Score based on email opens
    email_opens = FIND "email_tracking", "lead_id=" + lead.id + " AND opened=true"
    score = score + MIN(UBOUND(email_opens) * 5, 25)
    
    UPDATE "leads", "id=" + lead.id, score, NOW()
NEXT lead

' Auto-assign hot leads to sales reps
hot_leads = FIND "leads", "score >= 70 AND assigned_to IS NULL"
FOR EACH lead IN hot_leads
    ' Round-robin assignment
    available_reps = FIND "attendants", "department='commercial' AND status='online'"
    IF UBOUND(available_reps) > 0 THEN
        ' Get rep with fewest active leads
        rep = available_reps[0]
        min_leads = 999
        FOR EACH r IN available_reps
            rep_leads = FIND "leads", "assigned_to='" + r.id + "' AND status NOT IN ('converted', 'lost')"
            IF UBOUND(rep_leads) < min_leads THEN
                min_leads = UBOUND(rep_leads)
                rep = r
            END IF
        NEXT r
        
        UPDATE "leads", "id=" + lead.id, rep.id, NOW()
        
        ' Notify sales rep via WhatsApp
        SEND TEMPLATE rep.phone, "new_hot_lead", rep.name, lead.name, lead.company, lead.score
        
        ' Create follow-up task
        CREATE TASK "Contact hot lead: " + lead.name, rep.email, NOW()
    END IF
NEXT lead

' Move stale opportunities
stale_opportunities = FIND "opportunities", "DATEDIFF(NOW(), last_activity) > 14 AND stage NOT IN ('closed_won', 'closed_lost')"
FOR EACH opp IN stale_opportunities
    owner = FIND "attendants", "id=" + opp.owner_id
    
    ' Send reminder to owner
    SEND TEMPLATE owner.phone, "stale_opportunity", owner.name, opp.name, opp.amount, DATEDIFF(NOW(), opp.last_activity)
    
    ' Create urgent task
    CREATE TASK "URGENT: Update stale opportunity - " + opp.name, owner.email, NOW()
    
    INSERT "activities", opp.id, "stale_alert", "Opportunity marked as stale", NOW()
NEXT opp

' Generate daily pipeline report
pipeline = FIND "opportunities", "stage NOT IN ('closed_won', 'closed_lost')"
total_value = AGGREGATE "SUM", pipeline, "amount"
weighted_value = 0
FOR EACH opp IN pipeline
    weighted_value = weighted_value + (opp.amount * opp.probability / 100)
NEXT opp

report = "📊 Pipeline Diário\n\n"
report = report + "Total Pipeline: R$ " + FORMAT(total_value, "#,##0.00") + "\n"
report = report + "Valor Ponderado: R$ " + FORMAT(weighted_value, "#,##0.00") + "\n"
report = report + "Oportunidades Ativas: " + UBOUND(pipeline) + "\n"
report = report + "Leads Quentes: " + UBOUND(hot_leads)

SEND MAIL "vendas@empresa.com", "Pipeline Diário - " + FORMAT(NOW(), "DD/MM/YYYY"), report

PRINT "Sales automation completed. Hot leads assigned: " + UBOUND(hot_leads)
```

---

## REST API Endpoints

### Queue Management

#### GET /api/queue
List conversations in queue.

#### POST /api/queue/assign
Assign conversation to attendant.

```json
{
    "session_id": "uuid",
    "attendant_id": "uuid"
}
```

#### POST /api/queue/transfer
Transfer conversation between attendants.

```json
{
    "session_id": "uuid",
    "from_attendant_id": "uuid",
    "to_attendant_id": "uuid",
    "reason": "Specialist needed"
}
```

### Attendant Management

#### GET /api/attendants
List all attendants with stats.

#### PUT /api/attendants/{id}/status
Update attendant status.

```json
{
    "status": "online"
}
```

### CRM Automation

#### GET /api/automation/status
Check automation job status.

#### POST /api/automation/trigger/{job_name}
Manually trigger an automation job.

---

## BASIC Keywords

### Transfer to Human

```basic
' Transfer to any available human
TRANSFER TO HUMAN

' Transfer to specific department
TRANSFER TO HUMAN "sales"

' Transfer with priority
TRANSFER TO HUMAN "support", "high"

' Transfer with context
TRANSFER TO HUMAN "technical", "normal", "Customer needs help with API integration"
```

### Create Lead

```basic
' Create lead from conversation
CREATE LEAD name, email, phone, source

' Create lead with company info
CREATE LEAD name, email, phone, "website", company, "enterprise"
```

### Schedule Appointment

```basic
' Schedule appointment
BOOK customer_email, service, date, time, staff_id

' Schedule with duration
BOOK customer_email, "Consultation", "2025-01-20", "14:00", staff_id, 60
```

---

## WhatsApp Templates

Configure these templates in your WhatsApp Business account:

| Template Name | Purpose | Variables |
|---------------|---------|-----------|
| `follow_up_thanks` | 1-day follow-up | name, interest |
| `follow_up_value` | 3-day value proposition | name, interest |
| `follow_up_offer` | 7-day special offer | name, discount |
| `payment_due_today` | Payment due reminder | name, invoice_id, amount |
| `payment_overdue_3` | 3-day overdue notice | name, invoice_id, amount |
| `payment_overdue_7` | 7-day overdue notice | name, invoice_id, amount |
| `payment_final_notice` | 15-day final notice | name, invoice_id, total |
| `appointment_reminder_24h` | Day-before reminder | name, service, date, time, staff |
| `appointment_reminder_2h` | 2-hour reminder | name, service, time |
| `missed_appointment` | No-show reschedule | name, service |
| `new_hot_lead` | Hot lead alert for sales | rep_name, lead_name, company, score |
| `stale_opportunity` | Stale deal reminder | rep_name, deal_name, amount, days |

---

## See Also

- [Human Approval](../04-basic-scripting/keyword-human-approval.md)
- [SEND TEMPLATE](../04-basic-scripting/keyword-send-template.md)
- [SET SCHEDULE](../04-basic-scripting/keyword-set-schedule.md)
- [CREATE LEAD](../04-basic-scripting/keywords-lead-scoring.md)
- [Sales CRM Template](../02-architecture-packages/template-crm.md)