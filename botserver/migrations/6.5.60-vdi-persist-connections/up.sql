-- VDI saved connections are stored in desktop_connections (created by
-- 6.4.2-01-desktop-vdi / 6.5.01-desktop-vdi). This migration backfills the
-- columns the app actually persists (name/host/port/protocol/auth_type) in
-- case an older schema variant is present.
--
-- No default connection is hardcoded here: the host/port/name come from
-- Vault at `secret/gbo/vdi` (fields default-host/default-port/default-name),
-- read at runtime, so no internal infrastructure addresses are committed
-- to the repository.

DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'desktop_connections' AND column_name = 'name') THEN
        ALTER TABLE desktop_connections ADD COLUMN name VARCHAR(255) NOT NULL DEFAULT 'Desktop';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'desktop_connections' AND column_name = 'host') THEN
        ALTER TABLE desktop_connections ADD COLUMN host VARCHAR(255) NOT NULL DEFAULT '';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'desktop_connections' AND column_name = 'port') THEN
        ALTER TABLE desktop_connections ADD COLUMN port INTEGER NOT NULL DEFAULT 5900;
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'desktop_connections' AND column_name = 'protocol') THEN
        ALTER TABLE desktop_connections ADD COLUMN protocol VARCHAR(10) NOT NULL DEFAULT 'vnc';
    END IF;
    IF NOT EXISTS (SELECT 1 FROM information_schema.columns
                   WHERE table_name = 'desktop_connections' AND column_name = 'auth_type') THEN
        ALTER TABLE desktop_connections ADD COLUMN auth_type VARCHAR(20);
    END IF;
END $$;
