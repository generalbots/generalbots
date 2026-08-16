-- RDP protocol support for the desktop proxy.
--
-- `secrets_vault_path` points at the Vault KV2 path holding the target's
-- credentials (password / domain) for an RDP connection. Credentials are
-- written to Vault by the connect API and NEVER stored in the database or
-- echoed to the UI. `rdp_domain` (optional) is the Windows domain used for
-- NLA authentication at connect time.

ALTER TABLE desktop_connections ADD COLUMN IF NOT EXISTS secrets_vault_path VARCHAR(500);
ALTER TABLE desktop_connections ADD COLUMN IF NOT EXISTS rdp_domain VARCHAR(255);

CREATE INDEX IF NOT EXISTS idx_desktop_connections_vault_path
    ON desktop_connections(secrets_vault_path);
