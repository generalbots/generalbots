-- =============================================================================
-- General Bots Sample Data
--
-- Populates every suite app with realistic demo data so a logged-in user can
-- exercise the full product. All rows are scoped to the DEFAULT org + branch
-- and owned by a dedicated `sample.user` (UUIDv5) so that data isolation by
-- branch + user can be verified.
--
-- Run:  psql -h localhost -U <user> -d botserver -f sample.sql
--
-- Idempotent: safe to re-run. Sample rows are tagged and deleted on re-run.
-- =============================================================================

BEGIN;

-- ---------------------------------------------------------------------------
-- 0. Scoping constants
-- ---------------------------------------------------------------------------
DO $$
DECLARE
    v_org_id    uuid := 'cad9d36f-2e00-4d60-ad19-337c6a9e2f53';  -- "default" org
    v_branch_id uuid := 'a188d531-4abd-404f-bf95-e9302cb011be';  -- "default" branch
    v_bot_id    uuid := 'f47ac10b-58cc-4372-a567-0e02b2c3d480';  -- "default" bot
    v_user_id   uuid := '10000000-0000-0000-0000-000000000001';  -- sample user
    v_cal_id    uuid;
    v_contact_id uuid;
    v_account_id uuid;
    v_deal_id   uuid;
    v_ticket_id uuid;
    v_inv_id    uuid;
    v_quote_id  uuid;
    v_task_id   uuid;
    v_proj_id   uuid;
    v_team_id   uuid;
    v_dep_id    uuid;
    v_obj_id    uuid;
    v_ws_id     uuid;
    v_proj_research_id uuid;
    v_check_id  uuid;
BEGIN

-- Ensure the sample user exists (isolated owner for all sample rows).
INSERT INTO users (id, username, email, password_hash, is_active, created_at, updated_at)
VALUES (v_user_id, 'sample.user', 'sample@example.com', 'x', true, NOW(), NOW())
ON CONFLICT (id) DO NOTHING;

