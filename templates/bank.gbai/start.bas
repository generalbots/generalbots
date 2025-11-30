' General Bots Conversational Banking
' Enterprise-grade banking through natural conversation
' Uses TOOLS (not SUBs) and HEAR AS validation

' ============================================================================
' CONFIGURATION
' ============================================================================

SET CONTEXT "You are a professional banking assistant for General Bank.
Help customers with accounts, transfers, payments, cards, loans, and investments.
Always verify identity before sensitive operations. Be helpful and secure.
Use the available tools to perform banking operations.
Never ask for full card numbers or passwords in chat."

USE KB "banking-faq"

' Add specialized bots for complex operations
ADD BOT "fraud-detector" WITH TRIGGER "suspicious, fraud, unauthorized, stolen, hack"
ADD BOT "investment-advisor" WITH TRIGGER "invest, stocks, funds, portfolio, returns, CDB, LCI"
ADD BOT "loan-specialist" WITH TRIGGER "loan, financing, credit, mortgage, empréstimo"
ADD BOT "card-services" WITH TRIGGER "card, limit, block, virtual card, cartão"

' ============================================================================
' BANKING TOOLS - Dynamic tools added to conversation
' ============================================================================

' Account Tools
USE TOOL "check_balance"
USE TOOL "get_statement"
USE TOOL "get_transactions"

' Transfer Tools
USE TOOL "pix_transfer"
USE TOOL "ted_transfer"
USE TOOL "schedule_transfer"

' Payment Tools
USE TOOL "pay_boleto"
USE TOOL "pay_utility"
USE TOOL "list_scheduled_payments"

' Card Tools
USE TOOL "list_cards"
USE TOOL "block_card"
USE TOOL "unblock_card"
USE TOOL "create_virtual_card"
USE TOOL "request_limit_increase"

' Loan Tools
USE TOOL "simulate_loan"
USE TOOL "apply_loan"
USE TOOL "list_loans"

' Investment Tools
USE TOOL "get_portfolio"
USE TOOL "list_investments"
USE TOOL "buy_investment"
USE TOOL "redeem_investment"

' ============================================================================
' AUTHENTICATION FLOW
' ============================================================================

authenticated = GET user_authenticated

IF NOT authenticated THEN
    TALK "Welcome to General Bank! 🏦"
    TALK "For your security, I need to verify your identity."
    TALK ""
    TALK "Please enter your CPF:"

    HEAR cpf AS CPF

    ' Look up customer
    customer = FIND "customers.csv" WHERE cpf = cpf

    IF LEN(customer) = 0 THEN
        TALK "I couldn't find an account with this CPF."
        TALK "Please check the number or visit a branch to open an account."
    ELSE
        ' Send verification code
        phone_masked = MID(FIRST(customer).phone, 1, 4) + "****" + RIGHT(FIRST(customer).phone, 2)
        TALK "I'll send a verification code to your phone ending in " + phone_masked

        ' Generate and store code
        code = STR(INT(RND() * 900000) + 100000)
        SET BOT MEMORY "verification_code", code
        SET BOT MEMORY "verification_cpf", cpf

        ' In production: SEND SMS FIRST(customer).phone, "Your General Bank code is: " + code

        TALK "Please enter the 6-digit code:"
        HEAR entered_code AS INTEGER

        stored_code = GET BOT MEMORY "verification_code"

        IF STR(entered_code) = stored_code THEN
            SET user_authenticated, TRUE
            SET user_id, FIRST(customer).id
            SET user_name, FIRST(customer).name
            SET user_cpf, cpf

            TALK "✅ Welcome, " + FIRST(customer).name + "!"
        ELSE
            TALK "❌ Invalid code. Please try again."
        END IF
    END IF
END IF

' ============================================================================
' MAIN CONVERSATION - LLM handles intent naturally
' ============================================================================

