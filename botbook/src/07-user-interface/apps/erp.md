# ERP → Billing (unified)

> **Financial, inventory & procurement folded into Billing**

The former standalone **ERP** app has been **unified into Billing**. There is no separate
ERP app in the launcher or catalog anymore — inventory, general ledger and procurement
are available as tabs inside the **Billing** app.

See [Billing](./billing.md) for the full documentation.

## What was unified

| ERP concept | Now in Billing (tab) |
|-------------|----------------------|
| Inventory | `Inventory` tab — reads `inventory_items` (scoped by bot) |
| General Ledger | `GL` tab — `gl_accounts` + `gl_journal_lines` (scoped by bot) |
| Procurement | `Procurement` tab — `purchase_orders` (scoped by bot) |

## Fragment endpoints

| Endpoint | Content |
|----------|---------|
| `/api/ui/billing/inventory` | Inventory table (HTML) |
| `/api/ui/billing/gl/accounts` | GL accounts (HTML) |
| `/api/ui/billing/gl/balance-sheet` | Balance sheet (HTML) |
| `/api/ui/billing/gl/income-statement` | Income statement (HTML) |
| `/api/ui/billing/procurement` | Purchase orders (HTML) |

All ERP/GL queries filter by the default bot id to enforce data isolation between
branches/tenants.

## Related

- [Billing](./billing.md)
- [Billing API](../08-rest-api-tools/billing-api.md)