-- ---------------------------------------------------------------------------
-- 1. People app
-- ---------------------------------------------------------------------------
-- Departments
INSERT INTO people_departments (id, org_id, bot_id, name, description, code, is_active, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Engineering', 'Product engineering', 'ENG', true, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people_departments WHERE org_id = v_org_id AND code = 'ENG')
RETURNING id INTO v_dep_id;

INSERT INTO people_departments (id, org_id, bot_id, name, description, code, is_active, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Marketing', 'Demand generation', 'MKT', true, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people_departments WHERE org_id = v_org_id AND code = 'MKT');

-- Teams
INSERT INTO people_teams (id, org_id, bot_id, name, description, color, icon, is_active, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Platform', 'Core platform squad', '#3b82f6', 'rocket', true, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people_teams WHERE org_id = v_org_id AND name = 'Platform')
RETURNING id INTO v_team_id;

-- People
INSERT INTO people (id, org_id, bot_id, user_id, first_name, last_name, email, phone, mobile, job_title, department, office_location, hire_date, timezone, locale, skills, social_links, custom_fields, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_user_id, 'Alice', 'Sample', 'alice.sample@example.com', '+1 555 0100', '+1 555 0101', 'Software Engineer', 'Engineering', 'Remote', '2024-03-01', 'UTC', 'en-US', ARRAY['rust','sql'], '{"linkedin":"https://linkedin.com/in/alicesample"}', '{}', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people WHERE org_id = v_org_id AND email = 'alice.sample@example.com');

INSERT INTO people (id, org_id, bot_id, user_id, first_name, last_name, email, phone, mobile, job_title, department, office_location, hire_date, timezone, locale, skills, social_links, custom_fields, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, NULL, 'Bruno', 'Demo', 'bruno.demo@example.com', '+1 555 0200', NULL, 'Product Manager', 'Marketing', 'New York', '2023-06-15', 'America/New_York', 'en-US', ARRAY['strategy','analytics'], '{}', '{}', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people WHERE org_id = v_org_id AND email = 'bruno.demo@example.com');

INSERT INTO people (id, org_id, bot_id, user_id, first_name, last_name, email, phone, mobile, job_title, department, office_location, hire_date, timezone, locale, skills, social_links, custom_fields, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, NULL, 'Carla', 'Test', 'carla.test@example.com', '+55 11 90000-0000', NULL, 'Designer', 'Engineering', 'São Paulo', '2025-01-20', 'America/Sao_Paulo', 'pt-BR', ARRAY['figma','design-systems'], '{}', '{}', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people WHERE org_id = v_org_id AND email = 'carla.test@example.com');

-- Skills
INSERT INTO people_skills (id, org_id, bot_id, name, category, description, is_active, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Rust', 'Engineering', 'Systems programming', true, NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM people_skills WHERE org_id = v_org_id AND name = 'Rust');

-- ---------------------------------------------------------------------------
-- 2. CRM app
-- NOTE: migration 6.5.16 repurposed `org_id` in crm_contacts/accounts/deals and
-- billing_* to reference branches(id) (branch-id isolation). We therefore store
-- the branch id in `org_id` for those tables. Other CRM tables (leads,
-- opportunities, activities, notes) still reference organizations(org_id).
-- ---------------------------------------------------------------------------
-- Accounts
INSERT INTO crm_accounts (id, org_id, bot_id, name, industry, website, phone, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'Acme Corp', 'Technology', 'https://acme.example.com', '+1 555 1000', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM crm_accounts WHERE org_id = v_branch_id AND name = 'Acme Corp')
RETURNING id INTO v_account_id;

IF v_account_id IS NULL THEN
    SELECT id INTO v_account_id FROM crm_accounts WHERE org_id = v_branch_id AND name = 'Acme Corp' LIMIT 1;
END IF;

INSERT INTO crm_accounts (id, org_id, bot_id, name, industry, website, phone, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'Globex Inc', 'Retail', 'https://globex.example.com', '+1 555 2000', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM crm_accounts WHERE org_id = v_branch_id AND name = 'Globex Inc');

-- Contacts
INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, phone, mobile, company, job_title, source, status, tags, custom_fields, city, state, country, notes, owner_id, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'John', 'Doe', 'john.doe@acme.example.com', '+1 555 1100', NULL, 'Acme Corp', 'CTO', 'web', 'active', ARRAY['enterprise','warm'], '{}', 'Austin', 'TX', 'US', 'Primary technical contact', v_user_id, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM crm_contacts WHERE org_id = v_branch_id AND email = 'john.doe@acme.example.com')
RETURNING id INTO v_contact_id;

IF v_contact_id IS NULL THEN
    SELECT id INTO v_contact_id FROM crm_contacts WHERE org_id = v_branch_id AND email = 'john.doe@acme.example.com' LIMIT 1;
END IF;

INSERT INTO crm_contacts (id, org_id, bot_id, first_name, last_name, email, phone, mobile, company, job_title, source, status, tags, custom_fields, city, state, country, notes, owner_id, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'Jane', 'Roe', 'jane.roe@globex.example.com', '+1 555 2100', NULL, 'Globex Inc', 'Head of Ops', 'referral', 'active', ARRAY['retail','new'], '{}', 'Seattle', 'WA', 'US', NULL, v_user_id, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM crm_contacts WHERE org_id = v_branch_id AND email = 'jane.roe@globex.example.com');

-- Deals
INSERT INTO crm_deals (id, org_id, bot_id, contact_id, account_id, owner_id, title, name, description, value, currency, stage, probability, won, source, created_at, updated_at, branch_id)
VALUES
  (gen_random_uuid(), v_branch_id, v_bot_id, v_contact_id, v_account_id, v_user_id, 'Acme expansion', 'Acme expansion', 'Multi-seat enterprise expansion', 45000.0, 'USD', 'proposal', 60, false, 'web', NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_branch_id, v_bot_id, NULL, NULL, v_user_id, 'Globex pilot', 'Globex pilot', 'Retail pilot program', 12000.0, 'USD', 'discovery', 20, false, 'referral', NOW(), NOW(), v_branch_id);

-- ---------------------------------------------------------------------------
-- 3. Tickets app (ITSM unified)
-- ---------------------------------------------------------------------------
-- Categories
INSERT INTO ticket_categories (id, org_id, bot_id, name, description, color, icon, sort_order, is_active, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Incident', 'Service disruption', '#ef4444', 'alert', 1, true, NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM ticket_categories WHERE org_id = v_org_id AND name = 'Incident');

INSERT INTO ticket_categories (id, org_id, bot_id, name, description, color, icon, sort_order, is_active, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Request', 'Service request', '#22c55e', 'hand', 2, true, NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM ticket_categories WHERE org_id = v_org_id AND name = 'Request');

-- Tickets
INSERT INTO support_tickets (id, org_id, bot_id, ticket_number, subject, description, status, priority, category, source, requester_id, requester_email, requester_name, assignee_id, tags, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'TK-1001', 'Cannot access dashboard', 'Dashboard returns 502 after login', 'open', 'high', 'Incident', 'web', v_user_id, 'alice.sample@example.com', 'Alice Sample', v_user_id, ARRAY['dashboard','urgent'], NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM support_tickets WHERE org_id = v_org_id AND ticket_number = 'TK-1001')
RETURNING id INTO v_ticket_id;

INSERT INTO support_tickets (id, org_id, bot_id, ticket_number, subject, description, status, priority, category, source, requester_id, requester_email, requester_name, assignee_id, tags, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'TK-1002', 'New hire onboarding', 'Please provision access for new hire', 'in_progress', 'medium', 'Request', 'email', NULL, 'bruno.demo@example.com', 'Bruno Demo', v_user_id, ARRAY['onboarding'], NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM support_tickets WHERE org_id = v_org_id AND ticket_number = 'TK-1002');

INSERT INTO support_tickets (id, org_id, bot_id, ticket_number, subject, description, status, priority, category, source, requester_id, requester_email, requester_name, assignee_id, tags, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'TK-1003', 'Export report missing columns', 'Monthly export is missing created_at column', 'resolved', 'low', 'Request', 'web', NULL, 'carla.test@example.com', 'Carla Test', v_user_id, ARRAY['reports'], NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM support_tickets WHERE org_id = v_org_id AND ticket_number = 'TK-1003');

-- Ticket comments (ticket may already exist from prior run)
INSERT INTO ticket_comments (id, ticket_id, author_id, author_name, author_email, content, is_internal, attachments, created_at)
SELECT gen_random_uuid(), st.id, v_user_id, 'Alice Sample', 'alice.sample@example.com', 'Reproduced the 502, checking logs.', false, '[]', NOW()
FROM support_tickets st
WHERE st.org_id = v_org_id AND st.ticket_number = 'TK-1001'
  AND NOT EXISTS (SELECT 1 FROM ticket_comments tc WHERE tc.ticket_id = st.id AND tc.content = 'Reproduced the 502, checking logs.');

INSERT INTO ticket_comments (id, ticket_id, author_id, author_name, author_email, content, is_internal, attachments, created_at)
SELECT gen_random_uuid(), st.id, v_user_id, 'Alice Sample', 'alice.sample@example.com', 'Internal note: suspect upstream cache.', true, '[]', NOW()
FROM support_tickets st
WHERE st.org_id = v_org_id AND st.ticket_number = 'TK-1001'
  AND NOT EXISTS (SELECT 1 FROM ticket_comments tc WHERE tc.ticket_id = st.id AND tc.content = 'Internal note: suspect upstream cache.');

-- ---------------------------------------------------------------------------
-- 4. Billing + ERP app
-- ---------------------------------------------------------------------------
-- Products (with branch scope)
INSERT INTO products (id, org_id, bot_id, sku, name, description, category, product_type, price, cost, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, attributes, created_at, updated_at, branch_id)
SELECT * FROM (VALUES
  (gen_random_uuid(), v_org_id, v_bot_id, 'vps-small', 'VPS Small', '4 vCPU / 8 GB RAM / 100 GB NVMe', 'infrastructure', 'service', 9.99, 4.00, 'USD', 0, 'unit', -1, 0, true, '{"vcpu":4,"ram_gb":8,"storage_gb":100}'::jsonb, NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_org_id, v_bot_id, 'gpu-basic', 'GPU Basic', 'RTX 3060 12 GB VRAM', 'infrastructure', 'service', 39.99, 20.00, 'USD', 0, 'unit', -1, 0, true, '{"gpu_model":"RTX 3060","vram_gb":12}'::jsonb, NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_org_id, v_bot_id, 'domain-com', 'Domain .com', 'Annual .com domain', 'communication', 'service', 21.99, 10.00, 'USD', 0, 'unit', -1, 0, true, '{"tld":".com"}'::jsonb, NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_org_id, v_bot_id, 'printer-bambu-x1', 'Bambu X1 Printer', 'High-speed FDM printer', 'print3d', 'physical', 1499.99, 1100.00, 'USD', 0, 'unit', 25, 5, true, '{"brand":"Bambu","technology":"FDM","max_temp_c":300}'::jsonb, NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_org_id, v_bot_id, 'print-pla-100g', 'PLA Filament 1kg', 'PLA filament, 1.75mm, black', 'print3d', 'physical', 19.99, 8.00, 'USD', 0, 'unit', 500, 20, true, '{"material":"PLA","diameter_mm":1.75,"color":"black"}'::jsonb, NOW(), NOW(), v_branch_id),
  (gen_random_uuid(), v_org_id, v_bot_id, 'print-service-sla', 'SLA Print Service', 'Online resin print, per cm³', 'print3d', 'service', 0.35, 0.15, 'USD', 0, 'cm3', 100000, 1000, true, '{"process":"SLA","resolution_um":50}'::jsonb, NOW(), NOW(), v_branch_id)
) AS v(id, org_id, bot_id, sku, name, description, category, product_type, price, cost, currency, tax_rate, unit, stock_quantity, low_stock_threshold, is_active, attributes, created_at, updated_at, branch_id)
WHERE NOT EXISTS (SELECT 1 FROM products WHERE org_id = v_org_id AND sku = v.sku);

-- Invoices
INSERT INTO billing_invoices (id, org_id, bot_id, invoice_number, customer_id, customer_name, customer_email, status, issue_date, due_date, subtotal, tax_rate, tax_amount, total, amount_paid, amount_due, currency, notes, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'INV-2026-0001', v_account_id, 'Acme Corp', 'billing@acme.example.com', 'paid', '2026-07-01', '2026-07-31', 2500.00, 0, 0, 2500.00, 2500.00, 0, 'USD', 'Sample paid invoice', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM billing_invoices WHERE org_id = v_branch_id AND invoice_number = 'INV-2026-0001')
RETURNING id INTO v_inv_id;

IF v_inv_id IS NULL THEN
    SELECT id INTO v_inv_id FROM billing_invoices WHERE org_id = v_branch_id AND invoice_number = 'INV-2026-0001' LIMIT 1;
END IF;

INSERT INTO billing_invoices (id, org_id, bot_id, invoice_number, customer_id, customer_name, customer_email, status, issue_date, due_date, subtotal, tax_rate, tax_amount, total, amount_paid, amount_due, currency, notes, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'INV-2026-0002', v_account_id, 'Acme Corp', 'billing@acme.example.com', 'sent', '2026-08-01', '2026-08-31', 950.00, 0, 0, 950.00, 0, 950.00, 'USD', 'Sample pending invoice', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM billing_invoices WHERE org_id = v_branch_id AND invoice_number = 'INV-2026-0002');

-- Quotes
INSERT INTO billing_quotes (id, org_id, bot_id, quote_number, customer_id, customer_name, customer_email, status, issue_date, valid_until, subtotal, total, currency, notes, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, 'QUO-2026-0001', v_account_id, 'Acme Corp', 'billing@acme.example.com', 'draft', '2026-08-01', '2026-08-30', 45000.00, 45000.00, 'USD', 'Expansion quote', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM billing_quotes WHERE org_id = v_branch_id AND quote_number = 'QUO-2026-0001')
RETURNING id INTO v_quote_id;

-- Payments
INSERT INTO billing_payments (id, org_id, bot_id, invoice_id, payment_number, amount, currency, payment_method, status, payer_name, payer_email, paid_at, created_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, v_inv_id, 'PAY-2026-0001', 2500.00, 'USD', 'card', 'completed', 'Acme Corp', 'billing@acme.example.com', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM billing_payments WHERE org_id = v_branch_id AND payment_number = 'PAY-2026-0001');

-- Recurring subscription
INSERT INTO billing_recurring (id, org_id, bot_id, customer_id, customer_name, customer_email, status, frequency, interval_count, amount, currency, description, next_invoice_date, start_date, created_at, branch_id)
SELECT gen_random_uuid(), v_branch_id, v_bot_id, v_account_id, 'Acme Corp', 'billing@acme.example.com', 'active', 'monthly', 1, 499.00, 'USD', 'Enterprise monthly plan', '2026-09-01', '2026-08-01', NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM billing_recurring WHERE org_id = v_branch_id AND customer_email = 'billing@acme.example.com' AND description = 'Enterprise monthly plan');

-- ---------------------------------------------------------------------------
-- 5. Tasks + Project apps
-- ---------------------------------------------------------------------------
INSERT INTO tasks (id, title, description, status, priority, assignee_id, reporter_id, due_date, tags, estimated_hours, actual_hours, progress, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), 'Design onboarding flow', 'UX flow for new user onboarding', 'in_progress', 'high', v_user_id, v_user_id, '2026-08-20', ARRAY['ux','onboarding'], 12, 6, 50, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM tasks WHERE branch_id = v_branch_id AND title = 'Design onboarding flow');

INSERT INTO tasks (id, title, description, status, priority, assignee_id, reporter_id, due_date, tags, estimated_hours, actual_hours, progress, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), 'Fix login token refresh', 'Token refresh loop after idle', 'todo', 'urgent', v_user_id, v_user_id, '2026-08-18', ARRAY['auth','bug'], 6, 0, 0, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM tasks WHERE branch_id = v_branch_id AND title = 'Fix login token refresh');

-- Projects (uses `org` varchar + project_type, no branch_id column)
INSERT INTO projects (id, org, name, project_type, deploy_target, repo_url, environment, status, framework, description, created_at, updated_at)
SELECT gen_random_uuid(), 'default', 'Platform Migration', 'app', 'none', 'https://git.example.com/platform-migration.git', 'development', 'active', 'rust', 'Migrate platform to unified billing', NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM projects WHERE name = 'Platform Migration')
RETURNING id INTO v_proj_id;

-- ---------------------------------------------------------------------------
-- 6. Calendar app
-- ---------------------------------------------------------------------------
INSERT INTO calendars (id, org_id, bot_id, owner_id, name, description, color, timezone, is_primary, is_visible, is_shared, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_user_id, 'Work', 'Primary work calendar', '#3b82f6', 'UTC', true, true, false, NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM calendars WHERE org_id = v_org_id AND owner_id = v_user_id)
RETURNING id INTO v_cal_id;

IF v_cal_id IS NULL THEN
    SELECT id INTO v_cal_id FROM calendars WHERE org_id = v_org_id AND owner_id = v_user_id LIMIT 1;
END IF;

INSERT INTO calendar_events (id, org_id, bot_id, calendar_id, owner_id, title, description, location, start_time, end_time, status, visibility, busy_status, reminders, attendees, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_cal_id, v_user_id, 'Sprint Planning', 'Plan the next sprint', 'Zoom', '2026-08-10 09:00:00+00', '2026-08-10 10:00:00+00', 'confirmed', 'default', 'busy', '[]', '[{"email":"alice.sample@example.com"}]', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM calendar_events WHERE org_id = v_org_id AND title = 'Sprint Planning');

INSERT INTO calendar_events (id, org_id, bot_id, calendar_id, owner_id, title, description, location, start_time, end_time, status, visibility, busy_status, reminders, attendees, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_cal_id, v_user_id, 'Product Review', 'Demo latest changes', 'Meeting Room 1', '2026-08-12 15:00:00+00', '2026-08-12 16:00:00+00', 'confirmed', 'default', 'busy', '[]', '[]', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM calendar_events WHERE org_id = v_org_id AND title = 'Product Review');

-- ---------------------------------------------------------------------------
-- 7. Research app
-- ---------------------------------------------------------------------------
INSERT INTO research_projects (id, org_id, bot_id, name, description, status, owner_id, tags, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Market Analysis 2026', 'Competitive market research', 'active', v_user_id, ARRAY['market','2026'], NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM research_projects WHERE org_id = v_org_id AND name = 'Market Analysis 2026')
RETURNING id INTO v_proj_research_id;

IF v_proj_research_id IS NULL THEN
    SELECT id INTO v_proj_research_id FROM research_projects WHERE org_id = v_org_id AND name = 'Market Analysis 2026' LIMIT 1;
END IF;

INSERT INTO research_findings (id, project_id, title, content, finding_type, confidence_level, supporting_sources, status, created_by, created_at, updated_at)
SELECT gen_random_uuid(), v_proj_research_id, 'RAG adoption growing', 'Enterprise RAG adoption grew 40% YoY in 2025.', 'insight', 'high', '[{"url":"https://example.com/rag-report"}]', 'published', v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_findings WHERE project_id = v_proj_research_id AND title = 'RAG adoption growing');

INSERT INTO research_findings (id, project_id, title, content, finding_type, confidence_level, supporting_sources, status, created_by, created_at, updated_at)
SELECT gen_random_uuid(), v_proj_research_id, 'GPU demand shift', 'Demand shifting to smaller on-prem GPU nodes.', 'trend', 'medium', '[]', 'draft', v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_findings WHERE project_id = v_proj_research_id AND title = 'GPU demand shift');

