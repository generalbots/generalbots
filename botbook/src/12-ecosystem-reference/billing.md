# Subscription & Billing Guide

This guide covers General Bots' subscription management, billing integration, quota systems, and white-label billing configuration.

## Overview

General Bots provides a comprehensive billing system that supports:

- **Multiple Plans** - Free, Pro, Business, Enterprise tiers
- **Usage-Based Billing** - Pay for what you use
- **Quota Management** - Enforce limits per organization
- **Stripe Integration** - Secure payment processing
- **White-Label Support** - Custom branding for resellers

## Plan Structure

### Default Plans

| Plan | Price | Members | Bots | Storage | Messages | API Calls |
|------|-------|---------|------|---------|----------|-----------|
| Free | $0/mo | 5 | 2 | 1 GB | 1,000/mo | 10,000/mo |
| Pro | $49/mo | 50 | 20 | 50 GB | 100,000/mo | 500,000/mo |
| Business | $99/mo | 200 | 100 | 200 GB | Unlimited | Unlimited |
| Enterprise | Custom | Unlimited | Unlimited | 1 TB+ | Unlimited | Unlimited |

### Plan Configuration

Plans are defined in `.product` files for white-label deployments:

```yaml
version: 1
product:
  name: "My SaaS Product"
  company: "My Company"
  
plans:
  - id: starter
    name: Starter
    price_monthly: 29
    price_annual: 290
    features:
      max_members: 10
      max_bots: 5
      storage_gb: 10
      messages_per_month: 10000
      api_calls_per_month: 50000
      support_level: email
      
  - id: professional
    name: Professional
    price_monthly: 79
    price_annual: 790
    features:
      max_members: 50
      max_bots: 25
      storage_gb: 100
      messages_per_month: 100000
      api_calls_per_month: 500000
      support_level: priority
      custom_branding: true
      
  - id: enterprise
    name: Enterprise
    price_monthly: 0
    custom_pricing: true
    features:
      max_members: -1
      max_bots: -1
      storage_gb: 1000
      messages_per_month: -1
      api_calls_per_month: -1
      support_level: dedicated
      custom_branding: true
      sso: true
      audit_logs: true
      sla: "99.9%"
```

## Stripe Integration

### Configuration

Add Stripe credentials to your environment:

```bash
STRIPE_SECRET_KEY=sk_live_...
STRIPE_PUBLISHABLE_KEY=pk_live_...
STRIPE_WEBHOOK_SECRET=whsec_...
STRIPE_PRICE_ID_PRO_MONTHLY=price_...
STRIPE_PRICE_ID_PRO_ANNUAL=price_...
STRIPE_PRICE_ID_BUSINESS_MONTHLY=price_...
STRIPE_PRICE_ID_BUSINESS_ANNUAL=price_...
```

### Webhook Events

General Bots handles these Stripe webhook events:

| Event | Action |
|-------|--------|
| `customer.subscription.created` | Activate subscription |
| `customer.subscription.updated` | Update plan/status |
| `customer.subscription.deleted` | Cancel subscription |
| `invoice.paid` | Record payment |
| `invoice.payment_failed` | Handle failed payment |
| `customer.subscription.trial_will_end` | Send trial ending notification |

### Setting Up Webhooks

1. Go to Stripe Dashboard → Developers → Webhooks
2. Add endpoint: `https://your-domain.com/api/billing/webhooks/stripe`
3. Select events to listen for
4. Copy webhook secret to `STRIPE_WEBHOOK_SECRET`

## Subscription Lifecycle

### Creating a Subscription

```http
POST /api/billing/subscriptions
Authorization: Bearer <token>
Content-Type: application/json

{
  "plan_id": "pro",
  "billing_cycle": "monthly",
  "payment_method_id": "pm_...",
  "trial_days": 14,
  "coupon_code": "LAUNCH20"
}
```

Response:

```json
{
  "subscription_id": "sub_...",
  "organization_id": "org-uuid",
  "plan_id": "pro",
  "status": "trialing",
  "trial_end": "2025-02-04T00:00:00Z",
  "current_period_end": "2025-03-21T00:00:00Z",
  "cancel_at_period_end": false
}
```

### Upgrading/Downgrading

```http
POST /api/billing/subscriptions/{subscription_id}/change
Authorization: Bearer <token>
Content-Type: application/json

{
  "new_plan_id": "business",
  "prorate": true,
  "immediate": true
}
```

Proration calculates the difference and adjusts the next invoice.

### Cancellation

```http
POST /api/billing/subscriptions/{subscription_id}/cancel
Authorization: Bearer <token>
Content-Type: application/json

{
  "reason": "too_expensive",
  "feedback": "Need more affordable options",
  "cancel_immediately": false,
  "offer_retention": true
}
```

