# Consolidated Examples: Complete Tools & Workflows

This document provides complete, production-ready examples that demonstrate how multiple keywords work together to build real-world tools and automation workflows.

## 1. Customer Support Bot with AI

A complete customer support solution with knowledge base, ticket creation, and escalation.

```basic
' customer-support.bas
' Complete customer support tool with AI-powered responses

' Initialize knowledge bases
USE KB "product-docs"
USE KB "faq"
USE KB "policies"

' Set AI personality
SET CONTEXT "You are a helpful customer support agent for TechCorp. Be friendly, professional, and concise. If you cannot answer a question, offer to create a support ticket."

' Main conversation loop
TALK "Hello! I'm your TechCorp support assistant. How can I help you today?"

conversation_active = TRUE

WHILE conversation_active
    HEAR user_message
    
    ' Check for exit commands
    IF INSTR(LCASE(user_message), "bye") > 0 OR INSTR(LCASE(user_message), "exit") > 0 THEN
        TALK "Thank you for contacting TechCorp support. Have a great day!"
        conversation_active = FALSE
    ELSE IF INSTR(LCASE(user_message), "ticket") > 0 OR INSTR(LCASE(user_message), "human") > 0 THEN
        ' User wants to create a ticket or talk to human
        TALK "I'll create a support ticket for you. Please describe your issue in detail."
        HEAR issue_description
        
        TALK "What's your email address?"
        HEAR customer_email
        
        ' Create ticket in database
                ticket_id = "TKT-" + FORMAT(NOW(), "YYYYMMDDHHmmss")
                description = issue_description
                ticket_status = "open"
                priority = "normal"
                created_at = NOW()
                conversation_history = user_message
        
                SAVE "support_tickets", ticket_id, customer_email, description, ticket_status, priority, created_at, conversation_history
        
                ticket_result = ticket_id
        ticket_id = ticket_result.id
        
        ' Send confirmation email
        email_body = "Your support ticket #" + ticket_id + " has been created. Our team will respond within 24 hours."
        SEND MAIL customer_email, "Support Ticket Created - #" + ticket_id, email_body
        
        ' Notify support team
        SEND MAIL "support@techcorp.com", "New Ticket #" + ticket_id, "Customer: " + customer_email + "\n\nIssue: " + issue_description
        
        TALK "I've created ticket #" + ticket_id + ". You'll receive a confirmation email shortly. Is there anything else I can help with?"
    ELSE
        ' Use AI to answer the question
        response = LLM user_message
        TALK response
        
        ' Log the conversation
        question = user_message
        answer = response
        log_timestamp = NOW()
        INSERT "conversation_logs", question, answer, log_timestamp
        
        TALK "Did that answer your question? (yes/no/create ticket)"
        HEAR feedback
        
        IF INSTR(LCASE(feedback), "no") > 0 THEN
            TALK "I'm sorry I couldn't help. Would you like me to create a support ticket? (yes/no)"
            HEAR create_ticket
            IF INSTR(LCASE(create_ticket), "yes") > 0 THEN
                ' Trigger ticket creation on next loop
                user_message = "create ticket"
            END IF
        END IF
    END IF
WEND
```

## 2. E-Commerce Order Management System

Complete order processing with inventory, payments, and notifications.

