DROP INDEX IF EXISTS idx_people_time_off_status;
DROP INDEX IF EXISTS idx_people_time_off_dates;
DROP INDEX IF EXISTS idx_people_time_off_person;
DROP INDEX IF EXISTS idx_people_time_off_org_bot;

DROP INDEX IF EXISTS idx_people_person_skills_skill;
DROP INDEX IF EXISTS idx_people_person_skills_person;

DROP INDEX IF EXISTS idx_people_skills_category;
DROP INDEX IF EXISTS idx_people_skills_org_bot;

DROP INDEX IF EXISTS idx_people_departments_org_code;
DROP INDEX IF EXISTS idx_people_departments_head;
DROP INDEX IF EXISTS idx_people_departments_parent;
DROP INDEX IF EXISTS idx_people_departments_org_bot;

DROP INDEX IF EXISTS idx_people_org_chart_reports_to;
DROP INDEX IF EXISTS idx_people_org_chart_person;
DROP INDEX IF EXISTS idx_people_org_chart_org;

DROP INDEX IF EXISTS idx_people_team_members_person;
DROP INDEX IF EXISTS idx_people_team_members_team;

DROP INDEX IF EXISTS idx_people_teams_leader;
DROP INDEX IF EXISTS idx_people_teams_parent;
DROP INDEX IF EXISTS idx_people_teams_org_bot;

DROP INDEX IF EXISTS idx_people_org_email;
DROP INDEX IF EXISTS idx_people_user;
DROP INDEX IF EXISTS idx_people_active;
DROP INDEX IF EXISTS idx_people_manager;
DROP INDEX IF EXISTS idx_people_department;
DROP INDEX IF EXISTS idx_people_email;
DROP INDEX IF EXISTS idx_people_org_bot;

DROP TABLE IF EXISTS people_time_off;
DROP TABLE IF EXISTS people_person_skills;
DROP TABLE IF EXISTS people_skills;
DROP TABLE IF EXISTS people_departments;
DROP TABLE IF EXISTS people_org_chart;
DROP TABLE IF EXISTS people_team_members;
DROP TABLE IF EXISTS people_teams;
DROP TABLE IF EXISTS people;
