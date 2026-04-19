# Webhook Integrations and Jobs Examples

This document provides complete, production-ready examples demonstrating webhook endpoints and scheduled jobs. All examples use event-driven patterns—no conversation loops.

---

## 1. E-Commerce Order Management System

Complete order processing with inventory, payments, and notifications via webhook.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L1-80
' order-system.bas
' E-commerce order management webhook

WEBHOOK "new-order"

' Extract order data
order_id = body.order_id
customer_email = body.customer.email
customer_name = body.customer.name
items = body.items
shipping_address = body.shipping
payment_method = body.payment.method
payment_token = body.payment.token

' Validate order
IF order_id = "" OR customer_email = "" THEN
    result_status = 400
    result_error = "Missing required fields"
    EXIT
END IF

' Check inventory for all items
inventory_ok = TRUE
out_of_stock_items = ""

FOR EACH item IN items
    product = FIND "products", "sku=" + item.sku
    IF product.stock < item.quantity THEN
        inventory_ok = FALSE
        out_of_stock_items = out_of_stock_items + item.name + ", "
    END IF
NEXT item

IF NOT inventory_ok THEN
    SEND MAIL customer_email, "Order Issue - Items Out of Stock", "Unfortunately, the following items are out of stock: " + out_of_stock_items
    result_status = 400
    result_error = "Items out of stock"
    EXIT
END IF

' Process payment
SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "stripe_key"
payment_result = POST "https://api.stripe.com/v1/charges", body.total, "USD", payment_token, "Order " + order_id

IF payment_result.status <> "succeeded" THEN
    SEND MAIL customer_email, "Payment Failed", "Your payment could not be processed. Please try again."
    result_status = 402
    result_error = "Payment failed"
    EXIT
END IF

' Update inventory
FOR EACH item IN items
    current_stock = FIND "products", "sku=" + item.sku
    new_stock = current_stock.stock - item.quantity
    UPDATE "products", "sku=" + item.sku, new_stock, NOW()
    
    IF new_stock < 10 THEN
        SEND MAIL "inventory@company.com", "Low Stock Alert: " + item.sku, "Stock level: " + new_stock
    END IF
NEXT item

' Save order record
SAVE "orders", order_id, customer_email, customer_name, items, body.total, shipping_address, payment_result.id, "confirmed", NOW()

' Generate invoice PDF
invoice_pdf = GENERATE PDF "templates/invoice.html", order_id, customer_name, customer_email, items, body.subtotal, body.tax, body.shipping_cost, body.total, FORMAT(NOW(), "MMMM DD, YYYY"), "invoices/" + order_id + ".pdf"

' Send confirmation email
email_body = "Thank you for your order, " + customer_name + "!\n\nOrder #: " + order_id + "\nTotal: $" + body.total + "\n\nYour invoice is attached."
SEND MAIL customer_email, "Order Confirmed - #" + order_id, email_body, invoice_pdf.url

' Notify warehouse
POST "https://warehouse.internal/api/orders", order_id, items, shipping_address, "normal"

result_status = "confirmed"
result_order_id = order_id
result_payment_id = payment_result.id
```

---

## 2. HR Onboarding Automation

Complete employee onboarding workflow triggered by webhook.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L82-150
' onboarding.bas
' HR onboarding automation webhook

WEBHOOK "new-employee"

employee_name = body.name
employee_email = body.email
department = body.department
start_date = body.start_date
manager_email = body.manager_email
role = body.role

' Validate input
IF employee_email = "" OR employee_name = "" THEN
    result_status = 400
    result_error = "Missing employee name or email"
    EXIT
END IF

' Create employee record
employee_id = "EMP-" + FORMAT(NOW(), "YYYYMMDD") + "-" + LEFT(GUID(), 4)
SAVE "employees", employee_id, employee_name, employee_email, department, role, manager_email, start_date, "onboarding", NOW()

' Create tasks for IT setup
CREATE TASK "Create email account for " + employee_name, "it@company.com", start_date
CREATE TASK "Setup laptop for " + employee_name, "it@company.com", start_date
CREATE TASK "Create " + department + " system access for " + employee_name, "it@company.com", start_date

' Create tasks for HR
CREATE TASK "Prepare employment documents for " + employee_name, "hr@company.com", start_date
CREATE TASK "Schedule orientation for " + employee_name, "hr@company.com", start_date
CREATE TASK "Add " + employee_name + " to benefits enrollment", "hr@company.com", start_date

' Send welcome email to new employee
USE KB "employee-handbook"
SET CONTEXT "You are a friendly HR assistant. Create a warm, professional welcome message."
welcome_content = LLM "Write a welcome email for " + employee_name + " joining as " + role + " in " + department + " department, starting on " + start_date
SEND MAIL employee_email, "Welcome to the Team, " + employee_name + "!", welcome_content

' Notify manager
manager_message = "New team member alert!\n\n" + employee_name + " will be joining your team as " + role + " on " + start_date + ".\n\nPlease prepare:\n- First week schedule\n- Team introduction meeting\n- Project assignments"
SEND MAIL manager_email, "New Team Member: " + employee_name, manager_message

' Post to Slack
slack_channel = "#" + LCASE(department)
POST "https://hooks.slack.com/services/xxx", slack_channel, "🎉 Please welcome " + employee_name + " who will be joining us as " + role + " on " + start_date + "!"

' Schedule 30-60-90 day check-ins
check_in_dates = [30, 60, 90]
FOR EACH days IN check_in_dates
    check_in_date = DATEADD(start_date, days, "day")
    CREATE TASK days + "-day check-in with " + employee_name, manager_email, check_in_date
NEXT days

result_status = "success"
result_employee_id = employee_id
result_tasks_created = 9
```

