CREATE INDEX IF NOT EXISTS idx_org_invitations_token ON organization_invitations(token); ALTER TABLE organization_invitations DROP COLUMN IF EXISTS groups, DROP COLUMN IF EXISTS invited_by_name;
