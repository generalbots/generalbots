# Bank Template

The bank template provides a complete digital banking assistant for financial institutions, enabling customers to manage accounts, transfers, payments, cards, and investments through conversational AI.

## Topic: Digital Banking Assistant

This template is perfect for:
- Retail banking customer service
- Account management automation
- Payment and transfer processing
- Card services and support
- Investment inquiries

## The Code

```basic
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
```

## Sample Dialogs

These conversations show how the bank template works in real-world scenarios.

### Dialog 1: Check Balance

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🏦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bank</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-date"><span>Today</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">General Bank</div>
      <p><strong>General Bank</strong> - Digital Banking Assistant</p>
      <p>Welcome! I can help you with:</p>
      <p>• Account balance and statements<br>• Transfers and PIX<br>• Bill payments<br>• Card services</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>Check my balance</p>
      <div class="wa-time">09:15 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📊 <strong>Account Summary</strong></p>
      <p>Account: ****4521</p>
      <p>💵 <strong>Available Balance:</strong> $5,432.18</p>
      <p>📈 <strong>Savings:</strong> $12,850.00</p>
      <p>💳 <strong>Credit Available:</strong> $8,500.00</p>
      <div class="wa-time">09:15</div>
    </div>
  </div>
</div>

### Dialog 2: PIX Transfer

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🏦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bank</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I want to send a PIX</p>
      <div class="wa-time">14:20 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>💸 <strong>PIX Transfer</strong></p>
      <p>Please provide the PIX key (CPF, phone, email, or random key):</p>
      <div class="wa-time">14:20</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>maria@email.com</p>
      <div class="wa-time">14:21 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ Key found: <strong>Maria Santos</strong></p>
      <p>How much would you like to send?</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>R$ 150</p>
      <div class="wa-time">14:21 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>📋 <strong>Please confirm:</strong></p>
      <p>To: Maria Santos<br>Key: maria@email.com<br>Amount: R$ 150.00</p>
      <p>Type <strong>CONFIRM</strong> to proceed.</p>
      <div class="wa-time">14:21</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>CONFIRM</p>
      <div class="wa-time">14:22 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>✅ <strong>PIX Sent Successfully!</strong></p>
      <p>Amount: R$ 150.00<br>To: Maria Santos<br>Transaction ID: PIX2024031412220001</p>
      <p>Receipt sent to your email.</p>
      <div class="wa-time">14:22</div>
    </div>
  </div>
</div>

### Dialog 3: Block Lost Card

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🏦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bank</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>I lost my card! Block it now!</p>
      <div class="wa-time">22:45 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚨 <strong>Emergency Card Block</strong></p>
      <p>I'm blocking your card immediately for your security.</p>
      <p>Which card do you want to block?</p>
      <p>1️⃣ Debit Card ****4521<br>2️⃣ Credit Card ****7832</p>
      <div class="wa-time">22:45</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>1</p>
      <div class="wa-time">22:45 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🔒 <strong>Card Blocked</strong></p>
      <p>Your Debit Card ****4521 has been blocked.</p>
      <p>📦 A replacement card will be sent to your registered address within 5-7 business days.</p>
      <p>If you find your card, you can unblock it through this assistant or call 0800-123-4567.</p>
      <div class="wa-time">22:45</div>
    </div>
  </div>
</div>

### Dialog 4: Fraud Detection Escalation

<div class="wa-chat">
  <div class="wa-header">
    <div class="wa-header-avatar">🏦</div>
    <div class="wa-header-info">
      <div class="wa-header-name">General Bank</div>
      <div class="wa-header-status">online</div>
    </div>
  </div>

  <div class="wa-message user">
    <div class="wa-bubble">
      <p>There's a suspicious transaction on my account I didn't make</p>
      <div class="wa-time">16:30 <span class="wa-status read"></span></div>
    </div>
  </div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <p>🚨 <strong>Fraud Alert - Priority Response</strong></p>
      <p>I'm connecting you with our Fraud Prevention team immediately.</p>
      <p>For your protection, I've temporarily limited transactions on your account.</p>
      <div class="wa-time">16:30</div>
    </div>
  </div>

  <div class="wa-system"><span>Fraud Detector Bot joined</span></div>

  <div class="wa-message bot">
    <div class="wa-bubble">
      <div class="wa-sender">Fraud Detector</div>
      <p>Hello, I'm from the Fraud Prevention team. I can see your recent transactions.</p>
      <p>Which transaction looks suspicious? Please provide the approximate date and amount.</p>
      <div class="wa-time">16:30</div>
    </div>
  </div>
</div>

## Keywords Used

