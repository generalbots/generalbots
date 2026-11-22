-- #2536 — durable, branch-scoped do-not-contact list.
--
-- The in-process suppression store is the fast gate; this table is the
-- authoritative record so an opt-out or a hard bounce survives a restart.
-- `email` holds the normalized gate address (an email for the email channel,
-- a phone number for WhatsApp/SMS) so the unique key matches the send gate.

CREATE TABLE IF NOT EXISTS marketing_suppressions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    branch_id UUID NOT NULL,
    email TEXT NOT NULL,
    channel TEXT NOT NULL,
    reason TEXT NOT NULL DEFAULT 'unsubscribed',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS marketing_suppressions_branch_channel_email_idx
    ON marketing_suppressions (branch_id, channel, email);

CREATE INDEX IF NOT EXISTS marketing_suppressions_branch_email_idx
    ON marketing_suppressions (branch_id, email);