```basic
' order-system.bas
' E-commerce order management tool

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
    WITH result = NEW OBJECT
        .status = 400
        .error = "Missing required fields"
    END WITH
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
    ' Notify customer of stock issue
    SEND MAIL customer_email, "Order Issue - Items Out of Stock", "Unfortunately, the following items are out of stock: " + out_of_stock_items
    
    WITH result = NEW OBJECT
        .status = 400
        .error = "Items out of stock"
        .items = out_of_stock_items
    END WITH
    EXIT
END IF

' Process payment
WITH payment_request = NEW OBJECT
    .amount = body.total
    .currency = "USD"
    .token = payment_token
    .description = "Order " + order_id
END WITH

SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "stripe_key"
payment_result = POST "https://api.stripe.com/v1/charges", payment_request

IF payment_result.status <> "succeeded" THEN
    SEND MAIL customer_email, "Payment Failed", "Your payment could not be processed. Please try again."
    
    WITH result = NEW OBJECT
        .status = 402
        .error = "Payment failed"
    END WITH
    EXIT
END IF

' Update inventory
FOR EACH item IN items
    current_stock = FIND "products", "sku=" + item.sku
    new_stock = current_stock.stock - item.quantity
    
    WITH stock_update = NEW OBJECT
        .stock = new_stock
        .updated_at = NOW()
    END WITH
    UPDATE "products", "sku=" + item.sku, stock_update
    
    ' Alert if low stock
    IF new_stock < 10 THEN
        SEND MAIL "inventory@company.com", "Low Stock Alert: " + item.sku, "Stock level: " + new_stock
    END IF
NEXT item

' Create order record
WITH order_record = NEW OBJECT
    .order_id = order_id
    .customer_email = customer_email
    .customer_name = customer_name
    .items = items
    .total = body.total
    .shipping_address = shipping_address
    .payment_id = payment_result.id
    .status = "confirmed"
    .created_at = NOW()
END WITH

SAVE "orders", order_id, order_record

' Generate invoice PDF - pass variables directly
invoice_customer = customer_name
invoice_email = customer_email
invoice_items = order_items
subtotal = body.subtotal
tax = body.tax
shipping_cost = body.shipping_cost
invoice_date = FORMAT(NOW(), "MMMM DD, YYYY")

invoice_pdf = GENERATE PDF "templates/invoice.html", order_id, invoice_customer, invoice_email, invoice_items, subtotal, tax, shipping_cost, total, invoice_date, "invoices/" + order_id + ".pdf"

' Send confirmation email with invoice
email_body = "Thank you for your order, " + customer_name + "!\n\n"
email_body = email_body + "Order #: " + order_id + "\n"
email_body = email_body + "Total: $" + body.total + "\n\n"
email_body = email_body + "Your invoice is attached. You'll receive shipping updates soon."

SEND MAIL customer_email, "Order Confirmed - #" + order_id, email_body, invoice_pdf.url

' Notify warehouse - build simple notification
warehouse_order_id = order_id
warehouse_items = items
warehouse_address = shipping_address
warehouse_priority = "normal"
POST "https://warehouse.internal/api/orders", warehouse_order_id, warehouse_items, warehouse_address, warehouse_priority

' Return success
result_status = "confirmed"
result_order_id = order_id
result_payment_id = payment_result.id
result_invoice_url = invoice_pdf.url
```

## 3. HR Onboarding Automation

Complete employee onboarding workflow.

```basic
' onboarding.bas
' HR onboarding automation tool

WEBHOOK "new-employee"

employee_name = body.name
employee_email = body.email
department = body.department
start_date = body.start_date
manager_email = body.manager_email
role = body.role

' Validate input
IF employee_email = "" OR employee_name = "" THEN
    WITH result = NEW OBJECT
        .status = 400
        .error = "Missing employee name or email"
    END WITH
    EXIT
END IF

' Create employee record
employee_id = "EMP-" + FORMAT(NOW(), "YYYYMMDD") + "-" + LEFT(GUID(), 4)
emp_name = employee_name
emp_email = employee_email
emp_department = department
emp_role = role
emp_manager = manager_email
emp_start_date = start_date
emp_status = "onboarding"
created_at = NOW()

SAVE "employees", employee_id, emp_name, emp_email, emp_department, emp_role, emp_manager, emp_start_date, emp_status, created_at

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
manager_message = "New team member alert!\n\n"
manager_message = manager_message + employee_name + " will be joining your team as " + role + " on " + start_date + ".\n\n"
manager_message = manager_message + "Please prepare:\n"
manager_message = manager_message + "- First week schedule\n"
manager_message = manager_message + "- Team introduction meeting\n"
manager_message = manager_message + "- Project assignments\n"

SEND MAIL manager_email, "New Team Member: " + employee_name, manager_message

' Add to department Slack channel
slack_channel = "#" + LCASE(department)
slack_text = "🎉 Please welcome " + employee_name + " who will be joining us as " + role + " on " + start_date + "!"
POST "https://hooks.slack.com/services/xxx", slack_channel, slack_text

' Schedule 30-60-90 day check-ins
check_in_dates = [30, 60, 90]
FOR EACH days IN check_in_dates
    check_in_date = DATEADD(start_date, days, "day")
    CREATE TASK days + "-day check-in with " + employee_name, manager_email, check_in_date
NEXT days

' Return success
result_status = "success"
result_employee_id = employee_id
result_tasks_created = 9
result_message = "Onboarding initiated for " + employee_name
```

