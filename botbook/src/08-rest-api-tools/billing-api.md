# Billing API

> **Invoicing, payments, quotes, tax rates, and recurring billing management.**

---

## Base URL

```
/api/billing
```

## Authentication

All endpoints require a valid session token via `Authorization: Bearer <token>` header. Invoice voiding and deletion require `admin` or `billing_admin` role.

---

## Endpoints

### Invoices

**`GET /api/billing/invoices`**

Lists invoices with optional filters.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | `draft`, `sent`, `paid`, `overdue`, `void` |
| `customer_id` | string | No | Filter by customer |
| `from` | string | No | ISO 8601 start date |
| `to` | string | No | ISO 8601 end date |
| `page` | integer | No | Page number (default: 1) |
| `limit` | integer | No | Items per page (default: 20) |

**Response:**

```json
{
  "invoices": [
    {
      "id": "inv_001",
      "number": "INV-2025-0042",
      "customer_id": "cust_001",
      "customer_name": "Acme Corp",
      "status": "sent",
      "total": 5400.00,
      "currency": "BRL",
      "due_date": "2025-06-15",
      "issued_at": "2025-06-01T10:00:00Z",
      "paid_at": null
    }
  ],
  "total": 42,
  "page": 1,
  "limit": 20
}
```

---

**`POST /api/billing/invoices`**

Creates a new invoice.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `customer_id` | string | Yes | Customer ID |
| `items` | array | Yes | Line items (see below) |
| `currency` | string | No | Currency code (default: `BRL`) |
| `due_date` | string | Yes | ISO 8601 due date |
| `notes` | string | No | Internal notes |
| `tax_rate_id` | string | No | Applied tax rate |
| `discount_percent` | number | No | Discount percentage |

**Line item object:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `description` | string | Yes | Item description |
| `quantity` | integer | Yes | Quantity |
| `unit_price` | number | Yes | Price per unit |
| `unit` | string | No | Unit of measure |

**Request Body:**

```json
{
  "customer_id": "cust_001",
  "items": [
    {
      "description": "BotServer Pro License (Annual)",
      "quantity": 1,
      "unit_price": 4800.00
    },
    {
      "description": "Custom Integration Setup",
      "quantity": 8,
      "unit_price": 150.00,
      "unit": "hours"
    }
  ],
  "currency": "BRL",
  "due_date": "2025-07-01",
  "tax_rate_id": "tax_001",
  "notes": "Annual renewal + onboarding"
}
```

**Response:**

```json
{
  "id": "inv_002",
  "number": "INV-2025-0043",
  "customer_id": "cust_001",
  "status": "draft",
  "subtotal": 6000.00,
  "tax": 960.00,
  "discount": 0,
  "total": 6960.00,
  "currency": "BRL",
  "due_date": "2025-07-01",
  "items": [
    { "description": "BotServer Pro License (Annual)", "quantity": 1, "unit_price": 4800.00, "total": 4800.00 },
    { "description": "Custom Integration Setup", "quantity": 8, "unit_price": 150.00, "total": 1200.00 }
  ],
  "created_at": "2025-06-04T10:00:00Z"
}
```

---

### Overdue Invoices

**`GET /api/billing/invoices/overdue`**

Returns all invoices past their due date.

**Response:**

```json
{
  "overdue": [
    {
      "id": "inv_003",
      "number": "INV-2025-0038",
      "customer_id": "cust_005",
      "customer_name": "StartupXYZ",
      "total": 2400.00,
      "currency": "BRL",
      "due_date": "2025-05-15",
      "days_overdue": 20,
      "status": "overdue"
    }
  ],
  "count": 3,
  "total_overdue_amount": 8400.00
}
```

---

### Invoice Operations

**`GET /api/billing/invoices/:id`**

Retrieves a single invoice with full details.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Invoice ID |

**Response:**

