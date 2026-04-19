# Products - Product & Service Catalog

> **Manage your complete product and service offerings**

---

## Overview

Products is your catalog management solution following Microsoft Dynamics nomenclature. Maintain your product inventory, service offerings, and price lists for use across CRM quotes and billing invoices.

---

## Key Concepts (Dynamics Nomenclature)

| Entity | Description |
|--------|-------------|
| **Product** | Physical or digital item for sale |
| **Service** | Service offering (hourly, fixed, recurring) |
| **Price List** | Pricing tier with currency and validity dates |

### Entity Relationships

```
Product/Service ──► Price List ──► Quote/Invoice Line Items
```

---

## Features

### Product Catalog

Manage your product inventory:

- **Name** - Product name
- **Description** - Product details
- **SKU** - Stock keeping unit code
- **Category** - Product classification
- **Price** - Base price
- **Unit** - Unit of measure
- **Status** - Active or Inactive

### Product Categories

| Category | Description |
|----------|-------------|
| **Software** | Software licenses and subscriptions |
| **Hardware** | Physical equipment and devices |
| **Subscription** | Recurring subscription products |
| **Consulting** | Consulting packages |
| **Training** | Training courses and materials |
| **Support** | Support packages and plans |

### Services

Track your service offerings:

- **Name** - Service name
- **Description** - Service details
- **Type** - Billing type (hourly, fixed, recurring)
- **Price** - Service rate
- **Unit** - Billing unit (hour, project, month)
- **Status** - Active or Inactive

### Service Types

| Type | Description |
|------|-------------|
| **Hourly** | Billed per hour worked |
| **Fixed Price** | One-time fixed amount |
| **Recurring** | Monthly/annual subscription |

### Price Lists

Manage pricing across different contexts:

- **Name** - Price list name
- **Description** - Purpose or context
- **Currency** - USD, EUR, BRL, GBP, etc.
- **Items** - Number of products/services included
- **Valid From** - Start date
- **Valid To** - End date
- **Default** - Is this the default price list?

---

## Summary Dashboard

Real-time catalog metrics:

| Metric | Description |
|--------|-------------|
| **Total Products** | Number of products in catalog |
| **Total Services** | Number of services offered |
| **Active Items** | Currently active products and services |
| **Price Lists** | Number of configured price lists |

---

## Navigation Tabs

| Tab | View |
|-----|------|
| **Catalog** | Product grid/list view |
| **Services** | Service table |
| **Price Lists** | Price list management |

---

## View Options

The catalog supports two view modes:

| View | Description |
|------|-------------|
| **Grid** | Visual card layout (default) |
| **List** | Compact table layout |

Toggle between views using the view buttons in the toolbar.

---

## API Endpoints

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/api/products/items` | GET | List products with filters |
| `/api/products/items` | POST | Create new product |
| `/api/products/items/:id` | GET | Get product details |
| `/api/products/items/:id` | PUT | Update product |
| `/api/products/items/:id` | DELETE | Delete product |
| `/api/products/services` | GET | List services |
| `/api/products/services` | POST | Create new service |
| `/api/products/services/:id` | GET | Get service details |
| `/api/products/pricelists` | GET | List price lists |
| `/api/products/pricelists` | POST | Create new price list |
| `/api/products/pricelists/:id` | GET | Get price list details |
| `/api/products/pricelists/:id/items` | GET | Get items in price list |
| `/api/products/search` | GET | Search products and services |
| `/api/products/stats/*` | GET | Get catalog statistics |

---

## @ Mentions in Chat

Reference products directly in chat:

| Mention | Example |
|---------|---------|
| `@product:` | @product:Enterprise License |

Hover over a mention to see product details. Click to navigate to the record.

---

## Filtering Options

### Catalog Filters

| Filter | Options |
|--------|---------|
| **Category** | All, Software, Hardware, Subscription, Consulting, Training, Support |
| **Status** | Active, All, Inactive |

### Service Filters

| Filter | Options |
|--------|---------|
| **Type** | All Types, Hourly, Fixed Price, Recurring |

### Price List Filters

| Filter | Options |
|--------|---------|
| **Currency** | All Currencies, USD, EUR, BRL, GBP |

---

## Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `N` | New product (when in Products) |
| `Escape` | Close modal |
| `/` | Focus search |
| `G` | Toggle grid/list view |

---

## Integration with Billing

Products integrate with the billing system:

1. **Quote Line Items** - Add products/services to quotes
2. **Invoice Line Items** - Products appear on invoices
3. **Price List Selection** - Choose appropriate price list per quote/invoice
4. **Automatic Pricing** - Prices pulled from selected price list

---

## Best Practices

### Catalog Management

1. **Use clear names** - Make products easily identifiable
2. **Complete descriptions** - Include all relevant details
3. **Assign SKUs** - Use consistent SKU naming conventions
4. **Categorize properly** - Assign appropriate categories

### Pricing Strategy

1. **Multiple price lists** - Create lists for different markets/customers
2. **Set validity dates** - Use date ranges for promotional pricing
3. **Mark defaults** - Designate default price list per currency
4. **Review regularly** - Update pricing as needed

### Service Offerings

1. **Define scope** - Clear descriptions prevent misunderstandings
2. **Set appropriate units** - Choose billing units that match delivery
3. **Track utilization** - Monitor which services sell best

---

## Multi-Currency Support

Price lists support multiple currencies:

| Currency | Code |
|----------|------|
| US Dollar | USD |
| Euro | EUR |
| Brazilian Real | BRL |
| British Pound | GBP |

Create separate price lists for each currency you operate in.

---

## See Also

- [CRM](./crm.md) — Add products to opportunities
- [Billing](./billing.md) — Include products in invoices and quotes
- [Analytics](./analytics.md) — Product performance reports