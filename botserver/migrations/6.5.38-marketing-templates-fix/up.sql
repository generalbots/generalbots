-- Reconcile marketing_templates with the Rust schema:
--   * add missing template_type column (VARCHAR NOT NULL DEFAULT 'email')
--   * make org_id nullable (branch-scoped model uses branch_id)
ALTER TABLE marketing_templates ADD COLUMN IF NOT EXISTS template_type VARCHAR(255) NOT NULL DEFAULT 'email';
ALTER TABLE marketing_templates ALTER COLUMN org_id DROP NOT NULL;
UPDATE marketing_templates SET template_type = channel WHERE template_type = 'email' AND channel IS NOT NULL AND channel <> '';
CREATE INDEX IF NOT EXISTS idx_marketing_templates_template_type ON marketing_templates(template_type);