INSERT INTO research_sources (id, project_id, source_type, name, url, content, summary, credibility_score, is_verified, added_by, created_at, updated_at)
SELECT gen_random_uuid(), v_proj_research_id, 'web', 'RAG Industry Report 2025', 'https://example.com/rag-report', 'Full text...', 'Summary of RAG adoption', 85, true, v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_sources WHERE project_id = v_proj_research_id AND name = 'RAG Industry Report 2025');

INSERT INTO research_sources (id, project_id, source_type, name, url, content, summary, credibility_score, is_verified, added_by, created_at, updated_at)
SELECT gen_random_uuid(), v_proj_research_id, 'pdf', 'GPU Market Analysis', 'https://example.com/gpu-market.pdf', 'Full text...', 'GPU pricing trends', 70, false, v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_sources WHERE project_id = v_proj_research_id AND name = 'GPU Market Analysis');

INSERT INTO research_searches (id, user_id, query, created_at)
SELECT gen_random_uuid(), v_user_id, 'enterprise rag adoption 2025', NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_searches WHERE user_id = v_user_id AND query = 'enterprise rag adoption 2025');

INSERT INTO research_searches (id, user_id, query, created_at)
SELECT gen_random_uuid(), v_user_id, 'on-prem gpu inference', NOW()
WHERE NOT EXISTS (SELECT 1 FROM research_searches WHERE user_id = v_user_id AND query = 'on-prem gpu inference');

