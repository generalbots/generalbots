-- Email v1: Unified Inbox - email_messages table for cross-account indexing

CREATE TABLE IF NOT EXISTS email_messages (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    account_id UUID NOT NULL REFERENCES user_email_accounts(id) ON DELETE CASCADE,
    message_id_header VARCHAR(512),
    in_reply_to VARCHAR(512),
    subject TEXT NOT NULL DEFAULT '',
    normalized_subject TEXT NOT NULL DEFAULT '',
    from_address VARCHAR(512) NOT NULL,
    to_addresses TEXT,
    body_text TEXT,
    body_html TEXT,
    has_attachments BOOLEAN DEFAULT false,
    folder VARCHAR(255) NOT NULL DEFAULT 'INBOX',
    uid BIGINT NOT NULL,
    flags JSONB DEFAULT '[]'::jsonb,
    is_read BOOLEAN DEFAULT false,
    is_flagged BOOLEAN DEFAULT false,
    received_at TIMESTAMPTZ NOT NULL,
    synced_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_email_messages_account ON email_messages(account_id);
CREATE INDEX IF NOT EXISTS idx_email_messages_received ON email_messages(received_at DESC);
CREATE INDEX IF NOT EXISTS idx_email_messages_folder ON email_messages(account_id, folder);
CREATE INDEX IF NOT EXISTS idx_email_messages_read ON email_messages(is_read) WHERE is_read = false;
CREATE INDEX IF NOT EXISTS idx_email_messages_flagged ON email_messages(is_flagged) WHERE is_flagged = true;
CREATE INDEX IF NOT EXISTS idx_email_messages_thread ON email_messages(normalized_subject);
CREATE INDEX IF NOT EXISTS idx_email_messages_search ON email_messages USING gin(
    to_tsvector('english', coalesce(subject, '') || ' ' || coalesce(body_text, '') || ' ' || coalesce(from_address, ''))
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_email_messages_unique_sync ON email_messages(account_id, uid, folder);