## 4. Daily Business Intelligence Report

Automated daily report with AI-generated insights.

```basic
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

' Gather website analytics
SET HEADER "Authorization", "Bearer " + GET BOT MEMORY "analytics_api_key"
analytics = GET "https://api.analytics.com/v1/summary?date=" + today

' Compile data for AI analysis
report_date = today
sales_revenue = total_revenue_today
sales_orders = order_count_today
sales_change = revenue_change
tickets_opened = UBOUND(tickets_today)
tickets_resolved_count = UBOUND(tickets_resolved)
avg_resolution = avg_resolution_time
low_stock_count = UBOUND(low_stock)
out_of_stock_count = UBOUND(out_of_stock)
visitors = analytics.visitors
page_views = analytics.page_views
bounce_rate = analytics.bounce_rate

report_data = "Date: " + report_date + ", Revenue: $" + sales_revenue + ", Orders: " + sales_orders + ", Change: " + sales_change + "%, Tickets: " + tickets_opened + " opened, " + tickets_resolved_count + " resolved, Low stock: " + low_stock_count

' Generate AI insights
SET CONTEXT "You are a business analyst. Analyze this data and provide actionable insights. Be concise and focus on key trends and recommendations."

analysis_prompt = "Analyze this business data and provide 3-5 key insights:\n\n" + report_data
ai_insights = LLM analysis_prompt

' Build HTML report
report_title = "Daily Business Report - " + today
generated_at = NOW()

report_pdf = GENERATE PDF "templates/daily-report.html", report_title, report_data, ai_insights, generated_at, "reports/daily-" + today + ".pdf"

' Send to executives
executives = ["ceo@company.com", "cfo@company.com", "coo@company.com"]

FOR EACH exec IN executives
    SEND MAIL exec, "Daily Business Report - " + today, "Please find attached today's business intelligence report.\n\n" + ai_insights, report_pdf.url
NEXT exec

' Post summary to Slack
slack_channel = "#executive-updates"
slack_text = "📊 *Daily Report - " + today + "*\n\n"
slack_text = slack_text + "💰 Revenue: $" + FORMAT(total_revenue_today, "#,##0.00") + " (" + FORMAT(revenue_change, "+0.0") + "%)\n"
slack_text = slack_text + "📦 Orders: " + order_count_today + "\n"
slack_text = slack_text + "🎫 Support Tickets: " + UBOUND(tickets_today) + " opened, " + UBOUND(tickets_resolved) + " resolved\n"
slack_text = slack_text + "⚠️ Low Stock Items: " + UBOUND(low_stock) + "\n\n"
slack_text = slack_text + "*AI Insights:*\n" + ai_insights
POST "https://hooks.slack.com/services/xxx", slack_channel, slack_text

' Store report in database
report_date = today
pdf_url = report_pdf.url
SAVE "daily_reports", report_date, report_data, ai_insights, pdf_url

PRINT "Daily report generated and distributed for " + today
```

## 5. Multi-Language Customer Survey Tool

Interactive survey with real-time translation and sentiment analysis.