---

## 3. Daily Business Intelligence Report

Automated daily report job with AI-generated insights.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L152-230
' daily-report.bas
' Automated daily business intelligence report

SET SCHEDULE "daily-bi-report", "0 7 * * 1-5"

today = FORMAT(NOW(), "YYYY-MM-DD")
yesterday = FORMAT(DATEADD(NOW(), -1, "day"), "YYYY-MM-DD")

' Gather sales data
sales_today = FIND "orders", "DATE(created_at)='" + today + "'"
sales_yesterday = FIND "orders", "DATE(created_at)='" + yesterday + "'"

total_revenue_today = AGGREGATE "SUM", sales_today, "total"
total_revenue_yesterday = AGGREGATE "SUM", sales_yesterday, "total"
order_count_today = AGGREGATE "COUNT", sales_today, "id"

revenue_change = ((total_revenue_today - total_revenue_yesterday) / total_revenue_yesterday) * 100

' Gather support metrics
tickets_today = FIND "support_tickets", "DATE(created_at)='" + today + "'"
tickets_resolved = FILTER tickets_today, "status=resolved"
avg_resolution_time = AGGREGATE "AVG", tickets_resolved, "resolution_time_hours"

' Gather inventory alerts
low_stock = FIND "products", "stock < 10"
out_of_stock = FIND "products", "stock = 0"

' Compile data for AI analysis
report_data = "Date: " + today + ", Revenue: $" + total_revenue_today + ", Orders: " + order_count_today + ", Change: " + revenue_change + "%, Tickets: " + UBOUND(tickets_today) + " opened, " + UBOUND(tickets_resolved) + " resolved, Low stock: " + UBOUND(low_stock)

' Generate AI insights
SET CONTEXT "You are a business analyst. Analyze this data and provide actionable insights. Be concise and focus on key trends and recommendations."
ai_insights = LLM "Analyze this business data and provide 3-5 key insights:\n\n" + report_data

' Build report PDF
report_pdf = GENERATE PDF "templates/daily-report.html", "Daily Business Report - " + today, report_data, ai_insights, NOW(), "reports/daily-" + today + ".pdf"

' Send to executives
executives = ["ceo@company.com", "cfo@company.com", "coo@company.com"]
FOR EACH exec IN executives
    SEND MAIL exec, "Daily Business Report - " + today, "Please find attached today's business intelligence report.\n\n" + ai_insights, report_pdf.url
NEXT exec

' Post summary to Slack
slack_summary = "📊 *Daily Report - " + today + "*\n\n💰 Revenue: $" + FORMAT(total_revenue_today, "#,##0.00") + " (" + FORMAT(revenue_change, "+0.0") + "%)\n📦 Orders: " + order_count_today + "\n🎫 Support Tickets: " + UBOUND(tickets_today) + " opened, " + UBOUND(tickets_resolved) + " resolved\n⚠️ Low Stock Items: " + UBOUND(low_stock)
POST "https://hooks.slack.com/services/xxx", "#executive-updates", slack_summary

' Store report in database
SAVE "daily_reports", today, report_data, ai_insights, report_pdf.url

