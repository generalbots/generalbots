-- Rollback Migration: 6.1.0 Enterprise Features
-- WARNING: This will delete all enterprise feature data!
-- NOTE: TABLES AND INDEXES ONLY - No views, triggers, or functions per project standards

-- Drop test support tables
DROP TABLE IF EXISTS test_execution_logs;
DROP TABLE IF EXISTS test_accounts;

-- Drop calendar tables
DROP TABLE IF EXISTS calendar_shares;
DROP TABLE IF EXISTS calendar_resource_bookings;
DROP TABLE IF EXISTS calendar_resources;

-- Drop task tables
DROP TABLE IF EXISTS task_recurrence;
DROP TABLE IF EXISTS task_time_entries;
DROP TABLE IF EXISTS task_dependencies;

-- Drop collaboration tables
DROP TABLE IF EXISTS document_presence;

-- Drop drive tables
DROP TABLE IF EXISTS storage_quotas;
DROP TABLE IF EXISTS file_sync_status;
DROP TABLE IF EXISTS file_trash;
DROP TABLE IF EXISTS file_activities;
DROP TABLE IF EXISTS file_shares;
DROP TABLE IF EXISTS file_comments;
DROP TABLE IF EXISTS file_versions;

-- Drop meet tables
DROP TABLE IF EXISTS user_virtual_backgrounds;
DROP TABLE IF EXISTS meeting_captions;
DROP TABLE IF EXISTS meeting_waiting_room;
DROP TABLE IF EXISTS meeting_questions;
DROP TABLE IF EXISTS meeting_polls;
DROP TABLE IF EXISTS meeting_breakout_rooms;
DROP TABLE IF EXISTS meeting_recordings;

-- Drop email tables (order matters due to foreign keys)
DROP TABLE IF EXISTS shared_mailbox_members;
DROP TABLE IF EXISTS shared_mailboxes;
DROP TABLE IF EXISTS distribution_lists;
DROP TABLE IF EXISTS email_label_assignments;
DROP TABLE IF EXISTS email_labels;
DROP TABLE IF EXISTS email_rules;
DROP TABLE IF EXISTS email_auto_responders;
DROP TABLE IF EXISTS email_templates;
DROP TABLE IF EXISTS scheduled_emails;
DROP TABLE IF EXISTS email_signatures;
DROP TABLE IF EXISTS global_email_signatures;