IF GET user_authenticated THEN
    user_name = GET user_name

    TALK ""
    TALK "How can I help you today, " + user_name + "?"
    TALK ""
    TALK "You can ask me things like:"
    TALK "• What's my balance?"
    TALK "• Send R$ 100 via PIX to 11999998888"
    TALK "• Pay this boleto: 23793.38128..."
    TALK "• Block my credit card"
    TALK "• Simulate a loan of R$ 10,000"

    ADD SUGGESTION "Check balance"
    ADD SUGGESTION "Make a transfer"
    ADD SUGGESTION "Pay a bill"
    ADD SUGGESTION "My cards"
END IF

' ============================================================================
' TOOL: check_balance
' Returns account balances for the authenticated user
' ============================================================================

' @tool check_balance
' @description Get account balances for the current user
' @param account_type string optional Filter by account type (checking, savings, all)
' @returns Account balances with available amounts

' ============================================================================
' TOOL: pix_transfer
' Performs a PIX transfer
' ============================================================================

' @tool pix_transfer
' @description Send money via PIX instant transfer
' @param pix_key string required The recipient's PIX key (CPF, phone, email, or random key)
' @param amount number required Amount to transfer in BRL
' @param description string optional Transfer description
' @returns Transfer confirmation with transaction ID

ON TOOL "pix_transfer"
    pix_key = GET TOOL PARAM "pix_key"
    amount = GET TOOL PARAM "amount"
    description = GET TOOL PARAM "description"

    ' Validate PIX key format
    TALK "🔍 Validating PIX key..."

    ' Get recipient info (simulated API call)
    recipient_name = LLM "Given PIX key " + pix_key + ", return a realistic Brazilian name. Just the name, nothing else."
    recipient_bank = "Banco Example"

    TALK ""
    TALK "📤 **Transfer Details**"
    TALK "To: **" + recipient_name + "**"
    TALK "Bank: " + recipient_bank
    TALK "Amount: **R$ " + FORMAT(amount, "#,##0.00") + "**"
    TALK ""
    TALK "Confirm this PIX transfer?"

    ADD SUGGESTION "Yes, confirm"
    ADD SUGGESTION "No, cancel"

    HEAR confirmation AS BOOLEAN

    IF confirmation THEN
        TALK "🔐 Enter your 4-digit PIN:"
        HEAR pin AS INTEGER

        ' Validate PIN (in production, verify against stored hash)
        IF LEN(STR(pin)) = 4 THEN
            ' Execute transfer
            transaction_id = "PIX" + FORMAT(NOW(), "yyyyMMddHHmmss") + STR(INT(RND() * 1000))

            ' Get current balance
            user_id = GET user_id
            account = FIRST(FIND "accounts.csv" WHERE user_id = user_id)
            new_balance = account.balance - amount

            ' Save transaction
            TABLE transaction
                ROW transaction_id, account.account_number, "pix_out", -amount, new_balance, NOW(), pix_key, recipient_name, "completed"
            END TABLE
            SAVE "transactions.csv", transaction

            ' Update balance
            UPDATE "accounts.csv" SET balance = new_balance WHERE id = account.id

            TALK ""
            TALK "✅ **PIX Transfer Completed!**"
            TALK ""
            TALK "Transaction ID: " + transaction_id
            TALK "Amount: R$ " + FORMAT(amount, "#,##0.00")
            TALK "New Balance: R$ " + FORMAT(new_balance, "#,##0.00")
            TALK "Date: " + FORMAT(NOW(), "dd/MM/yyyy HH:mm")

            RETURN transaction_id
        ELSE
            TALK "❌ Invalid PIN format."
            RETURN "CANCELLED"
        END IF
    ELSE
        TALK "Transfer cancelled."
        RETURN "CANCELLED"
    END IF
END ON

' ============================================================================
' TOOL: pay_boleto
' Pays a Brazilian bank slip (boleto)
' ============================================================================

' @tool pay_boleto
' @description Pay a boleto (bank slip) using the barcode
' @param barcode string required The boleto barcode (47 or 48 digits)
' @returns Payment confirmation