PRINT "Daily report generated and distributed for " + today
```

---

## 4. Document Processing Pipeline

Automated document intake, processing, and classification via webhook.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L232-330
' document-pipeline.bas
' Automated document processing and classification

WEBHOOK "document-upload"

document_url = body.document_url
document_name = body.filename
uploader_email = body.uploader_email

IF document_url = "" THEN
    result_status = 400
    result_error = "No document URL provided"
    EXIT
END IF

' Download document
local_path = DOWNLOAD document_url, "incoming/" + document_name

' Extract text based on file type
file_extension = LCASE(RIGHT(document_name, 4))
content = GET local_path

' Classify document using AI
SET CONTEXT "You are a document classifier. Classify this document into one of these categories: invoice, contract, report, correspondence, legal, hr, other. Respond with just the category name."
classification_prompt = "Classify this document:\n\n" + LEFT(content, 5000)
category = TRIM(LCASE(LLM classification_prompt))

' Move to appropriate folder
destination = category + "/" + document_name
MOVE local_path, destination

' Create searchable index entry
doc_id = INSERT "documents", document_name, document_url, destination, category, LEFT(content, 1000), content, uploader_email, NOW()

' Add to knowledge base for future queries
USE KB category + "-docs"

' Category-specific processing
IF category = "invoice" THEN
    SET CONTEXT "Extract from this invoice: vendor name, invoice number, date, due date, total amount. Respond in JSON."
    invoice_data = LLM content
    INSERT "accounts_payable", doc_id, invoice_data, "pending_review", NOW()
    SEND MAIL "accounting@company.com", "New Invoice for Review", "A new invoice has been uploaded.\n\nDocument: " + document_name
END IF

IF category = "contract" THEN
    SET CONTEXT "Extract from this contract: parties involved, effective date, expiration date, key terms. Respond in JSON."
    contract_data = LLM content
    INSERT "contracts", doc_id, contract_data, "active", NOW()
    SEND MAIL "legal@company.com", "New Contract Uploaded", "A new contract has been processed.\n\nDocument: " + document_name
END IF

IF category = "hr" THEN
    SEND MAIL "hr@company.com", "New HR Document", "A new HR document has been uploaded: " + document_name
END IF

' Notify uploader
SEND MAIL uploader_email, "Document Processed: " + document_name, "Your document has been successfully processed.\n\nCategory: " + category + "\nDocument ID: " + doc_id

result_status = "processed"
result_doc_id = doc_id
result_category = category
```

---

## 5. Real-time Data Sync (CRM to ERP)

Bidirectional sync between systems via webhook.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L332-420
' data-sync.bas
' Real-time data synchronization between CRM and ERP

WEBHOOK "crm-update"

event_type = body.event
record_type = body.record_type
record_id = body.record_id
data = body.data
timestamp = body.timestamp

' Log sync event
INSERT "sync_logs", "crm", event_type, record_type, record_id, timestamp, NOW()

' Check for sync conflicts
last_erp_update = FIND "erp_sync_status", "record_id=" + record_id
IF last_erp_update.updated_at > timestamp THEN
    INSERT "sync_conflicts", record_id, timestamp, last_erp_update.updated_at, data, last_erp_update.data, "pending_resolution"
    SEND MAIL "data-admin@company.com", "Sync Conflict Detected", "Record " + record_id + " has conflicting updates. Please resolve in the admin portal."
    result_status = "conflict"
    result_message = "Newer data exists in ERP"
    EXIT
END IF

' Transform data for ERP format based on record type
IF record_type = "customer" THEN
    erp_endpoint = "/api/customers/" + record_id
    erp_customer_code = record_id
    erp_company_name = data.company
    erp_contact_name = data.contact_first_name + " " + data.contact_last_name
    erp_email = data.email
    erp_phone = data.phone
END IF

IF record_type = "order" THEN
    erp_endpoint = "/api/orders/" + record_id
    erp_order_number = record_id
    erp_customer_code = data.customer_id
    erp_order_date = data.created_at
    erp_total = data.total
END IF

IF record_type = "product" THEN
    erp_endpoint = "/api/products/" + record_id
    erp_sku = record_id
    erp_description = data.name
    erp_unit_price = data.price
END IF

' Send to ERP
erp_api_key = GET BOT MEMORY "erp_api_key"
SET HEADER "Authorization", "Bearer " + erp_api_key
SET HEADER "Content-Type", "application/json"

IF event_type = "create" THEN
    erp_result = POST "https://erp.company.com" + erp_endpoint, data
