ALTER TABLE marketing_list_contacts
  ADD CONSTRAINT marketing_list_contacts_contact_id_fkey
  FOREIGN KEY (contact_id) REFERENCES crm_contacts(id) ON DELETE CASCADE;
