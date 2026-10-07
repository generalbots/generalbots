-- Corrective migration for 6.6.0-bot-scripts.
--
-- The 6.6.0 backfill stored the *bucket* in `bot_scripts.bot_name`
-- (split_part(file_path,'/',1) => 'e2e-alpha.gbai') instead of the bot. The
-- writer paths in Rust derive the bot from segment 2, so the backfilled rows
-- disagreed with every row added since, and the repair path that requeues a
-- script whose .ast vanished would requeue it under the wrong name.

UPDATE bot_scripts
   SET bot_name = trim(trailing '.gbdialog' FROM split_part(script_path, '/', 2)),
       updated_at = NOW()
 WHERE script_path ~ '^[a-z0-9_-]+\.gbai/[^/]+\.gbdialog/';
