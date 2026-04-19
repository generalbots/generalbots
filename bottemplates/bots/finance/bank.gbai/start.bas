ADD TOOL "check-balance"
ADD TOOL "transfer-money"
ADD TOOL "pay-bill"
ADD TOOL "card-services"
ADD TOOL "loan-inquiry"
ADD TOOL "investment-info"
ADD TOOL "transaction-history"
ADD TOOL "open-account"

ADD BOT "fraud-detector" WITH TRIGGER "suspicious, fraud, unauthorized, stolen, hack"
ADD BOT "investment-advisor" WITH TRIGGER "invest, stocks, funds, portfolio, returns, CDB, LCI"
ADD BOT "loan-specialist" WITH TRIGGER "loan, financing, credit, mortgage, empréstimo"
ADD BOT "card-services" WITH TRIGGER "card, credit card, debit card, block card, limit"

USE KB "banking-faq"

CLEAR SUGGESTIONS

ADD SUGGESTION "balance" AS "Check my balance"
ADD SUGGESTION "transfer" AS "Make a transfer"
ADD SUGGESTION "pix" AS "Send PIX"
ADD SUGGESTION "bills" AS "Pay a bill"
ADD SUGGESTION "card" AS "Card services"
ADD SUGGESTION "history" AS "Transaction history"
ADD SUGGESTION "invest" AS "Investment options"
ADD SUGGESTION "loan" AS "Loan information"

SET CONTEXT "You are a professional banking assistant for General Bank. Help customers with accounts, transfers, payments, cards, loans, and investments. Always verify identity before sensitive operations. Be helpful and secure. Never ask for full card numbers or passwords in chat."

BEGIN TALK
**General Bank** - Digital Banking Assistant

Welcome! I can help you with:
• Account balance and statements
• Transfers and PIX
• Bill payments
• Card services
• Investments
• Loans and financing

Select an option below or tell me what you need.
END TALK

BEGIN SYSTEM PROMPT
You are a secure banking assistant.

Security rules:
- Never display full account numbers
- Mask card numbers showing only last 4 digits
- Require confirmation for transactions over $1000
- Log all sensitive operations
- Escalate fraud concerns immediately
END SYSTEM PROMPT
