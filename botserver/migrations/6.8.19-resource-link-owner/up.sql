-- #2253 — remember the storage owner of a shared resource.
--
-- Office documents are stored under `users/{owner}/docs/{id}.html` (#2135),
-- never under a shared namespace. A public link is redeemed by a reader with
-- no session, so it cannot derive `owner` from the request: the owner is
-- captured when the link is minted and read back when it is redeemed.
--
-- NULL means the resource type does not need an owner-scoped lookup (sheets,
-- drive files and other resources address their storage by id alone).

ALTER TABLE resource_links
    ADD COLUMN IF NOT EXISTS owner_id VARCHAR(255);

COMMENT ON COLUMN resource_links.owner_id IS
    'Storage owner captured at mint time for owner-scoped resources (docs/slides/paper).';