When `offer_retention` is true, the system may return retention offers:

```json
{
  "status": "pending_retention",
  "offers": [
    {
      "id": "offer-uuid",
      "type": "discount",
      "discount_percent": 25,
      "duration_months": 3,
      "expires_at": "2025-01-28T00:00:00Z"
    },
    {
      "id": "offer-uuid-2",
      "type": "pause",
      "pause_months": 3
    }
  ]
}
```

### Pausing a Subscription

```http
POST /api/billing/subscriptions/{subscription_id}/pause
Authorization: Bearer <token>
Content-Type: application/json

{
  "pause_months": 3
}
```

### Resuming a Subscription

```http
POST /api/billing/subscriptions/{subscription_id}/resume
Authorization: Bearer <token>
```

## Quota Management

### Quota Types

| Quota | Enforcement | Reset |
|-------|-------------|-------|
| `members` | Hard limit | N/A |
| `bots` | Hard limit | N/A |
| `storage_bytes` | Hard limit | N/A |
| `messages` | Soft limit | Monthly |
| `api_calls` | Rate limit | Monthly |

### Checking Quotas

```http
GET /api/billing/quotas
Authorization: Bearer <token>
```

Response:

```json
{
  "organization_id": "org-uuid",
  "plan_id": "pro",
  "quotas": {
    "members": {
      "used": 12,
      "limit": 50,
      "percent": 24
    },
    "bots": {
      "used": 5,
      "limit": 20,
      "percent": 25
    },
    "storage_bytes": {
      "used": 5368709120,
      "limit": 53687091200,
      "percent": 10
    },
    "messages": {
      "used": 45000,
      "limit": 100000,
      "percent": 45,
      "resets_at": "2025-02-01T00:00:00Z"
    },
    "api_calls": {
      "used": 125000,
      "limit": 500000,
      "percent": 25,
      "resets_at": "2025-02-01T00:00:00Z"
    }
  }
}
```

### Usage Alerts

The system sends alerts at these thresholds:

| Threshold | Alert Type | Notification |
|-----------|------------|--------------|
| 80% | Warning | Email, In-app |
| 90% | Critical | Email, In-app, Webhook |
| 100% | Limit Reached | Email, In-app, Webhook |

### Configuring Alerts

```http
PUT /api/billing/alerts/preferences
Authorization: Bearer <token>
Content-Type: application/json

{
  "email_notifications": true,
  "webhook_url": "https://your-app.com/webhooks/quota",
  "slack_webhook": "https://hooks.slack.com/...",
  "alert_thresholds": [70, 85, 95, 100]
}
```

### Grace Period

When quotas are exceeded, organizations enter a grace period:

- Default: 7 days
- During grace: Soft limits become warnings
- After grace: Hard enforcement applies

```http
GET /api/billing/grace-period
Authorization: Bearer <token>
```

Response:

```json
{
  "in_grace_period": true,
  "exceeded_quotas": ["messages"],
  "grace_started": "2025-01-21T00:00:00Z",
  "grace_ends": "2025-01-28T00:00:00Z",
  "overage_percent": 15,
  "recommended_action": "upgrade_plan"
}
```

## Usage Metering

### Recording Usage

Usage is automatically tracked, but can be manually recorded:

```http
POST /api/billing/usage
Authorization: Bearer <token>
Content-Type: application/json

{
  "metric": "api_calls",
  "quantity": 100,
  "timestamp": "2025-01-21T10:30:00Z",
  "metadata": {
    "endpoint": "/api/chat",
    "bot_id": "bot-uuid"
  }
}
```

### Querying Usage

```http
GET /api/billing/usage/history
Authorization: Bearer <token>
```

Query parameters:

| Parameter | Description |
|-----------|-------------|
| `metric` | Filter by metric type |
| `start_date` | Period start |
| `end_date` | Period end |
| `granularity` | hour, day, week, month |

Response:

```json
{
  "metric": "api_calls",
  "period": {
    "start": "2025-01-01T00:00:00Z",
    "end": "2025-01-31T23:59:59Z"
  },
  "total": 125000,
  "data_points": [
    {"date": "2025-01-01", "value": 4200},
    {"date": "2025-01-02", "value": 3800},
    ...
  ]
}
```

## Invoices

### Listing Invoices

```http
GET /api/billing/invoices
Authorization: Bearer <token>
```

Response:

```json
{
  "invoices": [
    {
      "id": "inv_...",
      "number": "INV-2025-0001",
      "date": "2025-01-21",
      "amount": 4900,
      "currency": "usd",
      "status": "paid",
      "pdf_url": "https://...",
      "items": [
        {
          "description": "Pro Plan (Monthly)",
          "amount": 4900,
          "quantity": 1
        }
      ]
    }
  ],
  "has_more": false
}
```