ELSE IF event_type = "update" THEN
    erp_result = PUT "https://erp.company.com" + erp_endpoint, data
ELSE IF event_type = "delete" THEN
    erp_result = DELETE "https://erp.company.com" + erp_endpoint
END IF

' Update sync status
SAVE "erp_sync_status", record_id, record_type, timestamp, NOW(), erp_result.status, data

result_status = "synced"
result_record_id = record_id
result_erp_status = erp_result.status
```

---

## 6. Scheduled Lead Nurturing Job

Automated lead follow-up and nurturing campaign.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L422-490
' lead-nurturing.bas
' Scheduled lead nurturing campaign

SET SCHEDULE "lead-nurture", "0 9 * * *"

' Find leads needing follow-up
cold_leads_3_days = FIND "leads", "status='cold' AND DATEDIFF(NOW(), last_contact) >= 3"
cold_leads_7_days = FIND "leads", "status='cold' AND DATEDIFF(NOW(), last_contact) >= 7"
cold_leads_14_days = FIND "leads", "status='cold' AND DATEDIFF(NOW(), last_contact) >= 14"
cold_leads_30_days = FIND "leads", "status='cold' AND DATEDIFF(NOW(), last_contact) >= 30"

' 3-day follow-up: Tips email
FOR EACH lead IN cold_leads_3_days
    IF lead.nurture_stage = 0 THEN
        SEND MAIL lead.email, "5 Tips to Improve Your Business", "templates/nurture-tips.html"
        UPDATE "leads", "id=" + lead.id, 1, NOW()
    END IF
NEXT lead

' 7-day follow-up: Case study
FOR EACH lead IN cold_leads_7_days
    IF lead.nurture_stage = 1 THEN
        SEND MAIL lead.email, "Case Study: How We Helped Similar Companies", "templates/nurture-case-study.html"
        UPDATE "leads", "id=" + lead.id, 2, NOW()
    END IF
NEXT lead

' 14-day follow-up: Free consultation
FOR EACH lead IN cold_leads_14_days
    IF lead.nurture_stage = 2 THEN
        SEND MAIL lead.email, "Free Consultation Offer", "templates/nurture-consultation.html"
        UPDATE "leads", "id=" + lead.id, 3, NOW()
    END IF
NEXT lead

' 30-day follow-up: Special offer
FOR EACH lead IN cold_leads_30_days
    IF lead.nurture_stage = 3 THEN
        SEND MAIL lead.email, "Special Limited Time Offer", "templates/nurture-special-offer.html"
        UPDATE "leads", "id=" + lead.id, 4, NOW()
    END IF
NEXT lead

' Log nurturing stats
PRINT "Lead nurturing completed: " + UBOUND(cold_leads_3_days) + " at stage 1, " + UBOUND(cold_leads_7_days) + " at stage 2"
```

---

## 7. Payment Collection Reminder Job

Automated payment reminders and collection workflow.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L492-560
' payment-collection.bas
' Scheduled payment collection reminders

SET SCHEDULE "payment-reminders", "0 8 * * 1-5"

' Find overdue invoices
due_today = FIND "invoices", "status='pending' AND due_date = CURDATE()"
overdue_3_days = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 3"
overdue_7_days = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 7"
overdue_14_days = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) = 14"
overdue_30_days = FIND "invoices", "status='pending' AND DATEDIFF(NOW(), due_date) >= 30"

' Due today reminder
FOR EACH invoice IN due_today
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND MAIL customer.email, "Payment Due Today - Invoice #" + invoice.id, "Your invoice #" + invoice.id + " for $" + invoice.amount + " is due today. Please make payment to avoid late fees."
NEXT invoice

' 3-day overdue: First reminder
FOR EACH invoice IN overdue_3_days
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND MAIL customer.email, "Payment Overdue - Invoice #" + invoice.id, "Your invoice #" + invoice.id + " for $" + invoice.amount + " is now 3 days overdue. Please remit payment as soon as possible."
    UPDATE "invoices", "id=" + invoice.id, "first_reminder_sent", NOW()
NEXT invoice

' 7-day overdue: Second reminder with late fee warning
FOR EACH invoice IN overdue_7_days
    customer = FIND "customers", "id=" + invoice.customer_id
    SEND MAIL customer.email, "URGENT: Payment Overdue - Invoice #" + invoice.id, "Your invoice #" + invoice.id + " is now 7 days overdue. A late fee may be applied if not paid within 7 days."
    UPDATE "invoices", "id=" + invoice.id, "second_reminder_sent", NOW()