ON TOOL "pay_boleto"
    barcode = GET TOOL PARAM "barcode"

    ' Clean barcode
    barcode = REPLACE(REPLACE(REPLACE(barcode, ".", ""), " ", ""), "-", "")

    IF LEN(barcode) <> 47 AND LEN(barcode) <> 48 THEN
        TALK "❌ Invalid barcode. Please enter all 47 or 48 digits."
        RETURN "INVALID_BARCODE"
    END IF

    ' Parse boleto (simplified - in production use banking API)
    beneficiary = "Company " + LEFT(barcode, 3)
    amount = VAL(MID(barcode, 38, 10)) / 100
    due_date = DATEADD(NOW(), INT(RND() * 30), "day")

    TALK ""
    TALK "📄 **Bill Details**"
    TALK "Beneficiary: **" + beneficiary + "**"
    TALK "Amount: **R$ " + FORMAT(amount, "#,##0.00") + "**"
    TALK "Due Date: " + FORMAT(due_date, "dd/MM/yyyy")
    TALK ""
    TALK "Pay this bill now?"

    ADD SUGGESTION "Yes, pay now"
    ADD SUGGESTION "Schedule for due date"
    ADD SUGGESTION "Cancel"

    HEAR choice AS "Pay now", "Schedule", "Cancel"

    IF choice = "Pay now" THEN
        TALK "🔐 Enter your PIN:"
        HEAR pin AS INTEGER

        IF LEN(STR(pin)) = 4 THEN
            transaction_id = "BOL" + FORMAT(NOW(), "yyyyMMddHHmmss")
            auth_code = FORMAT(INT(RND() * 100000000), "00000000")

            TALK ""
            TALK "✅ **Payment Completed!**"
            TALK ""
            TALK "Transaction ID: " + transaction_id
            TALK "Authentication: " + auth_code
            TALK "Amount: R$ " + FORMAT(amount, "#,##0.00")

            RETURN transaction_id
        ELSE
            TALK "❌ Invalid PIN."
            RETURN "INVALID_PIN"
        END IF

    ELSEIF choice = "Schedule" THEN
        TABLE scheduled
            ROW NOW(), GET user_id, "boleto", barcode, amount, due_date, "pending"
        END TABLE
        SAVE "scheduled_payments.csv", scheduled

        TALK "✅ Payment scheduled for " + FORMAT(due_date, "dd/MM/yyyy")
        RETURN "SCHEDULED"
    ELSE
        TALK "Payment cancelled."
        RETURN "CANCELLED"
    END IF
END ON

' ============================================================================
' TOOL: block_card
' Blocks a card for security
' ============================================================================

' @tool block_card
' @description Block a credit or debit card
' @param card_type string optional Type of card to block (credit, debit, all)
' @param reason string optional Reason for blocking (lost, stolen, suspicious, temporary)
' @returns Block confirmation

