-- Stripe payment card management (SetupIntent) — enterprise-grade release
--
-- Tracks the Stripe Customer per branch so SetupIntents, payment methods and
-- the default card can be resolved without asking Stripe for the customer on
-- every request. Card numbers never touch our servers: Stripe holds the PAN
-- and we persist only display metadata (brand, last4, expiry).

CREATE TABLE IF NOT EXISTS billing_customers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL UNIQUE,
    stripe_customer_id VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_billing_customers_stripe_id
    ON billing_customers (stripe_customer_id);

CREATE TABLE IF NOT EXISTS billing_payment_methods (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    stripe_customer_id VARCHAR(255) NOT NULL,
    stripe_pm_id VARCHAR(255) NOT NULL UNIQUE,
    brand VARCHAR(50) NOT NULL,
    last4 VARCHAR(4) NOT NULL,
    exp_month INT NOT NULL,
    exp_year INT NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_billing_payment_methods_branch
    ON billing_payment_methods (branch_id);
CREATE INDEX IF NOT EXISTS idx_billing_payment_methods_customer
    ON billing_payment_methods (stripe_customer_id);

-- Billing-scoped audit trail: every card add / remove / default change is
-- recorded here with the acting user (see also the bot-scoped audit_log).
CREATE TABLE IF NOT EXISTS cloud_audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    actor_email VARCHAR(255),
    action VARCHAR(100) NOT NULL,
    entity VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255),
    details TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_cloud_audit_log_branch
    ON cloud_audit_log (branch_id, created_at DESC);
