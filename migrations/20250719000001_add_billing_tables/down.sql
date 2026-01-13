DROP INDEX IF EXISTS idx_billing_tax_rates_org_bot;

DROP INDEX IF EXISTS idx_billing_recurring_next;
DROP INDEX IF EXISTS idx_billing_recurring_status;
DROP INDEX IF EXISTS idx_billing_recurring_org_bot;

DROP INDEX IF EXISTS idx_billing_quote_items_quote;

DROP INDEX IF EXISTS idx_billing_quotes_number;
DROP INDEX IF EXISTS idx_billing_quotes_valid_until;
DROP INDEX IF EXISTS idx_billing_quotes_customer;
DROP INDEX IF EXISTS idx_billing_quotes_status;
DROP INDEX IF EXISTS idx_billing_quotes_org_bot;

DROP INDEX IF EXISTS idx_billing_payments_number;
DROP INDEX IF EXISTS idx_billing_payments_paid_at;
DROP INDEX IF EXISTS idx_billing_payments_status;
DROP INDEX IF EXISTS idx_billing_payments_invoice;
DROP INDEX IF EXISTS idx_billing_payments_org_bot;

DROP INDEX IF EXISTS idx_billing_invoice_items_invoice;

DROP INDEX IF EXISTS idx_billing_invoices_number;
DROP INDEX IF EXISTS idx_billing_invoices_created;
DROP INDEX IF EXISTS idx_billing_invoices_due_date;
DROP INDEX IF EXISTS idx_billing_invoices_customer;
DROP INDEX IF EXISTS idx_billing_invoices_status;
DROP INDEX IF EXISTS idx_billing_invoices_org_bot;

DROP TABLE IF EXISTS billing_tax_rates;
DROP TABLE IF EXISTS billing_recurring;
DROP TABLE IF EXISTS billing_quote_items;
DROP TABLE IF EXISTS billing_quotes;
DROP TABLE IF EXISTS billing_payments;
DROP TABLE IF EXISTS billing_invoice_items;
DROP TABLE IF EXISTS billing_invoices;
