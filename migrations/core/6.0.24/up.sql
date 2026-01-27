-- Organization Invitations Table
-- Manages user invitations to organizations

CREATE TABLE IF NOT EXISTS organization_invitations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    org_id UUID NOT NULL REFERENCES organizations(id) ON DELETE CASCADE,
    email VARCHAR(255) NOT NULL,
    role VARCHAR(50) NOT NULL DEFAULT 'member',
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    message TEXT,
    invited_by UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    token VARCHAR(255) UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ,
    accepted_at TIMESTAMPTZ,
    accepted_by UUID REFERENCES users(id) ON DELETE SET NULL,

    -- Constraint to prevent duplicate pending invitations
    CONSTRAINT unique_pending_invitation UNIQUE (org_id, email)
);

-- Index for looking up invitations by organization
CREATE INDEX IF NOT EXISTS idx_org_invitations_org_id ON organization_invitations(org_id);

-- Index for looking up invitations by email
CREATE INDEX IF NOT EXISTS idx_org_invitations_email ON organization_invitations(email);

-- Index for looking up pending invitations
CREATE INDEX IF NOT EXISTS idx_org_invitations_status ON organization_invitations(status) WHERE status = 'pending';

-- Index for token lookups (for invitation acceptance)
CREATE INDEX IF NOT EXISTS idx_org_invitations_token ON organization_invitations(token) WHERE token IS NOT NULL;

-- Index for cleanup of expired invitations
CREATE INDEX IF NOT EXISTS idx_org_invitations_expires ON organization_invitations(expires_at) WHERE status = 'pending';

-- Add updated_at trigger
CREATE OR REPLACE FUNCTION update_org_invitation_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trigger_org_invitation_updated_at ON organization_invitations;
CREATE TRIGGER trigger_org_invitation_updated_at
    BEFORE UPDATE ON organization_invitations
    FOR EACH ROW
    EXECUTE FUNCTION update_org_invitation_updated_at();

-- Comments
COMMENT ON TABLE organization_invitations IS 'Stores pending and historical organization invitations';
COMMENT ON COLUMN organization_invitations.status IS 'pending, accepted, cancelled, expired';
COMMENT ON COLUMN organization_invitations.token IS 'Secure token for invitation acceptance via email link';
COMMENT ON COLUMN organization_invitations.role IS 'Role to assign upon acceptance: member, admin, owner, etc.';
