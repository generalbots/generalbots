CREATE TABLE delivery_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    platform VARCHAR(50) NOT NULL,
    platform_order_id VARCHAR(255) NOT NULL,
    order_date DATE NOT NULL,
    customer_name TEXT,
    items JSONB,
    subtotal NUMERIC(12,2) NOT NULL,
    delivery_fee NUMERIC(12,2) NOT NULL DEFAULT 0,
    platform_commission NUMERIC(12,2) NOT NULL DEFAULT 0,
    net_value NUMERIC(12,2) NOT NULL,
    payment_method VARCHAR(50),
    status VARCHAR(50) NOT NULL DEFAULT 'pending',
    reconciled BOOLEAN NOT NULL DEFAULT false,
    reconciled_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_delivery_bot_id ON delivery_transactions(bot_id);
CREATE INDEX idx_delivery_platform ON delivery_transactions(platform);
CREATE INDEX idx_delivery_order_id ON delivery_transactions(platform_order_id);
CREATE INDEX idx_delivery_reconciled ON delivery_transactions(reconciled);
CREATE INDEX idx_delivery_date ON delivery_transactions(order_date);

CREATE TABLE bank_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    bank VARCHAR(100),
    account VARCHAR(50),
    transaction_date DATE NOT NULL,
    description TEXT NOT NULL,
    amount NUMERIC(12,2) NOT NULL,
    balance NUMERIC(12,2),
    category VARCHAR(100),
    reconciled BOOLEAN NOT NULL DEFAULT false,
    reconciled_at TIMESTAMPTZ,
    matched_delivery_id UUID REFERENCES delivery_transactions(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_bank_tx_bot_id ON bank_transactions(bot_id);
CREATE INDEX idx_bank_tx_date ON bank_transactions(transaction_date);
CREATE INDEX idx_bank_tx_reconciled ON bank_transactions(reconciled);
CREATE INDEX idx_bank_tx_matched ON bank_transactions(matched_delivery_id);
CREATE INDEX idx_bank_tx_amount ON bank_transactions(amount);

CREATE TABLE reconciliation_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    name VARCHAR(255) NOT NULL,
    match_field VARCHAR(50) NOT NULL,
    match_operator VARCHAR(20) NOT NULL,
    match_value TEXT NOT NULL,
    category VARCHAR(100),
    auto_reconcile BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX idx_recon_rules_bot ON reconciliation_rules(bot_id);
CREATE INDEX idx_recon_rules_active ON reconciliation_rules(is_active);

CREATE TABLE reconciliation_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    bot_id UUID NOT NULL,
    status VARCHAR(50) NOT NULL DEFAULT 'running',
    matched_count INTEGER NOT NULL DEFAULT 0,
    unmatched_count INTEGER NOT NULL DEFAULT 0,
    total_amount_matched NUMERIC(14,2) NOT NULL DEFAULT 0,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX idx_recon_runs_bot ON reconciliation_runs(bot_id);
CREATE INDEX idx_recon_runs_status ON reconciliation_runs(status);