-- ---------------------------------------------------------------------------
-- 8. Compliance app
-- ---------------------------------------------------------------------------
INSERT INTO compliance_checks (id, org_id, bot_id, framework, control_id, control_name, status, score, checked_at, evidence, notes, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'SOC2', 'CC6.1', 'Access control policy', 'compliant', 1.00, NOW(), '[]', 'Review passed', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM compliance_checks WHERE org_id = v_org_id AND control_id = 'CC6.1')
RETURNING id INTO v_check_id;

IF v_check_id IS NULL THEN
    SELECT id INTO v_check_id FROM compliance_checks WHERE org_id = v_org_id AND control_id = 'CC6.1' LIMIT 1;
END IF;

INSERT INTO compliance_checks (id, org_id, bot_id, framework, control_id, control_name, status, score, checked_at, evidence, notes, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'ISO27001', 'A.9.4.2', 'Secure log-on procedures', 'non_compliant', 0.40, NOW(), '[]', 'MFA gaps found', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM compliance_checks WHERE org_id = v_org_id AND control_id = 'A.9.4.2');

INSERT INTO compliance_issues (id, org_id, bot_id, check_id, severity, title, description, remediation, status, assigned_to, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_check_id, 'high', 'MFA not enforced', 'MFA is optional for admin accounts', 'Enforce MFA for all admins', 'open', v_user_id, NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM compliance_issues WHERE org_id = v_org_id AND title = 'MFA not enforced');