| Keyword | Purpose |
|---------|---------|
| `ADD TOOL` | Register banking operation tools |
| `ADD BOT` | Register specialized bots with triggers |
| `USE KB` | Load banking FAQ knowledge base |
| `ADD SUGGESTION` | Create quick action buttons |
| `SET CONTEXT` | Define bot behavior and security rules |
| `BEGIN TALK` | Welcome message block |
| `BEGIN SYSTEM PROMPT` | Security instructions for AI |

## Multi-Bot Architecture

The bank template uses a multi-bot architecture for specialized handling:

| Bot | Trigger Words | Purpose |
|-----|---------------|---------|
| `fraud-detector` | suspicious, fraud, unauthorized, stolen, hack | Handle security concerns |
| `investment-advisor` | invest, stocks, funds, portfolio, CDB, LCI | Investment guidance |
| `loan-specialist` | loan, financing, credit, mortgage | Loan inquiries |
| `card-services` | card, credit card, debit card, block, limit | Card management |

## Security Features

### Built-in Protections

1. **Data Masking**: Account and card numbers are always masked
2. **Transaction Limits**: Confirmation required for large transactions
3. **Fraud Escalation**: Automatic routing to fraud team for suspicious activity
4. **Audit Logging**: All sensitive operations are logged
5. **No Sensitive Data**: Never asks for passwords or full card numbers

### Implementing Security Checks

```basic
' Example: Verify identity before sensitive operation
PARAM operation AS STRING

IF operation = "transfer" AND amount > 1000 THEN
    TALK "For your security, please confirm your identity."
    TALK "Enter the last 4 digits of your CPF:"
    HEAR verification
    
    IF NOT VERIFY_IDENTITY(verification) THEN
        TALK "Verification failed. Please try again or call support."
        RETURN
    END IF
END IF
```

## Customization Ideas

### Add Investment Products

```basic
ADD TOOL "simulate-investment"
ADD TOOL "compare-products"

' In investment flow
products = FIND "investment_products.csv", "risk_level = 'low'"
TALK "Here are our low-risk investment options:"
FOR EACH product IN products
    TALK "• " + product.name + " - " + product.rate + "% p.a."
NEXT
```

### Add Bill Payment with Barcode

```basic
PARAM barcode AS STRING DESCRIPTION "Bill barcode or PIX copy-paste code"

IF LEN(barcode) = 47 THEN
    ' Boleto bancário
    bill = PARSE_BOLETO(barcode)
    TALK "Bill Details:"
    TALK "Payee: " + bill.payee
    TALK "Amount: R$ " + FORMAT(bill.amount, "#,##0.00")
    TALK "Due Date: " + FORMAT(bill.due_date, "DD/MM/YYYY")
ELSE IF INSTR(barcode, "pix") > 0 THEN
    ' PIX QR Code
    pix = PARSE_PIX(barcode)
    TALK "PIX Payment: R$ " + FORMAT(pix.amount, "#,##0.00")
END IF
```

### Add Account Statements

```basic
PARAM period AS STRING LIKE "last 30 days" DESCRIPTION "Statement period"

transactions = FIND "transactions.csv", "account_id = '" + account_id + "' AND date >= '" + start_date + "'"

TALK "📋 **Account Statement**"
TALK "Period: " + period
TALK ""

balance = 0
FOR EACH tx IN transactions
    IF tx.type = "credit" THEN
        balance = balance + tx.amount
        TALK "➕ " + tx.description + ": R$ " + FORMAT(tx.amount, "#,##0.00")
    ELSE
        balance = balance - tx.amount
        TALK "➖ " + tx.description + ": R$ " + FORMAT(tx.amount, "#,##0.00")
    END IF
NEXT

TALK ""
TALK "**Final Balance:** R$ " + FORMAT(balance, "#,##0.00")
```

## Related Templates

- [store.bas](./store.md) - E-commerce with payment integration
- [privacy.bas](./privacy.md) - Data protection compliance
- [auth.bas](./auth.md) - Authentication patterns

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
.wa-system{text-align:center;margin:15px 0;clear:both}
.wa-system span{background-color:#e1f2fb;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px}
.wa-date{text-align:center;margin:15px 0;clear:both}
.wa-date span{background-color:#fff;color:#54656f;padding:5px 12px;border-radius:8px;font-size:12px;box-shadow:0 1px .5px rgba(0,0,0,.13)}
.wa-header{background-color:#075e54;color:#fff;padding:10px 15px;margin:-20px -15px 15px -15px;border-radius:8px 8px 0 0;display:flex;align-items:center;gap:10px}
.wa-header-avatar{width:40px;height:40px;background-color:#25d366;border-radius:50%;display:flex;align-items:center;justify-content:center;font-size:18px}
.wa-header-info{flex:1}
.wa-header-name{font-weight:600;font-size:16px}
.wa-header-status{font-size:12px;opacity:.8}
</style>