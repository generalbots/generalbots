-- #1475 — replace `drive_files.etag` as the compile queue with a real table.
--
-- Problem
--   `DriveCompiler::check_and_compile` used `drive_files` as its work queue:
--     SELECT ... FROM drive_files
--      WHERE file_type = 'bas' AND file_path LIKE '%.gbdialog/%'
--   The leading `%` can never use a btree index, and 6.5.25 dropped
--   `idx_drive_files_type`, so every 5 s tick sequentially scanned the whole
--   table (10k+ rows in prod, of which the KB/media/shared files are
--   irrelevant). Two further defects rode along:
--     * `drive_files.etag` is written by two callers with different meanings —
--       a git commit sha (git_bot_monitor::mark_for_compile) and an S3 ETag
--       (botdrive DriveMonitor) — so each can clobber the other's version.
--     * the compiler records the new version only after a *successful*
--       compile, so a permanently broken script stays "changed" forever and
--       is retried on every single tick.
--
-- Design
--   `bot_scripts` is the queue, not an inventory. `dirty` is a boolean with a
--   partial index, so the compile loop performs an index-only scan of a tiny
--   set instead of scanning `drive_files`. `source_version` (what the source
--   currently is) is separated from `compiled_version` (what last compiled),
--   which is what makes a failed script stop spinning. Leases let more than
--   one compiler tick claim disjoint work. `fail_count`/`last_failed_at`
--   drive the retry backoff.

CREATE TABLE IF NOT EXISTS bot_scripts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    -- Scope: a script belongs to a bot on a branch. `branch_id` is nullable
    -- so a Drive-discovered bot with no branch row still gets a queue entry.
    branch_id UUID,
    bot_name TEXT NOT NULL,
    -- Object key as it appears in `drive_files.file_path`, e.g.
    -- 'mybranch.gbai/mybranch.gbdialog/tools/media.bas'.
    script_path TEXT NOT NULL,
    -- 'git' when the version is a commit sha, 'drive' when it is an S3 ETag.
    -- Keeping the two apart stops the writers from overwriting each other.
    source_kind VARCHAR(16) NOT NULL DEFAULT 'drive',
    source_version TEXT NOT NULL,
    -- Last version that compiled cleanly. NULL until the first success.
    compiled_version TEXT,
    dirty BOOLEAN NOT NULL DEFAULT TRUE,
    fail_count INTEGER NOT NULL DEFAULT 0,
    last_failed_at TIMESTAMPTZ,
    last_error TEXT,
    -- Lease held by the compiler tick currently processing this script.
    lease_owner TEXT,
    lease_expires_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT bot_scripts_scope_path_unique UNIQUE (branch_id, script_path)
);

-- The compile queue. Partial on `dirty`, so the index holds only pending work
-- and stays tiny no matter how many scripts are compiled. `lease_expires_at`
-- is in the key so the claim query can filter expired leases in the index.
CREATE INDEX IF NOT EXISTS idx_bot_scripts_dirty
    ON bot_scripts (dirty, lease_expires_at)
    WHERE dirty;

-- Per-bot draining, and the retry backoff sweep, both index-only.
CREATE INDEX IF NOT EXISTS idx_bot_scripts_branch_dirty
    ON bot_scripts (branch_id, dirty);
CREATE INDEX IF NOT EXISTS idx_bot_scripts_backoff
    ON bot_scripts (last_failed_at)
    WHERE fail_count > 0;

-- Backfill every script the old inventory-based queue knew about, so the
-- first tick after deploy still sees the full pending set. Everything starts
-- dirty with no `compiled_version`: the previous pipeline only recorded an
-- etag after a successful compile, so we cannot know which were already
-- compiled and re-running them once is the safe direction.
--
-- `/archive/` paths are excluded. The archive pass writes
-- `{bucket}/archive/{bot}-{stamp}/.gbdialog/...` copies of Drive sources that
-- are explicitly never read again (AGENTS.md: git-owned bots archive their
-- Drive `.gbdialog`/`.gbot` and never compile them). The old `LIKE
-- '%.gbdialog/%'` matched them because the substring still appears; a real
-- Drive bucket prefix (`{branch}.gbai/{bot}.gbdialog/`) is required instead.
INSERT INTO bot_scripts (
    branch_id, bot_name, script_path, source_kind, source_version, dirty
)
SELECT
    df.branch_id,
    split_part(df.file_path, '/', 1),
    df.file_path,
    'drive',
    COALESCE(df.etag, ''),
    TRUE
FROM drive_files df
WHERE df.file_type = 'bas'
  AND df.file_path LIKE '%.gbdialog/%'
  AND df.file_path NOT LIKE '%/archive/%'
  AND df.file_path ~ '^[a-z0-9_-]+\.gbai/[^/]+\.gbdialog/'
  AND df.etag IS NOT NULL
ON CONFLICT (branch_id, script_path) DO NOTHING;