```basic
' survey-tool.bas
' Multi-language customer survey with AI analysis

TALK "Welcome! What language would you prefer? / ¡Bienvenido! ¿Qué idioma prefiere? / Bienvenue! Quelle langue préférez-vous?"

HEAR language_choice

' Detect language
SET CONTEXT "Identify the language of this text. Respond with only the language code: en, es, fr, de, pt, or other."
detected_lang = LLM language_choice
detected_lang = TRIM(LCASE(detected_lang))

' Set translations
IF detected_lang = "es" THEN
    q1 = "¿Cómo calificaría su experiencia general? (1-10)"
    q2 = "¿Qué es lo que más le gustó de nuestro servicio?"
    q3 = "¿Qué podríamos mejorar?"
    q4 = "¿Nos recomendaría a un amigo? (sí/no)"
    thank_you = "¡Gracias por sus comentarios!"
ELSE IF detected_lang = "fr" THEN
    q1 = "Comment évalueriez-vous votre expérience globale ? (1-10)"
    q2 = "Qu'avez-vous le plus apprécié dans notre service ?"
    q3 = "Que pourrions-nous améliorer ?"
    q4 = "Nous recommanderiez-vous à un ami ? (oui/non)"
    thank_you = "Merci pour vos commentaires !"
ELSE
    q1 = "How would you rate your overall experience? (1-10)"
    q2 = "What did you like most about our service?"
    q3 = "What could we improve?"
    q4 = "Would you recommend us to a friend? (yes/no)"
    thank_you = "Thank you for your feedback!"
    detected_lang = "en"
END IF

' Collect responses
TALK q1
HEAR rating
IF NOT IS NUMERIC(rating) THEN
    rating = 5
END IF

TALK q2
HEAR liked_most

TALK q3
HEAR improvements

TALK q4
HEAR recommend

' Translate responses to English for analysis if needed
IF detected_lang <> "en" THEN
    SET CONTEXT "Translate the following text to English. Only provide the translation, nothing else."
    liked_most_en = LLM liked_most
    improvements_en = LLM improvements
ELSE
    liked_most_en = liked_most
    improvements_en = improvements
END IF

' Sentiment analysis
SET CONTEXT "Analyze the sentiment of this feedback. Respond with: positive, neutral, or negative."
sentiment = LLM "Liked: " + liked_most_en + ". Improvements: " + improvements_en
sentiment = TRIM(LCASE(sentiment))

' NPS calculation
IF INSTR(LCASE(recommend), "yes") > 0 OR INSTR(LCASE(recommend), "sí") > 0 OR INSTR(LCASE(recommend), "oui") > 0 THEN
    would_recommend = TRUE
    nps_category = "promoter"
ELSE
    would_recommend = FALSE
    nps_category = "detractor"
END IF

IF rating >= 9 THEN
    nps_category = "promoter"
ELSE IF rating >= 7 THEN
    nps_category = "passive"
ELSE
    nps_category = "detractor"
END IF

' Save survey response
survey_language = detected_lang
survey_rating = rating
submitted_at = NOW()

SAVE "survey_responses", survey_language, survey_rating, liked_most, liked_most_en, improvements, improvements_en, would_recommend, sentiment, nps_category, submitted_at

' Generate AI summary for low scores
IF rating < 6 THEN
    SET CONTEXT "You are a customer experience manager. Summarize this negative feedback and suggest immediate actions."
    
    alert_summary = LLM "Customer rated us " + rating + "/10. They liked: " + liked_most_en + ". They want us to improve: " + improvements_en
    
    SEND MAIL "cx-team@company.com", "⚠️ Low Survey Score Alert", "Rating: " + rating + "/10\n\nSummary: " + alert_summary + "\n\nOriginal feedback:\n- Liked: " + liked_most + "\n- Improve: " + improvements
END IF

TALK thank_you

' Show aggregated stats
total_responses = AGGREGATE "COUNT", "survey_responses", "id"
avg_rating = AGGREGATE "AVG", "survey_responses", "rating"
promoters = FILTER "survey_responses", "nps_category=promoter"
detractors = FILTER "survey_responses", "nps_category=detractor"
nps_score = ((UBOUND(promoters) - UBOUND(detractors)) / total_responses) * 100

PRINT "Survey completed. Total responses: " + total_responses + ", Avg rating: " + FORMAT(avg_rating, "0.0") + ", NPS: " + FORMAT(nps_score, "0")
```

## 6. Document Processing Pipeline

Automated document intake, processing, and classification.

