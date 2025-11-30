PARAM action AS STRING
PARAM opp_data AS OBJECT

user_id = GET "session.user_id"
opportunity_id = GET "session.opportunity_id"
account_id = GET "session.account_id"
current_time = FORMAT NOW() AS "YYYY-MM-DD HH:mm:ss"

IF action = "create" THEN
    opp_name = GET "opp_data.name"
    opp_value = GET "opp_data.value"
    close_date = GET "opp_data.close_date"

    IF account_id = "" THEN
        TALK "Which account is this opportunity for?"
        account_name = HEAR
        account = FIND "accounts", "name LIKE '%" + account_name + "%'"
        IF account != NULL THEN
            account_id = account.id
        ELSE
            TALK "Account not found. Please create the account first."
            EXIT
        END IF
    END IF

    IF opp_name = "" THEN
        TALK "What should we call this opportunity?"
        opp_name = HEAR
    END IF

    IF opp_value = "" THEN
        TALK "What is the estimated value of this deal?"
        opp_value = HEAR
    END IF

    IF close_date = "" THEN
        TALK "When do you expect to close this deal? (YYYY-MM-DD)"
        close_date = HEAR
    END IF

    opportunity = CREATE OBJECT
    SET opportunity.id = FORMAT GUID()
    SET opportunity.name = opp_name
    SET opportunity.account_id = account_id
    SET opportunity.amount = opp_value
    SET opportunity.close_date = close_date
    SET opportunity.stage = "qualification"
    SET opportunity.probability = 10
    SET opportunity.owner_id = user_id
    SET opportunity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opportunity AS JSON

    SET "session.opportunity_id" = opportunity.id
    REMEMBER "opportunity_" + opportunity.id = opportunity

    TALK "Opportunity created: " + opp_name + " valued at $" + opp_value

    CREATE_TASK "Qualify opportunity: " + opp_name, "high", user_id

    activity = CREATE OBJECT
    SET activity.type = "opportunity_created"
    SET activity.opportunity_id = opportunity.id
    SET activity.description = "Created opportunity: " + opp_name
    SET activity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "activities", FORMAT activity AS JSON

END IF

IF action = "update_stage" THEN
    IF opportunity_id = "" THEN
        TALK "Which opportunity do you want to update?"
        opp_name = HEAR
        opportunity = FIND "opportunities", "name LIKE '%" + opp_name + "%'"
        IF opportunity != NULL THEN
            opportunity_id = opportunity.id
        ELSE
            TALK "Opportunity not found."
            EXIT
        END IF
    END IF

    opportunity = FIND "opportunities", "id = '" + opportunity_id + "'"

    IF opportunity = NULL THEN
        TALK "Opportunity not found."
        EXIT
    END IF

    TALK "Current stage: " + opportunity.stage
    TALK "Select new stage:"
    TALK "1. Qualification (10%)"
    TALK "2. Needs Analysis (20%)"
    TALK "3. Value Proposition (50%)"
    TALK "4. Decision Makers (60%)"
    TALK "5. Proposal (75%)"
    TALK "6. Negotiation (90%)"
    TALK "7. Closed Won (100%)"
    TALK "8. Closed Lost (0%)"

    stage_choice = HEAR

    new_stage = ""
    new_probability = 0

    IF stage_choice = "1" THEN
        new_stage = "qualification"
        new_probability = 10
    ELSE IF stage_choice = "2" THEN
        new_stage = "needs_analysis"
        new_probability = 20
    ELSE IF stage_choice = "3" THEN
        new_stage = "value_proposition"
        new_probability = 50
    ELSE IF stage_choice = "4" THEN
        new_stage = "decision_makers"
        new_probability = 60
    ELSE IF stage_choice = "5" THEN
        new_stage = "proposal"
        new_probability = 75
    ELSE IF stage_choice = "6" THEN
        new_stage = "negotiation"
        new_probability = 90
    ELSE IF stage_choice = "7" THEN
        new_stage = "closed_won"
        new_probability = 100
        opportunity.won = true
        opportunity.closed_at = current_time
    ELSE IF stage_choice = "8" THEN
        new_stage = "closed_lost"
        new_probability = 0
        opportunity.won = false
        opportunity.closed_at = current_time
    END IF

    old_stage = opportunity.stage
    opportunity.stage = new_stage
    opportunity.probability = new_probability
    opportunity.updated_at = current_time

    SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opportunity AS JSON

    REMEMBER "opportunity_stage_" + opportunity_id = new_stage

    activity = CREATE OBJECT
    SET activity.type = "stage_change"
    SET activity.opportunity_id = opportunity_id
    SET activity.description = "Stage changed from " + old_stage + " to " + new_stage
    SET activity.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "activities", FORMAT activity AS JSON

    TALK "Stage updated to " + new_stage + " (" + new_probability + "%)"

    IF new_stage = "closed_won" THEN
        TALK "Congratulations! Deal closed for $" + opportunity.amount

        notification = "Deal Won: " + opportunity.name + " - $" + opportunity.amount
        SEND MAIL "management@company.com", "Deal Won", notification

        CREATE_TASK "Onboard new customer: " + opportunity.name, "high", user_id

    ELSE IF new_stage = "closed_lost" THEN
        TALK "What was the reason for losing this deal?"
        loss_reason = HEAR

        opportunity.loss_reason = loss_reason
        SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opportunity AS JSON

        CREATE_TASK "Analyze lost deal: " + opportunity.name, "low", user_id
    END IF

