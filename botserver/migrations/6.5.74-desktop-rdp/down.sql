DROP INDEX IF EXISTS idx_desktop_connections_vault_path;

ALTER TABLE desktop_connections DROP COLUMN IF EXISTS secrets_vault_path;
ALTER TABLE desktop_connections DROP COLUMN IF EXISTS rdp_domain;