-- ---------------------------------------------------------------------------
-- 9. OKR / Goals app
-- ---------------------------------------------------------------------------
INSERT INTO okr_objectives (id, org_id, bot_id, owner_id, title, description, period, period_start, period_end, status, progress, visibility, tags, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_user_id, 'Ship v1.0', 'Deliver the v1 platform', 'Q3-2026', '2026-07-01', '2026-09-30', 'active', 45, 'team', ARRAY['v1','launch'], NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM okr_objectives WHERE org_id = v_org_id AND title = 'Ship v1.0')
RETURNING id INTO v_obj_id;

IF v_obj_id IS NULL THEN
    SELECT id INTO v_obj_id FROM okr_objectives WHERE org_id = v_org_id AND title = 'Ship v1.0' LIMIT 1;
END IF;

INSERT INTO okr_key_results (id, org_id, bot_id, objective_id, owner_id, title, metric_type, start_value, target_value, current_value, unit, status, due_date, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_obj_id, v_user_id, 'Complete all 12 milestones', 'count', 0, 12, 5, 'milestones', 'in_progress', '2026-09-30', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM okr_key_results WHERE org_id = v_org_id AND objective_id = v_obj_id AND title = 'Complete all 12 milestones');

INSERT INTO okr_key_results (id, org_id, bot_id, objective_id, owner_id, title, metric_type, start_value, target_value, current_value, unit, status, due_date, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_obj_id, v_user_id, 'Reach 95% test coverage', 'percentage', 60, 95, 72, '%', 'in_progress', '2026-09-30', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM okr_key_results WHERE org_id = v_org_id AND objective_id = v_obj_id AND title = 'Reach 95% test coverage');

