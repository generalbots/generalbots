-- 6.5.44-user-org-bindings
-- Fixes issue #736: user_organizations is empty in production, so no
-- user -> org binding exists and tenant resolution falls back to ambiguous
-- "first default bot" hacks. Idempotently backfill bindings from the
-- verified identities we already hold:
--   * users bound to a CRM contact (crm_contacts.email = users.email) get the
--     org owning their contact's branch.
-- Row ids are deterministic (md5 uuid) so re-runs are no-ops.

INSERT INTO public.user_organizations (id, user_id, org_id, role, is_default, joined_at)
SELECT
    md5('uo:' || u.id::text || ':' || br.org_id::text)::uuid AS id,
    u.id                                    AS user_id,
    br.org_id                               AS org_id,
    CASE WHEN c.status = 'admin' THEN 'admin' ELSE 'member' END AS role,
    false                                   AS is_default,
    now()                                   AS joined_at
FROM public.users u
JOIN public.crm_contacts c   ON c.email = u.email
JOIN public.branches br      ON br.id = c.branch_id
WHERE c.branch_id IS NOT NULL
  AND br.org_id IS NOT NULL
ON CONFLICT (user_id, org_id) DO NOTHING;

-- Ensure each user has exactly one default binding: the earliest one. This
-- gives the deterministic context resolver a unique (user -> org -> branch)
-- path (issue #736, acceptance 4).
WITH ranked AS (
    SELECT user_id, org_id,
           ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY joined_at ASC) AS rn
    FROM public.user_organizations
)
UPDATE public.user_organizations uo
SET is_default = true
FROM ranked r
WHERE uo.user_id = r.user_id AND uo.org_id = r.org_id AND r.rn = 1;

UPDATE public.user_organizations SET is_default = false
WHERE (user_id, id) NOT IN (
    SELECT user_id, id
    FROM (
        SELECT id, user_id,
               ROW_NUMBER() OVER (PARTITION BY user_id ORDER BY joined_at ASC) AS rn
        FROM public.user_organizations
    ) rnk
    WHERE rnk.rn = 1
);