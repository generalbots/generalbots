-- 6.5.54-tax-calculations
-- #722: audit trail for service-tax calculations. Every calculation performed
-- through the chat (service.tax) or REST (/api/tax/calculate) is persisted
-- with the tenant (branch) scope for audit and reporting.

CREATE TABLE IF NOT EXISTS tax_calculations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    service_id UUID,
    service_name VARCHAR(255),
    service_value NUMERIC(18,2) NOT NULL,
    irpj NUMERIC(18,2) NOT NULL DEFAULT 0,
    csll NUMERIC(18,2) NOT NULL DEFAULT 0,
    pis_cofins NUMERIC(18,2) NOT NULL DEFAULT 0,
    iss NUMERIC(18,2) NOT NULL DEFAULT 0,
    total_taxes NUMERIC(18,2) NOT NULL DEFAULT 0,
    effective_rate NUMERIC(6,4) NOT NULL DEFAULT 0,
    rate_source VARCHAR(32) NOT NULL DEFAULT 'default',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX IF NOT EXISTS idx_tax_calculations_branch_created
    ON tax_calculations(branch_id, created_at DESC);