-- ---------------------------------------------------------------------------
-- 10. Workspace app
-- ---------------------------------------------------------------------------
INSERT INTO workspaces (id, org_id, branch_id, name, description, settings, created_by, created_at, updated_at)
SELECT gen_random_uuid(), v_org_id, v_branch_id, 'Product Workspace', 'Shared product documentation', '{}', v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM workspaces WHERE org_id = v_org_id AND name = 'Product Workspace')
RETURNING id INTO v_ws_id;

IF v_ws_id IS NULL THEN
    SELECT id INTO v_ws_id FROM workspaces WHERE org_id = v_org_id AND name = 'Product Workspace' LIMIT 1;
END IF;

INSERT INTO workspace_pages (id, workspace_id, title, content, properties, is_template, public_edit, position, created_by, created_at, updated_at)
SELECT gen_random_uuid(), v_ws_id, 'Home', '{"type":"doc","content":[]}', '{}', false, false, 0, v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM workspace_pages WHERE workspace_id = v_ws_id AND title = 'Home');

INSERT INTO workspace_pages (id, workspace_id, title, content, properties, is_template, public_edit, position, created_by, created_at, updated_at)
SELECT gen_random_uuid(), v_ws_id, 'Roadmap', '{"type":"doc","content":[]}', '{}', false, false, 1, v_user_id, NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM workspace_pages WHERE workspace_id = v_ws_id AND title = 'Roadmap');

