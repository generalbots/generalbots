DROP INDEX IF EXISTS idx_account_sync_items_unique;
DROP INDEX IF EXISTS idx_account_sync_items_embedding;
DROP INDEX IF EXISTS idx_account_sync_items_date;
DROP INDEX IF EXISTS idx_account_sync_items_type;
DROP INDEX IF EXISTS idx_account_sync_items_account;
DROP TABLE IF EXISTS account_sync_items;

DROP INDEX IF EXISTS idx_session_account_assoc_unique;
DROP INDEX IF EXISTS idx_session_account_assoc_active;
DROP INDEX IF EXISTS idx_session_account_assoc_account;
DROP INDEX IF EXISTS idx_session_account_assoc_session;
DROP TABLE IF EXISTS session_account_associations;

DROP INDEX IF EXISTS idx_connected_accounts_bot_email;
DROP INDEX IF EXISTS idx_connected_accounts_status;
DROP INDEX IF EXISTS idx_connected_accounts_provider;
DROP INDEX IF EXISTS idx_connected_accounts_email;
DROP INDEX IF EXISTS idx_connected_accounts_user_id;
DROP INDEX IF EXISTS idx_connected_accounts_bot_id;
DROP TABLE IF EXISTS connected_accounts;