ON TOOL "block_card"
    card_type = GET TOOL PARAM "card_type"
    reason = GET TOOL PARAM "reason"

    user_id = GET user_id
    cards = FIND "cards.csv" WHERE user_id = user_id AND status = "active"

    IF LEN(cards) = 0 THEN
        TALK "You don't have any active cards to block."
        RETURN "NO_CARDS"
    END IF

    IF card_type = "" OR card_type = "all" THEN
        TALK "Which card do you want to block?"

        FOR i = 1 TO LEN(cards)
            card = cards[i]
            masked = "**** " + RIGHT(card.card_number, 4)
            TALK STR(i) + ". " + UPPER(card.card_type) + " - " + masked
            ADD SUGGESTION card.card_type + " " + RIGHT(card.card_number, 4)
        NEXT

        HEAR selection AS INTEGER

        IF selection < 1 OR selection > LEN(cards) THEN
            TALK "Invalid selection."
            RETURN "INVALID_SELECTION"
        END IF

        selected_card = cards[selection]
    ELSE
        selected_card = FIRST(FILTER cards WHERE card_type = card_type)
    END IF

    IF reason = "" THEN
        TALK "Why are you blocking this card?"
        ADD SUGGESTION "Lost"
        ADD SUGGESTION "Stolen"
        ADD SUGGESTION "Suspicious activity"
        ADD SUGGESTION "Temporary block"

        HEAR reason AS "Lost", "Stolen", "Suspicious activity", "Temporary block"
    END IF

    ' Block the card
    UPDATE "cards.csv" SET status = "blocked", blocked_reason = reason, blocked_at = NOW() WHERE id = selected_card.id

    masked = "**** " + RIGHT(selected_card.card_number, 4)

    TALK ""
    TALK "🔒 **Card Blocked**"
    TALK ""
    TALK "Card: " + UPPER(selected_card.card_type) + " " + masked
    TALK "Reason: " + reason
    TALK "Blocked at: " + FORMAT(NOW(), "dd/MM/yyyy HH:mm")

    IF reason = "Stolen" OR reason = "Lost" THEN
        TALK ""
        TALK "⚠️ For your security, we recommend requesting a replacement card."
        TALK "Would you like me to request a new card?"

        ADD SUGGESTION "Yes, request new card"
        ADD SUGGESTION "No, not now"

        HEAR request_new AS BOOLEAN

        IF request_new THEN
            TALK "✅ New card requested! It will arrive in 5-7 business days."
        END IF
    END IF

    RETURN "BLOCKED"
END ON

' ============================================================================
' TOOL: simulate_loan
' Simulates loan options
' ============================================================================

' @tool simulate_loan
' @description Simulate a personal loan with different terms
' @param amount number required Loan amount in BRL
' @param months integer optional Number of months (12, 24, 36, 48, 60)
' @param loan_type string optional Type of loan (personal, payroll, home_equity)
' @returns Loan simulation with monthly payments

ON TOOL "simulate_loan"
    amount = GET TOOL PARAM "amount"
    months = GET TOOL PARAM "months"
    loan_type = GET TOOL PARAM "loan_type"

    IF amount < 500 THEN
        TALK "Minimum loan amount is R$ 500.00"
        RETURN "AMOUNT_TOO_LOW"
    END IF

    IF amount > 100000 THEN
        TALK "For amounts above R$ 100,000, please visit a branch."
        RETURN "AMOUNT_TOO_HIGH"
    END IF

    IF months = 0 THEN
        TALK "In how many months would you like to pay?"
        ADD SUGGESTION "12 months"
        ADD SUGGESTION "24 months"
        ADD SUGGESTION "36 months"
        ADD SUGGESTION "48 months"
        ADD SUGGESTION "60 months"

        HEAR months_input AS INTEGER
        months = months_input
    END IF

    IF loan_type = "" THEN
        loan_type = "personal"
    END IF

    ' Calculate rates based on type
    IF loan_type = "payroll" THEN
        monthly_rate = 0.0149
        rate_label = "1.49%"
    ELSEIF loan_type = "home_equity" THEN
        monthly_rate = 0.0099
        rate_label = "0.99%"
    ELSE
        monthly_rate = 0.0199
        rate_label = "1.99%"
    END IF

    ' PMT calculation
    pmt = amount * (monthly_rate * POWER(1 + monthly_rate, months)) / (POWER(1 + monthly_rate, months) - 1)
    total = pmt * months
    interest_total = total - amount

    TALK ""
    TALK "💰 **Loan Simulation**"
    TALK ""
    TALK "📊 **" + UPPER(loan_type) + " LOAN**"
    TALK ""
    TALK "Amount: R$ " + FORMAT(amount, "#,##0.00")
    TALK "Term: " + STR(months) + " months"
    TALK "Interest Rate: " + rate_label + " per month"
    TALK ""
    TALK "📅 **Monthly Payment: R$ " + FORMAT(pmt, "#,##0.00") + "**"
    TALK ""
    TALK "Total to pay: R$ " + FORMAT(total, "#,##0.00")
    TALK "Total interest: R$ " + FORMAT(interest_total, "#,##0.00")
    TALK ""
    TALK "Would you like to apply for this loan?"

    ADD SUGGESTION "Yes, apply now"
    ADD SUGGESTION "Try different values"
    ADD SUGGESTION "Not now"

    HEAR decision AS "Apply", "Try again", "No"

    IF decision = "Apply" THEN
        TALK "Great! Let me collect some additional information."

        TALK "What is your monthly income?"
        HEAR income AS MONEY

        TALK "What is your profession?"
        HEAR profession AS NAME

        ' Check debt-to-income ratio
        IF pmt > income * 0.35 THEN
            TALK "⚠️ The monthly payment exceeds 35% of your income."
            TALK "We recommend a smaller amount or longer term."
            RETURN "HIGH_DTI"
        END IF

        application_id = "LOAN" + FORMAT(NOW(), "yyyyMMddHHmmss")

        TABLE loan_application
            ROW application_id, GET user_id, loan_type, amount, months, monthly_rate, income, profession, NOW(), "pending"
        END TABLE
        SAVE "loan_applications.csv", loan_application

        TALK ""
        TALK "🎉 **Application Submitted!**"
        TALK ""
        TALK "Application ID: " + application_id
        TALK "Status: Under Analysis"
        TALK ""
        TALK "We'll analyze your application within 24 hours."
        TALK "You'll receive updates via app notifications."

        RETURN application_id

    ELSEIF decision = "Try again" THEN
        TALK "No problem! What values would you like to try?"
        RETURN "RETRY"
    ELSE
        TALK "No problem! I'm here whenever you need."
        RETURN "DECLINED"
    END IF
