-- #2452 criterion 4 — caller address on the cross-app activity log.
--
-- A shared-link read and a document export append `view` / `export` events to
-- `collab_activity` (the same table the share dialog and every other collab
-- mutation write to). For enterprise compliance those rows must also carry the
-- address the request came from, so readers of GET /api/activity can see
-- "who read what, from where" in the very same timeline.
--
-- The column is deliberately nullable: the request layer may not be able to
-- resolve an address (e.g. a proxy without a trusted forwarding header), and
-- every row written before this migration has no address at all. `VARCHAR(64)`
-- is long enough for IPv4, IPv6 and a sanitized header value.

ALTER TABLE collab_activity
    ADD COLUMN IF NOT EXISTS ip VARCHAR(64);
