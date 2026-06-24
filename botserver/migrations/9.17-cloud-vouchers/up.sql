CREATE TABLE IF NOT EXISTS cloud_vouchers (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    code            VARCHAR(20) NOT NULL UNIQUE,
    plan            VARCHAR(50) NOT NULL DEFAULT 'shared',
    trial_days      INTEGER NOT NULL CHECK (trial_days >= 1 AND trial_days <= 180),
    max_uses        INTEGER NOT NULL DEFAULT 1,
    uses_count      INTEGER NOT NULL DEFAULT 0,
    created_by      UUID REFERENCES crm_contacts(id) ON DELETE SET NULL,
    expires_at      TIMESTAMPTZ,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_vouchers_code ON cloud_vouchers(code);
CREATE INDEX IF NOT EXISTS idx_vouchers_expires ON cloud_vouchers(expires_at);

CREATE TABLE IF NOT EXISTS cloud_voucher_redemptions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    voucher_id      UUID NOT NULL REFERENCES cloud_vouchers(id) ON DELETE CASCADE,
    contact_id      UUID NOT NULL REFERENCES crm_contacts(id) ON DELETE CASCADE,
    org_id          UUID NOT NULL REFERENCES organizations(org_id) ON DELETE CASCADE,
    branch_id       UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    subscription_id UUID,
    trial_days      INTEGER NOT NULL,
    redeemed_at     TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