-- ---------------------------------------------------------------------------
-- 11. Social app
-- ---------------------------------------------------------------------------
INSERT INTO social_communities (id, org_id, bot_id, name, slug, description, visibility, join_policy, owner_id, member_count, post_count, is_official, is_featured, settings, created_at, updated_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, 'Company Announcements', 'company-announcements', 'Internal announcements', 'public', 'open', v_user_id, 3, 2, true, false, '{}', NOW(), NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM social_communities WHERE org_id = v_org_id AND slug = 'company-announcements');

INSERT INTO social_posts (id, org_id, bot_id, author_id, content, content_type, hashtags, visibility, comment_count, reaction_counts, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_user_id, 'Welcome to the new platform! 🎉', 'text', ARRAY['launch'], 'public', 2, '{"like":3}', NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM social_posts WHERE org_id = v_org_id AND content = 'Welcome to the new platform! 🎉');

INSERT INTO social_posts (id, org_id, bot_id, author_id, content, content_type, hashtags, visibility, comment_count, reaction_counts, created_at, branch_id)
SELECT gen_random_uuid(), v_org_id, v_bot_id, v_user_id, 'v1.0 is shipping this quarter', 'text', ARRAY['v1'], 'public', 0, '{}', NOW(), v_branch_id
WHERE NOT EXISTS (SELECT 1 FROM social_posts WHERE org_id = v_org_id AND content = 'v1.0 is shipping this quarter');

