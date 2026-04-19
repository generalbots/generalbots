DROP INDEX IF EXISTS idx_attendant_session_wrap_up_session;

DROP INDEX IF EXISTS idx_attendant_wrap_up_codes_org_code;
DROP INDEX IF EXISTS idx_attendant_wrap_up_codes_org_bot;

DROP INDEX IF EXISTS idx_attendant_tags_org_name;
DROP INDEX IF EXISTS idx_attendant_tags_org_bot;

DROP INDEX IF EXISTS idx_attendant_canned_shortcut;
DROP INDEX IF EXISTS idx_attendant_canned_org_bot;

DROP INDEX IF EXISTS idx_attendant_transfers_session;

DROP INDEX IF EXISTS idx_attendant_agent_status_status;
DROP INDEX IF EXISTS idx_attendant_agent_status_org;

DROP INDEX IF EXISTS idx_attendant_queue_agents_agent;
DROP INDEX IF EXISTS idx_attendant_queue_agents_queue;

DROP INDEX IF EXISTS idx_attendant_session_messages_created;
DROP INDEX IF EXISTS idx_attendant_session_messages_session;

DROP INDEX IF EXISTS idx_attendant_sessions_number;
DROP INDEX IF EXISTS idx_attendant_sessions_created;
DROP INDEX IF EXISTS idx_attendant_sessions_customer;
DROP INDEX IF EXISTS idx_attendant_sessions_queue;
DROP INDEX IF EXISTS idx_attendant_sessions_agent;
DROP INDEX IF EXISTS idx_attendant_sessions_status;
DROP INDEX IF EXISTS idx_attendant_sessions_org_bot;

DROP INDEX IF EXISTS idx_attendant_queues_active;
DROP INDEX IF EXISTS idx_attendant_queues_org_bot;

DROP TABLE IF EXISTS attendant_session_wrap_up;
DROP TABLE IF EXISTS attendant_wrap_up_codes;
DROP TABLE IF EXISTS attendant_tags;
DROP TABLE IF EXISTS attendant_canned_responses;
DROP TABLE IF EXISTS attendant_transfers;
DROP TABLE IF EXISTS attendant_agent_status;
DROP TABLE IF EXISTS attendant_queue_agents;
DROP TABLE IF EXISTS attendant_session_messages;
DROP TABLE IF EXISTS attendant_sessions;
DROP TABLE IF EXISTS attendant_queues;
