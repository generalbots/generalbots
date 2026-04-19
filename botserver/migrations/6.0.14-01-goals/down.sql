DROP INDEX IF EXISTS idx_okr_activity_created;
DROP INDEX IF EXISTS idx_okr_activity_user;
DROP INDEX IF EXISTS idx_okr_activity_key_result;
DROP INDEX IF EXISTS idx_okr_activity_objective;
DROP INDEX IF EXISTS idx_okr_activity_org_bot;

DROP INDEX IF EXISTS idx_okr_comments_parent;
DROP INDEX IF EXISTS idx_okr_comments_key_result;
DROP INDEX IF EXISTS idx_okr_comments_objective;
DROP INDEX IF EXISTS idx_okr_comments_org_bot;

DROP INDEX IF EXISTS idx_okr_templates_system;
DROP INDEX IF EXISTS idx_okr_templates_category;
DROP INDEX IF EXISTS idx_okr_templates_org_bot;

DROP INDEX IF EXISTS idx_okr_alignments_parent;
DROP INDEX IF EXISTS idx_okr_alignments_child;
DROP INDEX IF EXISTS idx_okr_alignments_org_bot;

DROP INDEX IF EXISTS idx_okr_checkins_created;
DROP INDEX IF EXISTS idx_okr_checkins_user;
DROP INDEX IF EXISTS idx_okr_checkins_key_result;
DROP INDEX IF EXISTS idx_okr_checkins_org_bot;

DROP INDEX IF EXISTS idx_okr_key_results_due_date;
DROP INDEX IF EXISTS idx_okr_key_results_status;
DROP INDEX IF EXISTS idx_okr_key_results_owner;
DROP INDEX IF EXISTS idx_okr_key_results_objective;
DROP INDEX IF EXISTS idx_okr_key_results_org_bot;

DROP INDEX IF EXISTS idx_okr_objectives_status;
DROP INDEX IF EXISTS idx_okr_objectives_period;
DROP INDEX IF EXISTS idx_okr_objectives_parent;
DROP INDEX IF EXISTS idx_okr_objectives_owner;
DROP INDEX IF EXISTS idx_okr_objectives_org_bot;

DROP TABLE IF EXISTS okr_activity_log;
DROP TABLE IF EXISTS okr_comments;
DROP TABLE IF EXISTS okr_templates;
DROP TABLE IF EXISTS okr_alignments;
DROP TABLE IF EXISTS okr_checkins;
DROP TABLE IF EXISTS okr_key_results;
DROP TABLE IF EXISTS okr_objectives;
