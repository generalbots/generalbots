DROP INDEX IF EXISTS idx_marketing_templates_template_type;
ALTER TABLE marketing_templates ALTER COLUMN org_id SET NOT NULL;
ALTER TABLE marketing_templates DROP COLUMN IF EXISTS template_type;