```json
{
  "id": "inv_002",
  "number": "INV-2025-0043",
  "customer_id": "cust_001",
  "customer_name": "Acme Corp",
  "status": "draft",
  "subtotal": 6000.00,
  "tax": 960.00,
  "tax_rate_name": "ICMS 16%",
  "discount": 0,
  "total": 6960.00,
  "currency": "BRL",
  "due_date": "2025-07-01",
  "issued_at": null,
  "paid_at": null,
  "items": [
    { "description": "BotServer Pro License (Annual)", "quantity": 1, "unit_price": 4800.00, "total": 4800.00 },
    { "description": "Custom Integration Setup", "quantity": 8, "unit_price": 150.00, "total": 1200.00 }
  ],
  "payments": [],
  "created_at": "2025-06-04T10:00:00Z",
  "updated_at": "2025-06-04T10:00:00Z"
}
```

---

**`PUT /api/billing/invoices/:id`**

Updates a draft invoice.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Invoice ID |
| `items` | array | No | Updated line items |
| `due_date` | string | No | Updated due date |
| `notes` | string | No | Updated notes |
| `tax_rate_id` | string | No | Updated tax rate |
| `discount_percent` | number | No | Updated discount |

**Request Body:**

```json
{
  "due_date": "2025-07-15",
  "discount_percent": 5
}
```

**Response:**

```json
{
  "id": "inv_002",
  "status": "draft",
  "subtotal": 6000.00,
  "discount": 300.00,
  "tax": 912.00,
  "total": 6612.00,
  "due_date": "2025-07-15",
  "updated_at": "2025-06-04T11:00:00Z"
}
```

---

**`DELETE /api/billing/invoices/:id`**

Deletes a draft invoice. Sent or paid invoices cannot be deleted — use void instead.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Invoice ID |

**Response:**

```json
{
  "success": true,
  "message": "Draft invoice deleted"
}
```

---

### Send Invoice

**`PUT /api/billing/invoices/:id/send`**

Sends an invoice to the customer via email.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Invoice ID |
| `email` | string | No | Override recipient email |
| `message` | string | No | Custom email message |

**Request Body:**

```json
{
  "email": "billing@acme.com",
  "message": "Olá, segue a fatura referente ao mês de maio."
}
```

**Response:**

```json
{
  "id": "inv_002",
  "status": "sent",
  "sent_to": "billing@acme.com",
  "sent_at": "2025-06-04T12:00:00Z"
}
```

---

### Void Invoice

**`PUT /api/billing/invoices/:id/void`**

Voids a sent invoice (cannot be undone).

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Invoice ID |
| `reason` | string | Yes | Void reason |

**Request Body:**

```json
{
  "reason": "Duplicate invoice created in error"
}
```

**Response:**

```json
{
  "id": "inv_002",
  "status": "void",
  "voided_at": "2025-06-04T13:00:00Z",
  "void_reason": "Duplicate invoice created in error"
}
```

---

### Payments

**`GET /api/billing/payments`**

Lists all payments.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `invoice_id` | string | No | Filter by invoice |
| `status` | string | No | `completed`, `pending`, `failed`, `refunded` |
| `from` | string | No | ISO 8601 start date |
| `to` | string | No | ISO 8601 end date |
| `page` | integer | No | Page number |
| `limit` | integer | No | Items per page |

**Response:**

```json
{
  "payments": [
    {
      "id": "pay_001",
      "invoice_id": "inv_001",
      "amount": 5400.00,
      "currency": "BRL",
      "method": "bank_transfer",
      "status": "completed",
      "reference": "PIX-2025-0601-001",
      "paid_at": "2025-06-05T14:00:00Z"
    }
  ],
  "total": 38,
  "total_amount": 145600.00
}
```

---

**`POST /api/billing/payments`**

Records a payment against an invoice.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `invoice_id` | string | Yes | Invoice ID |
| `amount` | number | Yes | Payment amount |
| `method` | string | Yes | `pix`, `bank_transfer`, `credit_card`, `boleto`, `cash` |
| `reference` | string | No | Transaction reference |
| `paid_at` | string | No | ISO 8601 payment date (default: now) |
| `notes` | string | No | Payment notes |

