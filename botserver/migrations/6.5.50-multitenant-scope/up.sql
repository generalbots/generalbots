-- 6.5.50-multitenant-scope
-- Issue #734: multi-tenant isolation. Every business table in the HIGH/MED
-- risk app crates gains a branch_id tenant column so reads/writes can be
-- constrained to the caller's branch. Existing rows default to the nil
-- (legacy global) branch and keep working; new writes must set a real branch.
--
-- This migration is idempotent: each column already present is skipped.

-- bothr (HR)
ALTER TABLE IF EXISTS hr_employees       ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS hr_recruitment     ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS hr_attendance      ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS hr_review_cycles   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS hr_goals           ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- bottax: raw
ALTER TABLE IF EXISTS brazil_nfe         ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS brazil_nfse        ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS brazil_cte         ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS brazil_sped        ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botvision
ALTER TABLE IF EXISTS vision_analysis    ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- boterp
ALTER TABLE IF EXISTS erp_financial      ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS erp_inventory      ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS erp_procurement    ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS erp_branches       ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botintegrations
ALTER TABLE IF EXISTS integrations_connectors ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS integrations_etl_jobs   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botsales
ALTER TABLE IF EXISTS sales_deals        ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS sales_contacts     ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS sales_activities   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botminutes
ALTER TABLE IF EXISTS minutes_meetings   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS minutes_transcripts ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS minutes_documents  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- bottemplates
ALTER TABLE IF EXISTS app_templates      ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS app_template_deploys ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botitsm
ALTER TABLE IF EXISTS itsm_incidents         ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS itsm_service_requests  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS itsm_cmdb              ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS itsm_kb                ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botpos
ALTER TABLE IF EXISTS pos_products       ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS pos_orders         ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- bothandoff
ALTER TABLE IF EXISTS handoff_queue          ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS conversation_analytics ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS handoff_channels       ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS conversation_ratings   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botkyc
ALTER TABLE IF EXISTS identity_kyc_workflows ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS identity_signatures    ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS identity_certificates  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- bottimeclock
ALTER TABLE IF EXISTS timeclock_events   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS timeclock_records  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS timeclock_overtime ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS timeclock_reports  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botbanking
ALTER TABLE IF EXISTS banking_transactions       ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS banking_platforms           ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS banking_reconcile_results   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS banking_reports             ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botinventory + botgl (State-based; erp migration created them with bot_id only)
ALTER TABLE IF EXISTS gl_accounts         ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS gl_journal_entries  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS gl_journal_lines    ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS inventory_items     ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS purchase_orders     ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS purchase_order_items ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botm365 (org columns already present, add branch for uniformity)
ALTER TABLE IF EXISTS m365_sharepoint_items ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS m365_calendar_events  ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS m365_onedrive_files   ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';
ALTER TABLE IF EXISTS oauth_microsoft_settings ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- botresearch (MED: research_searches global history)
ALTER TABLE IF EXISTS research_searches ADD COLUMN IF NOT EXISTS branch_id uuid NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000';

