-- ITSM concepts folded into the Tickets app (record type + CMDB + KB articles).

ALTER TABLE support_tickets ADD COLUMN IF NOT EXISTS record_type VARCHAR(50) NOT NULL DEFAULT 'ticket';
CREATE INDEX IF NOT EXISTS idx_support_tickets_record_type ON support_tickets(record_type);

CREATE TABLE IF NOT EXISTS ticket_cis (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    name VARCHAR(255) NOT NULL,
    ci_type VARCHAR(100),
    description TEXT,
    status VARCHAR(50) NOT NULL DEFAULT 'operational',
    owner_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE IF NOT EXISTS ticket_kb_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL,
    branch_id UUID,
    title VARCHAR(500) NOT NULL,
    body TEXT NOT NULL,
    category VARCHAR(100),
    tags TEXT[] NOT NULL DEFAULT '{}',
    is_published BOOLEAN NOT NULL DEFAULT TRUE,
    author_id UUID,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
