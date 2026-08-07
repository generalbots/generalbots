-- 6.5.44-user-org-bindings (down)
-- The binding rows are derived data; dropping them returns the DB to the
-- pre-migration state. Indexes on user_organizations are preserved.
DELETE FROM public.user_organizations
WHERE id IN (
    SELECT md5('uo:' || u.id::text || ':' || br.org_id::text)::uuid
    FROM public.users u
    JOIN public.crm_contacts c ON c.email = u.email
    JOIN public.branches br ON br.id = c.branch_id
    WHERE c.branch_id IS NOT NULL AND br.org_id IS NOT NULL
);