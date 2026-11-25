-- #2452 — honor `allow_download` when a share link is redeemed.
--
-- `resource_links` is the persistence behind POST /api/collab/links. Until now
-- a link could grant a role and an expiry but carried no download policy, so
-- every role with source capability received the editable document body even
-- when the share dialog had been configured to disable downloads.
--
-- The column is deliberately nullable with a TRUE default: NULL means "use
-- the legacy behaviour" (downloads allowed) and existing rows are filled with
-- TRUE, so no link minted before this migration changes meaning and no data is
-- lost. Only an explicit FALSE denies the source bytes.

ALTER TABLE resource_links
    ADD COLUMN IF NOT EXISTS allow_download BOOLEAN DEFAULT TRUE;

COMMENT ON COLUMN resource_links.allow_download IS
    'When FALSE, a shared-link read never emits editable source bytes. NULL or TRUE keeps the legacy allow-download behaviour.';