-- ---------------------------------------------------------------------------
-- 13. o365 app (m365 tables)
-- ---------------------------------------------------------------------------
INSERT INTO m365_sharepoint_items (id, bot_id, organization_id, site_id, list_id, item_id, title, fields, author, modified_at, synced_at)
SELECT gen_random_uuid(), v_bot_id, v_org_id, 'site-engineering', 'list-docs', 'item-1', 'Engineering Docs', '{"doc_count": 42}', 'Alice Sample', NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM m365_sharepoint_items WHERE site_id = 'site-engineering' AND list_id = 'list-docs');

INSERT INTO m365_sharepoint_items (id, bot_id, organization_id, site_id, list_id, item_id, title, fields, author, modified_at, synced_at)
SELECT gen_random_uuid(), v_bot_id, v_org_id, 'site-marketing', 'list-assets', 'item-2', 'Marketing Assets', '{"doc_count": 18}', 'Bruno Demo', NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM m365_sharepoint_items WHERE site_id = 'site-marketing' AND list_id = 'list-assets');

INSERT INTO m365_calendar_events (id, bot_id, subject, start_time, end_time, location, attendees, status)
SELECT gen_random_uuid(), v_bot_id, 'o365 Sprint Review', NOW() + INTERVAL '2 days', NOW() + INTERVAL '2 days' + INTERVAL '1 hour', 'Virtual', '["alice.sample@example.com"]', 'confirmed'
WHERE NOT EXISTS (SELECT 1 FROM m365_calendar_events WHERE subject = 'o365 Sprint Review');

INSERT INTO m365_onedrive_files (id, bot_id, name, path, size_bytes, last_modified, author)
SELECT gen_random_uuid(), v_bot_id, 'quarterly-report.pdf', '/shared/quarterly-report.pdf', 524288, NOW(), 'Alice Sample'
WHERE NOT EXISTS (SELECT 1 FROM m365_onedrive_files WHERE path = '/shared/quarterly-report.pdf');

-- ---------------------------------------------------------------------------
-- 12. Drive app
-- ---------------------------------------------------------------------------
INSERT INTO drive_files (id, file_path, file_type, last_modified, file_size, indexed, user_id, scope, branch_id, path, name, mime_type, created_at, updated_at)
SELECT gen_random_uuid(), '/reports/q3-summary.pdf', 'pdf', NOW(), 245760, true, v_user_id, 'user', v_branch_id, '/reports', 'q3-summary.pdf', 'application/pdf', NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM drive_files WHERE branch_id = v_branch_id AND user_id = v_user_id AND file_path = '/reports/q3-summary.pdf');

INSERT INTO drive_files (id, file_path, file_type, last_modified, file_size, indexed, user_id, scope, branch_id, path, name, mime_type, created_at, updated_at)
SELECT gen_random_uuid(), '/exports/customers.csv', 'csv', NOW(), 10240, true, v_user_id, 'user', v_branch_id, '/exports', 'customers.csv', 'text/csv', NOW(), NOW()
WHERE NOT EXISTS (SELECT 1 FROM drive_files WHERE branch_id = v_branch_id AND user_id = v_user_id AND file_path = '/exports/customers.csv');

RAISE NOTICE 'Sample data populated for org %, branch %, user %', v_org_id, v_branch_id, v_user_id;

END $$;

COMMIT;