```basic
' document-pipeline.bas
' Automated document processing and classification

WEBHOOK "document-upload"

' Get uploaded document
document_url = body.document_url
document_name = body.filename
uploader_email = body.uploader_email

IF document_url = "" THEN
    WITH result = NEW OBJECT
        .status = 400
        .error = "No document URL provided"
    END WITH
    EXIT
END IF

' Download document
local_path = DOWNLOAD document_url, "incoming/" + document_name

' Extract text based on file type
file_extension = LCASE(RIGHT(document_name, 4))

IF file_extension = ".pdf" THEN
    content = GET local_path
ELSE IF file_extension = "docx" OR file_extension = ".doc" THEN
    content = GET local_path
ELSE IF file_extension = ".txt" THEN
    content = READ local_path
ELSE
    ' Try OCR for images
    content = GET local_path
END IF

' Classify document using AI
SET CONTEXT "You are a document classifier. Classify this document into one of these categories: invoice, contract, report, correspondence, legal, hr, other. Also extract key metadata. Respond in JSON format with fields: category, confidence, key_dates, key_parties, summary."

classification_prompt = "Classify this document:\n\n" + LEFT(content, 5000)
classification_result = LLM classification_prompt

' Parse AI response (simplified - in production use proper JSON parsing)
category = "other"
IF INSTR(classification_result, "invoice") > 0 THEN
    category = "invoice"
ELSE IF INSTR(classification_result, "contract") > 0 THEN
    category = "contract"
ELSE IF INSTR(classification_result, "report") > 0 THEN
    category = "report"
ELSE IF INSTR(classification_result, "legal") > 0 THEN
    category = "legal"
ELSE IF INSTR(classification_result, "hr") > 0 THEN
    category = "hr"
END IF

' Move to appropriate folder
destination = category + "/" + document_name
MOVE local_path, destination

' Create searchable index entry
filename = document_name
original_url = document_url
stored_path = destination
content_preview = LEFT(content, 1000)
full_text = content
classification_details = classification_result
uploaded_by = uploader_email
processed_at = NOW()

doc_id = INSERT "documents", filename, original_url, stored_path, category, content_preview, full_text, classification_details, uploaded_by, processed_at

' Add to knowledge base for future queries
USE KB category + "-docs"

' Category-specific processing
SELECT CASE category
    CASE "invoice"
        ' Extract invoice details
        SET CONTEXT "Extract from this invoice: vendor name, invoice number, date, due date, total amount, line items. Respond in JSON."
        invoice_data = LLM content
        
        ' Create payable record
        payable_doc_id = doc_id
        extracted_data = invoice_data
        payable_status = "pending_review"
        payable_created_at = NOW()
        INSERT "accounts_payable", payable_doc_id, extracted_data, payable_status, payable_created_at
        
        SEND MAIL "accounting@company.com", "New Invoice for Review", "A new invoice has been uploaded and classified.\n\nDocument: " + document_name + "\nCategory: Invoice\n\nPlease review in the accounting portal."
        
    CASE "contract"
        ' Extract contract details
        SET CONTEXT "Extract from this contract: parties involved, effective date, expiration date, key terms, renewal clauses. Respond in JSON."
        contract_data = LLM content
        
        contract_doc_id = doc_id
        contract_extracted = contract_data
        contract_status = "active"
        contract_created_at = NOW()
        INSERT "contracts", contract_doc_id, contract_extracted, contract_status, contract_created_at
        
        SEND MAIL "legal@company.com", "New Contract Uploaded", "A new contract has been processed.\n\nDocument: " + document_name + "\n\nDetails: " + contract_data
        
    CASE "hr"
        SEND MAIL "hr@company.com", "New HR Document", "A new HR document has been uploaded: " + document_name
END SELECT

' Notify uploader
SEND MAIL uploader_email, "Document Processed: " + document_name, "Your document has been successfully processed.\n\nCategory: " + category + "\nDocument ID: " + doc_id + "\n\nYou can now search for this document in the document portal."

' Return result
result_status = "processed"
result_doc_id = doc_id
result_category = category
result_path = destination
```

## 7. Real-time Data Sync Tool

Bidirectional sync between systems.

