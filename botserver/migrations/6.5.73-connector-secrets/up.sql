-- Connector secrets hardening: credentials live in Vault, never in the DB or
-- the UI. `secrets_vault_path` points at the Vault KV path holding the
-- connector's sensitive fields (api_key, oauth2_client_secret, passwords).
-- `last_test_at` / `last_test_status` record the most recent connectivity
-- check for the connector list health column.

ALTER TABLE connectors ADD COLUMN IF NOT EXISTS secrets_vault_path VARCHAR(500);
ALTER TABLE connectors ADD COLUMN IF NOT EXISTS last_test_at TIMESTAMPTZ;
ALTER TABLE connectors ADD COLUMN IF NOT EXISTS last_test_status VARCHAR(20);

CREATE INDEX IF NOT EXISTS idx_connectors_secrets_path ON connectors(secrets_vault_path);
