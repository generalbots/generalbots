CREATE TABLE IF NOT EXISTS bot_domains (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    domain          VARCHAR(255) NOT NULL UNIQUE,
    bot_id          UUID NOT NULL REFERENCES bots(id) ON DELETE CASCADE,
    org_id          UUID REFERENCES organizations(org_id) ON DELETE CASCADE,
    branch_id       UUID REFERENCES branches(id) ON DELETE CASCADE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_bot_domains_domain ON bot_domains(domain);
CREATE INDEX IF NOT EXISTS idx_bot_domains_bot_id ON bot_domains(bot_id);
