CREATE TABLE billing_invoices (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    invoice_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    customer_address TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    issue_date DATE NOT NULL,
    due_date DATE NOT NULL,
    subtotal DECIMAL(15,2) NOT NULL DEFAULT 0,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,2) NOT NULL DEFAULT 0,
    discount_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(15,2) NOT NULL DEFAULT 0,
    total DECIMAL(15,2) NOT NULL DEFAULT 0,
    amount_paid DECIMAL(15,2) NOT NULL DEFAULT 0,
    amount_due DECIMAL(15,2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    notes TEXT,
    terms TEXT,
    footer TEXT,
    paid_at TIMESTAMPTZ,
    sent_at TIMESTAMPTZ,
    voided_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_invoice_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    invoice_id UUID NOT NULL REFERENCES billing_invoices(id) ON DELETE CASCADE,
    product_id UUID,
    description VARCHAR(500) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL DEFAULT 1,
    unit_price DECIMAL(15,2) NOT NULL,
    discount_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    amount DECIMAL(15,2) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_payments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    invoice_id UUID REFERENCES billing_invoices(id) ON DELETE SET NULL,
    payment_number VARCHAR(50) NOT NULL,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    payment_method VARCHAR(50) NOT NULL DEFAULT 'other',
    payment_reference VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'completed',
    payer_name VARCHAR(255),
    payer_email VARCHAR(255),
    notes TEXT,
    paid_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    refunded_at TIMESTAMPTZ,
    refund_amount DECIMAL(15,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_quotes (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    quote_number VARCHAR(50) NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    customer_address TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    issue_date DATE NOT NULL,
    valid_until DATE NOT NULL,
    subtotal DECIMAL(15,2) NOT NULL DEFAULT 0,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_amount DECIMAL(15,2) NOT NULL DEFAULT 0,
    discount_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    discount_amount DECIMAL(15,2) NOT NULL DEFAULT 0,
    total DECIMAL(15,2) NOT NULL DEFAULT 0,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    notes TEXT,
    terms TEXT,
    accepted_at TIMESTAMPTZ,
    rejected_at TIMESTAMPTZ,
    converted_invoice_id UUID REFERENCES billing_invoices(id) ON DELETE SET NULL,
    sent_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_quote_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    quote_id UUID NOT NULL REFERENCES billing_quotes(id) ON DELETE CASCADE,
    product_id UUID,
    description VARCHAR(500) NOT NULL,
    quantity DECIMAL(10,2) NOT NULL DEFAULT 1,
    unit_price DECIMAL(15,2) NOT NULL,
    discount_percent DECIMAL(5,2) NOT NULL DEFAULT 0,
    tax_rate DECIMAL(5,2) NOT NULL DEFAULT 0,
    amount DECIMAL(15,2) NOT NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_recurring (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    customer_id UUID,
    customer_name VARCHAR(255) NOT NULL,
    customer_email VARCHAR(255),
    status VARCHAR(50) NOT NULL DEFAULT 'active',
    frequency VARCHAR(50) NOT NULL DEFAULT 'monthly',
    interval_count INTEGER NOT NULL DEFAULT 1,
    amount DECIMAL(15,2) NOT NULL,
    currency VARCHAR(3) NOT NULL DEFAULT 'USD',
    description TEXT,
    next_invoice_date DATE NOT NULL,
    last_invoice_date DATE,
    last_invoice_id UUID REFERENCES billing_invoices(id) ON DELETE SET NULL,
    start_date DATE NOT NULL,
    end_date DATE,
    invoices_generated INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE billing_tax_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    bot_id UUID NOT NULL,
    name VARCHAR(100) NOT NULL,
    rate DECIMAL(5,2) NOT NULL,
    description TEXT,
    region VARCHAR(100),
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_billing_invoices_org_bot ON billing_invoices(org_id, bot_id);
CREATE INDEX idx_billing_invoices_status ON billing_invoices(status);
CREATE INDEX idx_billing_invoices_customer ON billing_invoices(customer_id);
CREATE INDEX idx_billing_invoices_due_date ON billing_invoices(due_date);
CREATE INDEX idx_billing_invoices_created ON billing_invoices(created_at DESC);
CREATE UNIQUE INDEX idx_billing_invoices_number ON billing_invoices(org_id, invoice_number);

CREATE INDEX idx_billing_invoice_items_invoice ON billing_invoice_items(invoice_id);

CREATE INDEX idx_billing_payments_org_bot ON billing_payments(org_id, bot_id);
CREATE INDEX idx_billing_payments_invoice ON billing_payments(invoice_id);
CREATE INDEX idx_billing_payments_status ON billing_payments(status);
CREATE INDEX idx_billing_payments_paid_at ON billing_payments(paid_at DESC);
CREATE UNIQUE INDEX idx_billing_payments_number ON billing_payments(org_id, payment_number);

CREATE INDEX idx_billing_quotes_org_bot ON billing_quotes(org_id, bot_id);
CREATE INDEX idx_billing_quotes_status ON billing_quotes(status);
CREATE INDEX idx_billing_quotes_customer ON billing_quotes(customer_id);
CREATE INDEX idx_billing_quotes_valid_until ON billing_quotes(valid_until);
CREATE UNIQUE INDEX idx_billing_quotes_number ON billing_quotes(org_id, quote_number);

CREATE INDEX idx_billing_quote_items_quote ON billing_quote_items(quote_id);

CREATE INDEX idx_billing_recurring_org_bot ON billing_recurring(org_id, bot_id);
CREATE INDEX idx_billing_recurring_status ON billing_recurring(status);
CREATE INDEX idx_billing_recurring_next ON billing_recurring(next_invoice_date);

CREATE INDEX idx_billing_tax_rates_org_bot ON billing_tax_rates(org_id, bot_id);
