ALTER TABLE connectors DROP COLUMN IF EXISTS secrets_vault_path;
ALTER TABLE connectors DROP COLUMN IF EXISTS last_test_at;
ALTER TABLE connectors DROP COLUMN IF EXISTS last_test_status;