### Downloading Invoice PDF

```http
GET /api/billing/invoices/{invoice_id}/pdf
Authorization: Bearer <token>
```

Returns PDF binary with `Content-Type: application/pdf`.

## Payment Methods

### Adding a Payment Method

```http
POST /api/billing/payment-methods
Authorization: Bearer <token>
Content-Type: application/json

{
  "payment_method_id": "pm_...",
  "set_as_default": true
}
```

### Listing Payment Methods

```http
GET /api/billing/payment-methods
Authorization: Bearer <token>
```

### Removing a Payment Method

```http
DELETE /api/billing/payment-methods/{payment_method_id}
Authorization: Bearer <token>
```

## White-Label Billing

### Custom Branding

Configure billing UI branding in your `.product` file:

```yaml
branding:
  billing:
    company_name: "Your Company"
    support_email: "billing@yourcompany.com"
    invoice_logo: "/assets/logo.png"
    invoice_footer: "Thank you for your business!"
    currency: "usd"
    tax_id_label: "VAT Number"
```

### Reseller Mode

Enable reseller billing to manage customer subscriptions:

```yaml
reseller:
  enabled: true
  commission_percent: 20
  min_markup_percent: 0
  allow_custom_pricing: true
  billing_relationship: "reseller"  # or "direct"
```

### Custom Plan Pricing

Resellers can set custom pricing:

```http
POST /api/billing/reseller/plans/{plan_id}/pricing
Authorization: Bearer <token>
Content-Type: application/json

{
  "customer_id": "customer-uuid",
  "custom_price_monthly": 7900,
  "custom_price_annual": 79000
}
```

## Self-Hosted Mode

For self-hosted installations without SaaS billing:

```bash
SAAS_MODE=false
LOCAL_QUOTA_MODE=true
```

In local quota mode:
- No payment processing
- Quotas enforced locally
- No subscription management
- Resource limits based on configuration

Configure local quotas:

```yaml
local_quotas:
  max_members: 100
  max_bots: 50
  max_storage_gb: 500
  max_messages_per_month: 1000000
```

## Events and Webhooks

### Subscription Events

```json
{
  "event": "subscription.created",
  "timestamp": "2025-01-21T10:00:00Z",
  "data": {
    "subscription_id": "sub_...",
    "organization_id": "org-uuid",
    "plan_id": "pro",
    "status": "active"
  }
}
```

Event types:

| Event | Description |
|-------|-------------|
| `subscription.created` | New subscription started |
| `subscription.updated` | Plan or status changed |
| `subscription.cancelled` | Subscription cancelled |
| `subscription.renewed` | Subscription renewed |
| `payment.succeeded` | Payment processed |
| `payment.failed` | Payment failed |
| `quota.warning` | Quota threshold reached |
| `quota.exceeded` | Quota limit exceeded |

### Configuring Webhooks

```http
POST /api/billing/webhooks
Authorization: Bearer <token>
Content-Type: application/json

{
  "url": "https://your-app.com/webhooks/billing",
  "events": ["subscription.*", "payment.*", "quota.*"],
  "secret": "your-webhook-secret"
}
```

## Best Practices

### For SaaS Operators

1. **Start with a free tier** - Lower barrier to entry
2. **Offer annual discounts** - 15-20% savings encourages commitment
3. **Set clear usage limits** - Avoid surprise bills
4. **Provide grace periods** - Don't cut off access immediately
5. **Retention offers work** - Discounts reduce churn

### For Enterprise Deployments

1. **Custom contracts** - Negotiate terms for large deals
2. **Usage commitments** - Discounts for guaranteed usage
3. **Multi-year deals** - Better pricing for longer terms
4. **Success management** - Dedicated support reduces churn

### For Self-Hosted

1. **Resource planning** - Set realistic quotas
2. **Monitor usage** - Track before limits are hit
3. **Scale proactively** - Add capacity before needed

## Troubleshooting

### Payment Failed

1. Check payment method is valid
2. Verify billing address
3. Contact card issuer if declined
4. Add backup payment method

### Quota Exceeded

1. Review usage in dashboard
2. Identify heavy consumers
3. Upgrade plan or reduce usage
4. Request temporary increase (enterprise)

### Invoice Issues

1. Check billing email settings
2. Verify invoice delivery
3. Download from dashboard
4. Contact support for corrections

## Related Topics

- [Organization Management](../09-security/organizations.md)
- [RBAC Configuration](../09-security/rbac-configuration.md)
- [White-Label Setup](./setup.md)
- [API Authentication](../08-rest-api-tools/authentication.md)