-- 6.5.56-persistent-invitations
-- The InvitationService previously kept invitations in in-memory HashMaps,
-- losing every invite on restart and never binding accepted members to the
-- org. This migration adds the columns the persistent service needs
-- (groups, invited_by_name) and backfills nothing — existing in-memory
-- invites are gone by design; new invites are durable.

ALTER TABLE organization_invitations
    ADD COLUMN IF NOT EXISTS groups JSONB NOT NULL DEFAULT '[]'::jsonb,
    ADD COLUMN IF NOT EXISTS invited_by_name VARCHAR(255);

-- Token lookup is the accept path; ensure it is indexed (UNIQUE already
-- exists on token; add an index for org listing too).
CREATE INDEX IF NOT EXISTS idx_org_invitations_token ON organization_invitations(token);