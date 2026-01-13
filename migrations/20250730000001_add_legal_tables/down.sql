DROP INDEX IF EXISTS idx_consent_history_consent;
DROP INDEX IF EXISTS idx_consent_history_created;
DROP INDEX IF EXISTS idx_cookie_consents_user;
DROP INDEX IF EXISTS idx_cookie_consents_session;
DROP INDEX IF EXISTS idx_cookie_consents_org_bot;
DROP INDEX IF EXISTS idx_legal_documents_org_bot;
DROP INDEX IF EXISTS idx_legal_documents_slug;
DROP INDEX IF EXISTS idx_legal_documents_type;

DROP TABLE IF EXISTS consent_history;
DROP TABLE IF EXISTS cookie_consents;
DROP TABLE IF EXISTS legal_documents;
