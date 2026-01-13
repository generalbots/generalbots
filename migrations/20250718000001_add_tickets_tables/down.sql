DROP INDEX IF EXISTS idx_ticket_tags_org_name;
DROP INDEX IF EXISTS idx_ticket_tags_org_bot;

DROP INDEX IF EXISTS idx_ticket_categories_parent;
DROP INDEX IF EXISTS idx_ticket_categories_org_bot;

DROP INDEX IF EXISTS idx_ticket_canned_shortcut;
DROP INDEX IF EXISTS idx_ticket_canned_org_bot;

DROP INDEX IF EXISTS idx_ticket_sla_priority;
DROP INDEX IF EXISTS idx_ticket_sla_org_bot;

DROP INDEX IF EXISTS idx_ticket_comments_created;
DROP INDEX IF EXISTS idx_ticket_comments_ticket;

DROP INDEX IF EXISTS idx_support_tickets_org_number;
DROP INDEX IF EXISTS idx_support_tickets_number;
DROP INDEX IF EXISTS idx_support_tickets_created;
DROP INDEX IF EXISTS idx_support_tickets_requester;
DROP INDEX IF EXISTS idx_support_tickets_assignee;
DROP INDEX IF EXISTS idx_support_tickets_priority;
DROP INDEX IF EXISTS idx_support_tickets_status;
DROP INDEX IF EXISTS idx_support_tickets_org_bot;

DROP TABLE IF EXISTS ticket_tags;
DROP TABLE IF EXISTS ticket_categories;
DROP TABLE IF EXISTS ticket_canned_responses;
DROP TABLE IF EXISTS ticket_sla_policies;
DROP TABLE IF EXISTS ticket_comments;
DROP TABLE IF EXISTS support_tickets;