**Request Body:**

```json
{
  "invoice_id": "inv_001",
  "amount": 5400.00,
  "method": "pix",
  "reference": "PIX-2025-0605-002",
  "paid_at": "2025-06-05T14:30:00Z"
}
```

**Response:**

```json
{
  "id": "pay_002",
  "invoice_id": "inv_001",
  "amount": 5400.00,
  "method": "pix",
  "status": "completed",
  "reference": "PIX-2025-0605-002",
  "paid_at": "2025-06-05T14:30:00Z",
  "invoice_status": "paid"
}
```

---

**`GET /api/billing/payments/:id`**

Retrieves details for a specific payment.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Payment ID |

**Response:**

```json
{
  "id": "pay_001",
  "invoice_id": "inv_001",
  "amount": 5400.00,
  "currency": "BRL",
  "method": "bank_transfer",
  "status": "completed",
  "reference": "PIX-2025-0601-001",
  "paid_at": "2025-06-05T14:00:00Z",
  "customer_id": "cust_001",
  "customer_name": "Acme Corp",
  "created_at": "2025-06-05T14:00:00Z"
}
```

---

### Quotes

**`GET /api/billing/quotes`**

Lists all quotes.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `status` | string | No | `draft`, `sent`, `accepted`, `rejected`, `expired` |
| `customer_id` | string | No | Filter by customer |
| `page` | integer | No | Page number |
| `limit` | integer | No | Items per page |

**Response:**

```json
{
  "quotes": [
    {
      "id": "qte_001",
      "number": "QT-2025-0012",
      "customer_id": "cust_003",
      "customer_name": "TechStart Ltda",
      "status": "sent",
      "total": 12000.00,
      "currency": "BRL",
      "valid_until": "2025-07-04",
      "created_at": "2025-06-01T10:00:00Z"
    }
  ],
  "total": 15
}
```

---

**`POST /api/billing/quotes`**

Creates a new quote.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `customer_id` | string | Yes | Customer ID |
| `items` | array | Yes | Line items (same format as invoices) |
| `currency` | string | No | Currency (default: `BRL`) |
| `valid_until` | string | Yes | ISO 8601 expiry date |
| `notes` | string | No | Quote notes |
| `tax_rate_id` | string | No | Applied tax rate |

**Request Body:**

```json
{
  "customer_id": "cust_003",
  "items": [
    { "description": "BotServer Enterprise License", "quantity": 1, "unit_price": 9600.00 },
    { "description": "Annual Support Plan", "quantity": 1, "unit_price": 2400.00 }
  ],
  "valid_until": "2025-07-04",
  "notes": "Enterprise package with priority support"
}
```

**Response:**

```json
{
  "id": "qte_002",
  "number": "QT-2025-0013",
  "status": "draft",
  "subtotal": 12000.00,
  "total": 12000.00,
  "valid_until": "2025-07-04",
  "created_at": "2025-06-04T10:00:00Z"
}
```

---

**`GET /api/billing/quotes/:id`**

Retrieves a single quote.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Quote ID |

**Response:**

```json
{
  "id": "qte_001",
  "number": "QT-2025-0012",
  "customer_id": "cust_003",
  "customer_name": "TechStart Ltda",
  "status": "sent",
  "subtotal": 12000.00,
  "tax": 1920.00,
  "total": 13920.00,
  "currency": "BRL",
  "valid_until": "2025-07-04",
  "items": [
    { "description": "BotServer Enterprise License", "quantity": 1, "unit_price": 9600.00, "total": 9600.00 },
    { "description": "Annual Support Plan", "quantity": 1, "unit_price": 2400.00, "total": 2400.00 }
  ],
  "created_at": "2025-06-01T10:00:00Z"
}
```

---

**`DELETE /api/billing/quotes/:id`**

Deletes a draft or expired quote.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Quote ID |

**Response:**

```json
{
  "success": true,
  "message": "Quote deleted"
}
```

