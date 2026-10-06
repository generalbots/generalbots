-- #1475 — bot_scripts compile queue.
--
-- The backfilled rows are a copy of `drive_files` state, so dropping the table
-- loses no source of truth. `drive_files` itself is intentionally left in
-- place: it remains the Drive inventory used by search, KB indexing and the
-- media paths, and #1474 retires only its role as the *compile queue*. Any
-- script still queued by the old pipeline is rediscovered from `drive_files`
-- on the next deploy, so a rollback to the old compiler is non-destructive.

DROP TABLE IF EXISTS bot_scripts;