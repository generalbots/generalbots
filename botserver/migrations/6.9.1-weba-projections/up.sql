-- #2538 — read-only projections of the durable WebA snapshot.
--
-- The builder persists one atomic JSONB snapshot per branch in `weba_snapshots`
-- so a publish cannot leave apps and pages half-written. These views expose the
-- three entities the site-builder contract names, with the same branch_id
-- tenant boundary, without duplicating state that could drift. They are
-- intentionally read-only: mutations go through the WebA API, which rewrites
-- the snapshot and refreshes all three views in one transaction.

CREATE OR REPLACE VIEW weba_sites AS
SELECT s.branch_id,
       (app.value->>'id')::uuid        AS id,
       app.value->>'name'              AS name,
       app.value->>'slug'              AS slug,
       app.value->>'status'            AS status,
       app.value->'config'             AS config,
       s.updated_at
FROM weba_snapshots s
CROSS JOIN LATERAL jsonb_array_elements(COALESCE(s.payload->'apps', '[]'::jsonb)) AS app(value);

CREATE OR REPLACE VIEW weba_pages AS
SELECT s.branch_id,
       (page.value->>'id')::uuid       AS id,
       (page.value->>'app_id')::uuid   AS site_id,
       page.value->>'path'             AS path,
       page.value->>'title'            AS title,
       page.value->>'content'          AS content,
       (page.value->>'is_index')::bool AS is_index,
       s.updated_at
FROM weba_snapshots s
CROSS JOIN LATERAL jsonb_array_elements(COALESCE(s.payload->'pages', '[]'::jsonb)) AS page(value);

CREATE OR REPLACE VIEW weba_form_submissions AS
SELECT s.branch_id,
       (sub.value->>'id')::uuid                    AS id,
       (sub.value->>'site_id')::uuid               AS site_id,
       sub.value->>'slug'                          AS slug,
       sub.value->>'email'                         AS email,
       sub.value->>'source'                        AS source,
       (sub.value->>'created_at')::timestamptz     AS created_at,
       (sub.value->>'updated_at')::timestamptz     AS updated_at
FROM weba_snapshots s
CROSS JOIN LATERAL jsonb_array_elements(COALESCE(s.payload->'submissions', '[]'::jsonb)) AS sub(value);