---

### Accept / Reject Quote

**`PUT /api/billing/quotes/:id/accept`**

Marks a quote as accepted and optionally generates an invoice.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Quote ID |
| `generate_invoice` | boolean | No | Auto-create invoice (default: false) |

**Request Body:**

```json
{
  "generate_invoice": true
}
```

**Response:**

```json
{
  "id": "qte_001",
  "status": "accepted",
  "accepted_at": "2025-06-04T15:00:00Z",
  "invoice_id": "inv_004",
  "invoice_number": "INV-2025-0044"
}
```

---

**`PUT /api/billing/quotes/:id/reject`**

Marks a quote as rejected.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `id` | string | Yes | Quote ID |
| `reason` | string | No | Rejection reason |

**Request Body:**

```json
{
  "reason": "Budget constraints — will revisit next quarter"
}
```

**Response:**

```json
{
  "id": "qte_001",
  "status": "rejected",
  "rejected_at": "2025-06-04T15:30:00Z",
  "rejection_reason": "Budget constraints — will revisit next quarter"
}
```

---

### Billing Statistics

**`GET /api/billing/stats`**

Returns billing overview statistics.

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `period` | string | No | `month`, `quarter`, `year` (default: month) |

**Response:**

```json
{
  "period": "2025-06",
  "revenue": {
    "total": 85400.00,
    "collected": 62300.00,
    "pending": 18700.00,
    "overdue": 4400.00
  },
  "invoices": {
    "total": 42,
    "paid": 28,
    "sent": 8,
    "overdue": 4,
    "draft": 2
  },
  "quotes": {
    "total": 15,
    "accepted": 6,
    "rejected": 3,
    "pending": 6
  },
  "conversion_rate": 40.0,
  "average_payment_days": 18
}
```

---

### Tax Rates

**`GET /api/billing/tax-rates`**

Lists configured tax rates.

**Response:**

```json
{
  "tax_rates": [
    {
      "id": "tax_001",
      "name": "ICMS 16%",
      "rate": 16.0,
      "type": "percentage",
      "applies_to": "all",
      "active": true
    },
    {
      "id": "tax_002",
      "name": "ISS 5%",
      "rate": 5.0,
      "type": "percentage",
      "applies_to": "services",
      "active": true
    },
    {
      "id": "tax_003",
      "name": "PIS 1.65%",
      "rate": 1.65,
      "type": "percentage",
      "applies_to": "all",
      "active": true
    }
  ]
}
```

---

### Recurring Billing

**`GET /api/billing/recurring`**

Lists recurring billing schedules.

**Response:**

```json
{
  "recurring": [
    {
      "id": "rec_001",
      "customer_id": "cust_001",
      "customer_name": "Acme Corp",
      "plan": "BotServer Pro",
      "amount": 400.00,
      "currency": "BRL",
      "interval": "monthly",
      "next_billing_date": "2025-07-01",
      "status": "active",
      "invoices_generated": 6,
      "created_at": "2025-01-01T00:00:00Z"
    }
  ]
}
```

---

## Payment Methods

| Method | Code | Description |
|--------|------|-------------|
| PIX | `pix` | Instant payment (Brazil) |
| Bank Transfer | `bank_transfer` | TED/DOC transfer |
| Credit Card | `credit_card` | Visa, Mastercard, Amex |
| Boleto | `boleto` | Brazilian bank slip |
| Cash | `cash` | Cash payment |

---

## Response Codes

| Code | Description |
|------|-------------|
| 200 | Success |
| 201 | Created |
| 204 | No Content (deletion) |
| 400 | Bad Request |
| 401 | Unauthorized |
| 403 | Forbidden |
| 404 | Resource not found |
| 409 | Conflict (e.g., already paid) |
| 500 | Internal Server Error |

---

## See Also

- [Admin API](./admin-api-full.md) — customer and user management
- [Tasks API](./tasks-api.md) — task and project billing
- [Reports API](./reports-api.md) — financial report generation
