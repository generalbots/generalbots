DROP INDEX IF EXISTS idx_compliance_evidence_check;
DROP INDEX IF EXISTS idx_compliance_evidence_org_bot;
DROP INDEX IF EXISTS idx_audit_log_created;
DROP INDEX IF EXISTS idx_audit_log_resource;
DROP INDEX IF EXISTS idx_audit_log_user;
DROP INDEX IF EXISTS idx_audit_log_org_bot;
DROP INDEX IF EXISTS idx_compliance_issues_status;
DROP INDEX IF EXISTS idx_compliance_issues_severity;
DROP INDEX IF EXISTS idx_compliance_issues_check;
DROP INDEX IF EXISTS idx_compliance_issues_org_bot;
DROP INDEX IF EXISTS idx_compliance_checks_status;
DROP INDEX IF EXISTS idx_compliance_checks_framework;
DROP INDEX IF EXISTS idx_compliance_checks_org_bot;

DROP TABLE IF EXISTS compliance_evidence;
DROP TABLE IF EXISTS audit_log;
DROP TABLE IF EXISTS compliance_issues;
DROP TABLE IF EXISTS compliance_checks;