END ON

' ============================================================================
' TOOL: create_virtual_card
' Creates a virtual card for online purchases
' ============================================================================

' @tool create_virtual_card
' @description Create a virtual credit card for online shopping
' @param limit number optional Maximum limit for the virtual card
' @returns Virtual card details

ON TOOL "create_virtual_card"
    limit = GET TOOL PARAM "limit"

    user_id = GET user_id
    credit_cards = FIND "cards.csv" WHERE user_id = user_id AND card_type = "credit" AND status = "active"

    IF LEN(credit_cards) = 0 THEN
        TALK "You need an active credit card to create virtual cards."
        RETURN "NO_CREDIT_CARD"
    END IF

    main_card = FIRST(credit_cards)

    IF limit = 0 THEN
        TALK "What limit would you like for this virtual card?"
        TALK "Available credit: R$ " + FORMAT(main_card.available_limit, "#,##0.00")

        ADD SUGGESTION "R$ 100"
        ADD SUGGESTION "R$ 500"
        ADD SUGGESTION "R$ 1000"
        ADD SUGGESTION "Custom amount"

        HEAR limit AS MONEY
    END IF

    IF limit > main_card.available_limit THEN
        TALK "❌ Limit exceeds available credit."
        TALK "Maximum available: R$ " + FORMAT(main_card.available_limit, "#,##0.00")
        RETURN "LIMIT_EXCEEDED"
    END IF

    ' Generate virtual card
    virtual_number = "4" + FORMAT(INT(RND() * 1000000000000000), "000000000000000")
    virtual_cvv = FORMAT(INT(RND() * 1000), "000")
    virtual_expiry = FORMAT(DATEADD(NOW(), 1, "year"), "MM/yy")

    virtual_id = "VC" + FORMAT(NOW(), "yyyyMMddHHmmss")

    TABLE virtual_card
        ROW virtual_id, user_id, main_card.id, "virtual", virtual_number, virtual_cvv, virtual_expiry, limit, limit, "active", NOW()
    END TABLE
    SAVE "cards.csv", virtual_card

    ' Format card number for display
    formatted_number = LEFT(virtual_number, 4) + " " + MID(virtual_number, 5, 4) + " " + MID(virtual_number, 9, 4) + " " + RIGHT(virtual_number, 4)

    TALK ""
    TALK "✅ **Virtual Card Created!**"
    TALK ""
    TALK "🔢 Number: " + formatted_number
    TALK "📅 Expiry: " + virtual_expiry
    TALK "🔐 CVV: " + virtual_cvv
    TALK "💰 Limit: R$ " + FORMAT(limit, "#,##0.00")
    TALK ""
    TALK "⚠️ **Save these details now!**"
    TALK "The CVV will not be shown again for security."
    TALK ""
    TALK "This virtual card is linked to your main credit card."
    TALK "You can delete it anytime."

    RETURN virtual_id