END IF

IF action = "add_product" THEN
    IF opportunity_id = "" THEN
        TALK "No opportunity selected."
        EXIT
    END IF

    TALK "Enter product name or code:"
    product_search = HEAR

    product = FIND "products", "name LIKE '%" + product_search + "%' OR code = '" + product_search + "'"

    IF product = NULL THEN
        TALK "Product not found."
        EXIT
    END IF

    TALK "How many units?"
    quantity = HEAR

    TALK "Any discount percentage? (0 for none)"
    discount = HEAR

    line_item = CREATE OBJECT
    SET line_item.id = FORMAT GUID()
    SET line_item.opportunity_id = opportunity_id
    SET line_item.product_id = product.id
    SET line_item.product_name = product.name
    SET line_item.quantity = quantity
    SET line_item.unit_price = product.unit_price
    SET line_item.discount = discount
    SET line_item.total = quantity * product.unit_price * (1 - discount / 100)
    SET line_item.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "opportunity_products", FORMAT line_item AS JSON

    opportunity = FIND "opportunities", "id = '" + opportunity_id + "'"
    opportunity.amount = opportunity.amount + line_item.total
    SAVE_FROM_UNSTRUCTURED "opportunities", FORMAT opportunity AS JSON

    TALK "Added " + quantity + " x " + product.name + " = $" + line_item.total

END IF