-- indexes on the tenant column for every added column
CREATE INDEX IF NOT EXISTS idx_hr_employees_branch         ON hr_employees(branch_id);
CREATE INDEX IF NOT EXISTS idx_hr_recruitment_branch       ON hr_recruitment(branch_id);
CREATE INDEX IF NOT EXISTS idx_hr_attendance_branch        ON hr_attendance(branch_id);
CREATE INDEX IF NOT EXISTS idx_hr_review_cycles_branch     ON hr_review_cycles(branch_id);
CREATE INDEX IF NOT EXISTS idx_hr_goals_branch             ON hr_goals(branch_id);
CREATE INDEX IF NOT EXISTS idx_brazil_nfe_branch           ON brazil_nfe(branch_id);
CREATE INDEX IF NOT EXISTS idx_brazil_nfse_branch          ON brazil_nfse(branch_id);
CREATE INDEX IF NOT EXISTS idx_brazil_cte_branch           ON brazil_cte(branch_id);
CREATE INDEX IF NOT EXISTS idx_brazil_sped_branch          ON brazil_sped(branch_id);
CREATE INDEX IF NOT EXISTS idx_vision_analysis_branch      ON vision_analysis(branch_id);
CREATE INDEX IF NOT EXISTS idx_erp_financial_branch        ON erp_financial(branch_id);
CREATE INDEX IF NOT EXISTS idx_erp_inventory_branch        ON erp_inventory(branch_id);
CREATE INDEX IF NOT EXISTS idx_erp_procurement_branch      ON erp_procurement(branch_id);
CREATE INDEX IF NOT EXISTS idx_integrations_connectors_branch ON integrations_connectors(branch_id);
CREATE INDEX IF NOT EXISTS idx_integrations_etl_jobs_branch ON integrations_etl_jobs(branch_id);
CREATE INDEX IF NOT EXISTS idx_sales_deals_branch          ON sales_deals(branch_id);
CREATE INDEX IF NOT EXISTS idx_sales_contacts_branch       ON sales_contacts(branch_id);
CREATE INDEX IF NOT EXISTS idx_sales_activities_branch     ON sales_activities(branch_id);
CREATE INDEX IF NOT EXISTS idx_minutes_meetings_branch     ON minutes_meetings(branch_id);
CREATE INDEX IF NOT EXISTS idx_minutes_transcripts_branch  ON minutes_transcripts(branch_id);
CREATE INDEX IF NOT EXISTS idx_minutes_documents_branch    ON minutes_documents(branch_id);
CREATE INDEX IF NOT EXISTS idx_app_templates_branch        ON app_templates(branch_id);
CREATE INDEX IF NOT EXISTS idx_app_template_deploys_branch ON app_template_deploys(branch_id);
CREATE INDEX IF NOT EXISTS idx_itsm_incidents_branch       ON itsm_incidents(branch_id);
CREATE INDEX IF NOT EXISTS idx_itsm_service_requests_branch ON itsm_service_requests(branch_id);
CREATE INDEX IF NOT EXISTS idx_itsm_cmdb_branch            ON itsm_cmdb(branch_id);
CREATE INDEX IF NOT EXISTS idx_itsm_kb_branch              ON itsm_cmdb(branch_id);
CREATE INDEX IF NOT EXISTS idx_pos_products_branch         ON pos_products(branch_id);
CREATE INDEX IF NOT EXISTS idx_pos_orders_branch           ON pos_orders(branch_id);
CREATE INDEX IF NOT EXISTS idx_handoff_queue_branch        ON handoff_queue(branch_id);
CREATE INDEX IF NOT EXISTS idx_conversation_analytics_branch ON conversation_analytics(branch_id);
CREATE INDEX IF NOT EXISTS idx_handoff_channels_branch     ON handoff_channels(branch_id);
CREATE INDEX IF NOT EXISTS idx_conversation_ratings_branch ON conversation_ratings(branch_id);
CREATE INDEX IF NOT EXISTS idx_identity_kyc_workflows_branch ON identity_kyc_workflows(branch_id);
CREATE INDEX IF NOT EXISTS idx_identity_signatures_branch  ON identity_signatures(branch_id);
CREATE INDEX IF NOT EXISTS idx_identity_certificates_branch ON identity_certificates(branch_id);
CREATE INDEX IF NOT EXISTS idx_timeclock_events_branch     ON timeclock_events(branch_id);
CREATE INDEX IF NOT EXISTS idx_timeclock_records_branch    ON timeclock_records(branch_id);
CREATE INDEX IF NOT EXISTS idx_timeclock_overtime_branch   ON timeclock_overtime(branch_id);
CREATE INDEX IF NOT EXISTS idx_timeclock_reports_branch    ON timeclock_reports(branch_id);
CREATE INDEX IF NOT EXISTS idx_banking_transactions_branch ON banking_transactions(branch_id);
CREATE INDEX IF NOT EXISTS idx_banking_platforms_branch    ON banking_platforms(branch_id);
CREATE INDEX IF NOT EXISTS idx_banking_reconcile_results_branch ON banking_reconcile_results(branch_id);
CREATE INDEX IF NOT EXISTS idx_banking_reports_branch      ON banking_reports(branch_id);
CREATE INDEX IF NOT EXISTS idx_gl_accounts_branch         ON gl_accounts(branch_id);
CREATE INDEX IF NOT EXISTS idx_gl_journal_entries_branch  ON gl_journal_entries(branch_id);
CREATE INDEX IF NOT EXISTS idx_gl_journal_lines_branch    ON gl_journal_lines(branch_id);
CREATE INDEX IF NOT EXISTS idx_inventory_items_branch     ON inventory_items(branch_id);
CREATE INDEX IF NOT EXISTS idx_purchase_orders_branch     ON purchase_orders(branch_id);
CREATE INDEX IF NOT EXISTS idx_purchase_order_items_branch ON purchase_order_items(branch_id);
CREATE INDEX IF NOT EXISTS idx_m365_sharepoint_items_branch ON m365_sharepoint_items(branch_id);
CREATE INDEX IF NOT EXISTS idx_m365_calendar_events_branch ON m365_calendar_events(branch_id);
CREATE INDEX IF NOT EXISTS idx_m365_onedrive_files_branch  ON m365_onedrive_files(branch_id);
CREATE INDEX IF NOT EXISTS idx_oauth_microsoft_settings_branch ON oauth_microsoft_settings(branch_id);
CREATE INDEX IF NOT EXISTS idx_research_searches_branch ON research_searches(branch_id);