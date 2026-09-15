-- #1386 follow-up — the launcher lists only production ("drive"-origin)
-- bots. Bot-kind Vibe projects are test bots: they are reachable through
-- the Vibe workbench (Run window / Chat button) and /chat/{slug}, never as
-- desktop launcher tiles. `origin` marks where the bot row came from; the
-- default keeps every pre-existing bot launcher-visible.
ALTER TABLE bots ADD COLUMN IF NOT EXISTS origin VARCHAR(16) NOT NULL DEFAULT 'drive';
