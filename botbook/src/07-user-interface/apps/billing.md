# Billing - Invoices, Payments & Quotes

> **Manage your financial transactions from quote to payment**

---

## Overview

Billing is your complete invoicing solution following Microsoft Dynamics nomenclature. Create quotes for opportunities, convert them to invoices, and track payments through to completion.

---

## Key Concepts (Dynamics Nomenclature)

| Entity | Description |
|--------|-------------|
| **Quote** | Price quotation sent to prospect/customer |
| **Invoice** | Bill sent to customer for payment |
| **Payment** | Payment received against an invoice |

### Entity Flow

```
Quote ──(accept)──► Invoice ──(pay)──► Payment
```

---

## Features

### Invoices Management

Track all your billing documents:

- **Invoice Number** - Unique identifier
- **Account** - Customer being billed
- **Date** - Invoice creation date
- **Due Date** - Payment deadline
- **Amount** - Total invoice value
- **Status** - Draft, Sent, Paid, Overdue, Cancelled

### Invoice Statuses

| Status | Description |
|--------|-------------|
| **Draft** | Invoice being prepared, not yet sent |
| **Sent** | Invoice delivered to customer |
| **Paid** | Payment received in full |
| **Overdue** | Past due date, unpaid |
| **Cancelled** | Invoice voided |

### Payments

Record and track incoming payments:

- **Payment ID** - Unique identifier
- **Invoice** - Associated invoice
- **Account** - Paying customer
- **Date** - Payment received date
- **Amount** - Payment amount
- **Method** - Payment method used

### Payment Methods

| Method | Description |
|--------|-------------|
| **Bank Transfer** | Wire/ACH transfer |
| **Credit Card** | Card payment |
| **PIX** | Brazilian instant payment |
| **Boleto** | Brazilian bank slip |
| **Cash** | Cash payment |

### Quotes

Create proposals for potential deals:

- **Quote Number** - Unique identifier
- **Account** - Customer receiving quote
- **Opportunity** - Associated sales opportunity
- **Date** - Quote creation date
- **Valid Until** - Expiration date
- **Amount** - Total quoted value
- **Status** - Draft, Sent, Accepted, Rejected, Expired

---

## Summary Dashboard

Real-time financial metrics:

| Metric | Description |
|--------|-------------|
| **Pending** | Total value of unpaid invoices |
| **Overdue** | Total value past due date |
| **Paid This Month** | Payments received this month |
| **Revenue This Month** | Total revenue for current month |

---

## Navigation Tabs

| Tab | View |
|-----|------|
| **Invoices** | All invoice records |
| **Payments** | Payment history |
| **Quotes** | Price quotations |

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/billing/invoices` | GET | List invoices with filters |
| `/api/billing/invoices` | POST | Create new invoice |
| `/api/billing/invoices/:id` | GET | Get invoice details |
| `/api/billing/invoices/:id` | PUT | Update invoice |
| `/api/billing/invoices/:id/send` | POST | Mark invoice as sent |
| `/api/billing/invoices/:id/cancel` | POST | Cancel invoice |
| `/api/billing/invoices/export` | GET | Export invoices |
| `/api/billing/payments` | GET | List payments |
| `/api/billing/payments` | POST | Record new payment |
| `/api/billing/quotes` | GET | List quotes |
| `/api/billing/quotes` | POST | Create new quote |
| `/api/billing/quotes/:id/accept` | POST | Convert quote to invoice |
| `/api/billing/search` | GET | Search billing records |
| `/api/billing/stats/*` | GET | Get billing statistics |

---

## @ Mentions in Chat

Reference billing entities directly in chat:

| Mention | Example |
|---------|---------|
| `@invoice:` | @invoice:INV-2024-001 |

Hover over a mention to see invoice details. Click to navigate to the record.

---

## Filtering Options

### Invoice Filters

| Filter | Options |
|--------|---------|
| **Status** | All, Draft, Sent, Paid, Overdue, Cancelled |
| **Period** | All Time, This Month, This Quarter, This Year |

### Payment Filters

| Filter | Options |
|--------|---------|
| **Method** | All Methods, Bank Transfer, Credit Card, PIX, Boleto, Cash |

### Quote Filters

| Filter | Options |
|--------|---------|
| **Status** | All, Draft, Sent, Accepted, Rejected, Expired |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `N` | New invoice (when in Billing) |
| `Escape` | Close modal |
| `/` | Focus search |

---

## Integration with CRM

Billing integrates seamlessly with CRM:

1. **Quote from Opportunity** - Create quotes linked to opportunities
2. **Convert on Win** - When opportunity is won, convert quote to invoice
3. **Account Linking** - Invoices automatically linked to customer accounts

---

## Best Practices

### Invoice Management

1. **Send promptly** - Issue invoices immediately after delivery
2. **Set clear terms** - Include payment terms and due dates
3. **Follow up** - Track overdue invoices proactively

### Payment Tracking

1. **Record immediately** - Log payments as soon as received
2. **Match correctly** - Ensure payments match the right invoices
3. **Reconcile regularly** - Review payment records weekly

### Quote Management

1. **Include details** - List all line items with descriptions
2. **Set expiration** - Use reasonable validity periods
3. **Follow up** - Check on pending quotes before expiration

---

## Reports

Available in Analytics:

| Report | Description |
|--------|-------------|
| **Revenue Summary** | Total revenue over time |
| **Aging Report** | Overdue invoices by age |
| **Payment History** | Payments received over time |
| **Monthly Revenue** | Month-over-month comparison |

---

## See Also

- [CRM](./crm.md) — Link invoices to accounts and opportunities
- [Products](./products.md) — Add products to invoices and quotes
- [Analytics](./analytics.md) — Billing reports and dashboards