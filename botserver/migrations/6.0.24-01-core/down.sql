-- Down migration: Remove organization invitations table

DROP TRIGGER IF EXISTS trigger_org_invitation_updated_at ON organization_invitations;
DROP FUNCTION IF EXISTS update_org_invitation_updated_at();
DROP TABLE IF EXISTS organization_invitations;
