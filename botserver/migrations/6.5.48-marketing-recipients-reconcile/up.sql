-- 6.5.48-marketing-recipients-reconcile
-- The Diesel model (crates/botmarketing/src/schema.rs) reads email, name,
-- branch_id, list_id and opened_at/clicked_at from marketing_recipients, but
-- migrations 6.2.4 and 6.5.15.1 created the table without them. The durable
-- worker (#731) selects email/name so it would fail with
-- "column marketing_recipients.email does not exist".
--
-- This migration reconciles drifted databases to the compiled model, idempotently.

ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS list_id uuid;
ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS email varchar(255);
ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS name varchar(255);
ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS opened_at timestamptz;
ALTER TABLE marketing_recipients ADD COLUMN IF NOT EXISTS clicked_at timestamptz;

-- Backfill email/name from the owning contact row where the contact exists.
UPDATE marketing_recipients mr
SET email = COALESCE(mr.email, c.email),
    name  = COALESCE(mr.name, c.first_name)
FROM crm_contacts c
WHERE mr.contact_id IS NOT NULL AND c.id = mr.contact_id;

-- The concrete table uses jsonb for response, the model maps it to Text.
-- Keep the concrete jsonb; selectors cast as needed. Ensure a nullable default.
ALTER TABLE marketing_recipients ALTER COLUMN channel DROP NOT NULL;
ALTER TABLE marketing_recipients ALTER COLUMN status SET DEFAULT 'pending';

CREATE INDEX IF NOT EXISTS idx_marketing_recipients_list ON marketing_recipients(list_id);