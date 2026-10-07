-- The prior state cannot be recovered exactly (a bucket name is not a bot
-- name), so this only documents intent: re-running 6.6.0's backfill would
-- restore the bucket-in-bot_name form, which is the bug being corrected.
SELECT 1;