NEXT invoice

' 14-day overdue: Final notice
FOR EACH invoice IN overdue_14_days
    customer = FIND "customers", "id=" + invoice.customer_id
    late_fee = invoice.amount * 0.05
    new_total = invoice.amount + late_fee
    SEND MAIL customer.email, "FINAL NOTICE: Invoice #" + invoice.id, "Your invoice is now 14 days overdue. A 5% late fee ($" + late_fee + ") has been applied. New total: $" + new_total
    UPDATE "invoices", "id=" + invoice.id, late_fee, new_total, "final_notice_sent", NOW()
    
    ' Notify accounts receivable
    SEND MAIL "ar@company.com", "Invoice Escalation: #" + invoice.id, "Invoice #" + invoice.id + " for " + customer.name + " is 14 days overdue. Amount: $" + new_total
NEXT invoice

' 30+ day overdue: Send to collections
FOR EACH invoice IN overdue_30_days
    IF invoice.status <> "collections" THEN
        customer = FIND "customers", "id=" + invoice.customer_id
        UPDATE "invoices", "id=" + invoice.id, "collections", NOW()
        
        ' Notify collections team
        SEND MAIL "collections@company.com", "New Collections Account: " + customer.name, "Invoice #" + invoice.id + " - $" + invoice.total_with_fees + "\nCustomer: " + customer.name + "\nDays overdue: " + DATEDIFF(NOW(), invoice.due_date)
    END IF
NEXT invoice

PRINT "Payment reminders sent: " + UBOUND(due_today) + " due today, " + UBOUND(overdue_3_days) + " 3-day, " + UBOUND(overdue_7_days) + " 7-day"
```

---

## 8. Appointment Scheduling Webhook

Handle appointment bookings from external calendar systems.

```botbook/src/04-basic-scripting/examples-consolidated.bas#L562-620
' appointment-webhook.bas
' Handle appointment scheduling from external systems

WEBHOOK "appointment-booked"

appointment_id = body.appointment_id
customer_email = body.customer.email
customer_name = body.customer.name
customer_phone = body.customer.phone
service_type = body.service
appointment_date = body.date
appointment_time = body.time
staff_id = body.staff_id

' Validate
IF appointment_id = "" OR customer_email = "" THEN
    result_status = 400
    result_error = "Missing required fields"
    EXIT
END IF

' Check staff availability
existing = FIND "appointments", "staff_id='" + staff_id + "' AND date='" + appointment_date + "' AND time='" + appointment_time + "'"
IF UBOUND(existing) > 0 THEN
    result_status = 409
    result_error = "Time slot not available"
    EXIT
END IF

' Save appointment
SAVE "appointments", appointment_id, customer_email, customer_name, customer_phone, service_type, appointment_date, appointment_time, staff_id, "confirmed", NOW()

' Get staff info
staff = FIND "staff", "id=" + staff_id

' Send confirmation to customer
confirmation_msg = "Your appointment has been confirmed!\n\n📅 " + appointment_date + " at " + appointment_time + "\n🏢 Service: " + service_type + "\n👤 With: " + staff.name + "\n\nPlease arrive 10 minutes early."
SEND MAIL customer_email, "Appointment Confirmed - " + service_type, confirmation_msg

' Send SMS reminder setup
SET SCHEDULE "reminder-" + appointment_id, DATEADD(appointment_date + " " + appointment_time, -24, "hour")

' Notify staff
SEND MAIL staff.email, "New Appointment: " + customer_name, "You have a new appointment:\n\n📅 " + appointment_date + " at " + appointment_time + "\n👤 Customer: " + customer_name + "\n📞 Phone: " + customer_phone + "\n🏢 Service: " + service_type

' Add to calendar
BOOK staff.email, "Appointment: " + customer_name + " - " + service_type, appointment_date, appointment_time, 60

result_status = "confirmed"
result_appointment_id = appointment_id
```

---

## See Also

- [Keywords Reference](./keywords.md) — Complete keyword documentation
- [WEBHOOK](./keyword-webhook.md) — Creating API endpoints
- [SET SCHEDULE](./keyword-set-schedule.md) — Scheduled automation
- [Data Operations](./keywords-data.md) — Database keywords
- [File Operations](./keywords-file.md) — File handling
- [HTTP Operations](./keywords-http.md) — REST API calls