CREATE TABLE IF NOT EXISTS marketing_contacts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL REFERENCES branches(id) ON DELETE CASCADE,
    list_id UUID REFERENCES marketing_lists(id) ON DELETE CASCADE,
    email VARCHAR NOT NULL,
    name VARCHAR,
    phone VARCHAR,
    metadata JSONB,
    subscribed BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (list_id, email)
);
CREATE INDEX IF NOT EXISTS idx_marketing_contacts_branch_id ON marketing_contacts(branch_id);
CREATE INDEX IF NOT EXISTS idx_marketing_contacts_list_id ON marketing_contacts(list_id);