```basic
' data-sync.bas
' Real-time data synchronization between CRM and ERP

WEBHOOK "crm-update"

' Handle CRM webhook
event_type = body.event
record_type = body.record_type
record_id = body.record_id
data = body.data
timestamp = body.timestamp

' Log sync event
sync_source = "crm"
sync_event = event_type
sync_record_type = record_type
sync_record_id = record_id
sync_timestamp = source_timestamp
received_at = NOW()
INSERT "sync_logs", sync_source, sync_event, sync_record_type, sync_record_id, sync_timestamp, received_at

' Check for sync conflicts
last_erp_update = FIND "erp_sync_status", "record_id=" + record_id
IF last_erp_update.updated_at > timestamp THEN
    ' ERP has newer data - conflict
    WITH conflict = NEW OBJECT
        .record_id = record_id
        .crm_timestamp = timestamp
        .erp_timestamp = last_erp_update.updated_at
        .crm_data = data
        .erp_data = last_erp_update.data
        .status = "pending_resolution"
    END WITH
    INSERT "sync_conflicts", conflict
    
    SEND MAIL "data-admin@company.com", "Sync Conflict Detected", "Record " + record_id + " has conflicting updates. Please resolve in the admin portal."
    
    WITH result = NEW OBJECT
        .status = "conflict"
        .message = "Newer data exists in ERP"
    END WITH
    EXIT
END IF

' Transform data for ERP format
SELECT CASE record_type
    CASE "customer"
        WITH erp_data = NEW OBJECT
            .customer_code = record_id
            .company_name = data.company
            .contact_name = data.contact_first_name + " " + data.contact_last_name
            .email = data.email
            .phone = data.phone
            .billing_address = data.address.street + ", " + data.address.city + ", " + data.address.state + " " + data.address.zip
            .credit_limit = data.credit_limit
            .payment_terms = data.payment_terms
            .updated_at = NOW()
        END WITH
        
        erp_endpoint = "/api/customers/" + record_id
        
    CASE "order"
        WITH erp_data = NEW OBJECT
            .order_number = record_id
            .customer_code = data.customer_id
            .order_date = data.created_at
            .ship_date = data.expected_ship_date
            .line_items = data.products
            .subtotal = data.subtotal
            .tax = data.tax
            .total = data.total
            .status = data.status
        END WITH
        
        erp_endpoint = "/api/orders/" + record_id
        
    CASE "product"
        WITH erp_data = NEW OBJECT
            .sku = record_id
            .description = data.name
            .category = data.category
            .unit_price = data.price
            .cost = data.cost
            .weight = data.weight
            .dimensions = data.dimensions
        END WITH
        
        erp_endpoint = "/api/products/" + record_id
END SELECT

' Send to ERP
erp_api_key = GET BOT MEMORY "erp_api_key"
SET HEADER "Authorization", "Bearer " + erp_api_key
SET HEADER "Content-Type", "application/json"

IF event_type = "create" THEN
    erp_result = POST "https://erp.company.com" + erp_endpoint, erp_data
ELSE IF event_type = "update" THEN
    erp_result = PUT "https://erp.company.com" + erp_endpoint, erp_data
ELSE IF event_type = "delete" THEN
    erp_result = DELETE "https://erp.company.com" + erp_endpoint
END IF

' Update sync status
WITH sync_status = NEW OBJECT
    .record_id = record_id
    .record_type = record_type
    .last_crm_update = timestamp
    .last_erp_sync = NOW()
    .sync_result = erp_result.status
    .data = erp_data
END WITH
SAVE "erp_sync_status", record_id, sync_status

' Update sync log with result
WITH log_update = NEW OBJECT
    .sync_completed_at = NOW()
    .erp_response = erp_result.status
END WITH
UPDATE "sync_logs", "record_id=" + record_id + " AND source='crm'", log_update

WITH result = NEW OBJECT
    .status = "synced"
    .record_id = record_id
    .erp_status = erp_result.status
END WITH
```

## See Also

- [Keywords Reference](./keywords.md) - Complete keyword documentation
- [WEBHOOK](./keyword-webhook.md) - Creating API endpoints
- [SET SCHEDULE](./keyword-set-schedule.md) - Scheduled automation
- [Data Operations](./keywords.md#database--data-operations) - Database keywords
- [File Operations](./keywords.md#file--document-operations) - File handling