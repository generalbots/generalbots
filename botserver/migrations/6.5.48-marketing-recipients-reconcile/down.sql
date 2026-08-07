-- 6.5.48-marketing-recipients-reconcile (rollback)
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS clicked_at;
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS opened_at;
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS name;
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS email;
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS list_id;
ALTER TABLE marketing_recipients DROP COLUMN IF EXISTS branch_id;