-- 6.5.55-bot-config-unique
-- ConfigManager.write_db_value upserts with ON CONFLICT (bot_id, config_key),
-- but bot_configuration had no unique constraint on that pair (PK is id),
-- so every drive_monitor config.csv sync failed with
-- "there is no unique or exclusion constraint matching the ON CONFLICT specification".
-- Dedup existing rows (keep newest) and add the unique index.

DELETE FROM bot_configuration a
USING bot_configuration b
WHERE a.id <> b.id
  AND a.bot_id = b.bot_id
  AND a.config_key = b.config_key
  AND (a.updated_at < b.updated_at OR (a.updated_at = b.updated_at AND a.id < b.id));

CREATE UNIQUE INDEX IF NOT EXISTS uq_bot_configuration_bot_key
    ON bot_configuration(bot_id, config_key);