IF action = "generate_quote" THEN
    IF opportunity_id = "" THEN
        TALK "No opportunity selected."
        EXIT
    END IF

    opportunity = FIND "opportunities", "id = '" + opportunity_id + "'"
    products = FIND "opportunity_products", "opportunity_id = '" + opportunity_id + "'"

    IF products = NULL THEN
        TALK "No products added to this opportunity."
        EXIT
    END IF

    account = FIND "accounts", "id = '" + opportunity.account_id + "'"
    contact = FIND "contacts", "account_id = '" + opportunity.account_id + "' AND primary_contact = true"

    quote = CREATE OBJECT
    SET quote.id = FORMAT GUID()
    SET quote.quote_number = "Q-" + FORMAT NOW() AS "YYYYMMDD" + "-" + FORMAT RANDOM(1000, 9999)
    SET quote.opportunity_id = opportunity_id
    SET quote.account_id = account.id
    SET quote.contact_id = contact.id
    SET quote.status = "draft"
    SET quote.valid_until = FORMAT ADD_DAYS(NOW(), 30) AS "YYYY-MM-DD"
    SET quote.subtotal = opportunity.amount
    SET quote.tax_rate = 10
    SET quote.tax_amount = opportunity.amount * 0.1
    SET quote.total = opportunity.amount * 1.1
    SET quote.created_at = current_time

    SAVE_FROM_UNSTRUCTURED "quotes", FORMAT quote AS JSON

    REMEMBER "quote_" + quote.id = quote

    quote_content = "QUOTATION\n"
    quote_content = quote_content + "Quote #: " + quote.quote_number + "\n"
    quote_content = quote_content + "Date: " + current_time + "\n"
    quote_content = quote_content + "Valid Until: " + quote.valid_until + "\n\n"
    quote_content = quote_content + "To: " + account.name + "\n"
    quote_content = quote_content + "Contact: " + contact.name + "\n\n"
    quote_content = quote_content + "ITEMS:\n"

    FOR EACH item IN products DO
        quote_content = quote_content + item.product_name + " x " + item.quantity + " @ $" + item.unit_price + " = $" + item.total + "\n"
    END FOR

    quote_content = quote_content + "\nSubtotal: $" + quote.subtotal + "\n"
    quote_content = quote_content + "Tax (10%): $" + quote.tax_amount + "\n"
    quote_content = quote_content + "TOTAL: $" + quote.total + "\n"

    CREATE_DRAFT quote_content, "Quote " + quote.quote_number + " for " + account.name

    TALK "Quote " + quote.quote_number + " generated for $" + quote.total

    IF contact.email != "" THEN
        TALK "Send quote to " + contact.name + " at " + contact.email + "? (yes/no)"
        send_quote = HEAR

        IF send_quote = "yes" OR send_quote = "YES" OR send_quote = "Yes" THEN
            subject = "Quote " + quote.quote_number + " from Our Company"
            SEND MAIL contact.email, subject, quote_content

            quote.status = "sent"
            quote.sent_at = current_time
            SAVE_FROM_UNSTRUCTURED "quotes", FORMAT quote AS JSON

            TALK "Quote sent to " + contact.email

            CREATE_TASK "Follow up on quote " + quote.quote_number, "medium", user_id
        END IF
    END IF

END IF

IF action = "forecast" THEN
    opportunities = FIND "opportunities", "stage != 'closed_won' AND stage != 'closed_lost'"

    total_pipeline = 0
    weighted_pipeline = 0

    q1_forecast = 0
    q2_forecast = 0
    q3_forecast = 0
    q4_forecast = 0

    FOR EACH opp IN opportunities DO
        total_pipeline = total_pipeline + opp.amount
        weighted_value = opp.amount * opp.probability / 100
        weighted_pipeline = weighted_pipeline + weighted_value

        close_month = FORMAT opp.close_date AS "MM"

        IF close_month <= "03" THEN
            q1_forecast = q1_forecast + weighted_value
        ELSE IF close_month <= "06" THEN
            q2_forecast = q2_forecast + weighted_value
        ELSE IF close_month <= "09" THEN
            q3_forecast = q3_forecast + weighted_value
        ELSE
            q4_forecast = q4_forecast + weighted_value
        END IF
    END FOR

    TALK "SALES FORECAST"
    TALK "=============="
    TALK "Total Pipeline: $" + total_pipeline
    TALK "Weighted Pipeline: $" + weighted_pipeline
    TALK ""
    TALK "Quarterly Forecast:"
    TALK "Q1: $" + q1_forecast
    TALK "Q2: $" + q2_forecast
    TALK "Q3: $" + q3_forecast
    TALK "Q4: $" + q4_forecast

    forecast_report = CREATE OBJECT
    SET forecast_report.total_pipeline = total_pipeline
    SET forecast_report.weighted_pipeline = weighted_pipeline
    SET forecast_report.q1 = q1_forecast
    SET forecast_report.q2 = q2_forecast
    SET forecast_report.q3 = q3_forecast
    SET forecast_report.q4 = q4_forecast
    SET forecast_report.generated_at = current_time

    REMEMBER "forecast_" + FORMAT NOW() AS "YYYYMMDD" = forecast_report

END IF