END ON

' ============================================================================
' TOOL: get_statement
' Gets account statement
' ============================================================================

' @tool get_statement
' @description Get account statement for a period
' @param period string optional Period: "30days", "90days", "month", or custom dates
' @param format string optional Output format: "chat", "pdf", "email"
' @returns Statement data or download link

ON TOOL "get_statement"
    period = GET TOOL PARAM "period"
    format = GET TOOL PARAM "format"

    user_id = GET user_id
    account = FIRST(FIND "accounts.csv" WHERE user_id = user_id)

    IF period = "" THEN
        TALK "Select the period for your statement:"
        ADD SUGGESTION "Last 30 days"
        ADD SUGGESTION "Last 90 days"
        ADD SUGGESTION "This month"
        ADD SUGGESTION "Custom dates"

        HEAR period_choice AS "30 days", "90 days", "This month", "Custom"

        IF period_choice = "Custom" THEN
            TALK "Enter start date:"
            HEAR start_date AS DATE

            TALK "Enter end date:"
            HEAR end_date AS DATE
        ELSEIF period_choice = "30 days" THEN
            start_date = DATEADD(NOW(), -30, "day")
            end_date = NOW()
        ELSEIF period_choice = "90 days" THEN
            start_date = DATEADD(NOW(), -90, "day")
            end_date = NOW()
        ELSE
            start_date = DATEADD(NOW(), -DAY(NOW()) + 1, "day")
            end_date = NOW()
        END IF
    END IF

    ' Get transactions
    transactions = FIND "transactions.csv" WHERE account_number = account.account_number AND date >= start_date AND date <= end_date ORDER BY date DESC

    IF LEN(transactions) = 0 THEN
        TALK "No transactions found for this period."
        RETURN "NO_TRANSACTIONS"
    END IF

    TALK ""
    TALK "📋 **Account Statement**"
    TALK "Period: " + FORMAT(start_date, "dd/MM/yyyy") + " to " + FORMAT(end_date, "dd/MM/yyyy")
    TALK "Account: " + account.account_number
    TALK ""

    total_in = 0
    total_out = 0

    FOR EACH tx IN transactions
        IF tx.amount > 0 THEN
            icon = "💵 +"
            total_in = total_in + tx.amount
        ELSE
            icon = "💸 "
            total_out = total_out + ABS(tx.amount)
        END IF

        TALK icon + "R$ " + FORMAT(ABS(tx.amount), "#,##0.00") + " | " + FORMAT(tx.date, "dd/MM")
        TALK "   " + tx.description
    NEXT

    TALK ""
    TALK "📊 **Summary**"
    TALK "Total In: R$ " + FORMAT(total_in, "#,##0.00")
    TALK "Total Out: R$ " + FORMAT(total_out, "#,##0.00")
    TALK "Net: R$ " + FORMAT(total_in - total_out, "#,##0.00")

    IF format = "pdf" OR format = "email" THEN
        TALK ""
        TALK "Would you like me to send this statement to your email?"

        ADD SUGGESTION "Yes, send email"
        ADD SUGGESTION "No, thanks"

        HEAR send_email AS BOOLEAN

        IF send_email THEN
            customer = FIRST(FIND "customers.csv" WHERE id = user_id)
            SEND MAIL customer.email, "Your General Bank Statement", "Please find attached your account statement.", "statement.pdf"
            TALK "📧 Statement sent to your email!"
        END IF
    END IF

    RETURN "SUCCESS"
END ON

' ============================================================================
' FALLBACK - Let LLM handle anything not covered by tools
' ============================================================================

' The LLM will use the available tools based on user intent
' No need for rigid menu systems - natural conversation flow
