-- Marketing lists become isolated stores: campaign sends read marketing_contacts
-- (per-list rows) only. CRM deletions must never cascade into list membership or
-- send history, so the legacy FK from marketing_list_contacts -> crm_contacts is
-- dropped; the join table remains a best-effort display link (CRM-side only).
ALTER TABLE marketing_list_contacts DROP CONSTRAINT IF EXISTS marketing_list_contacts_contact_id_fkey;
