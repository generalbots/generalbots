DROP INDEX IF EXISTS idx_research_exports_status;
DROP INDEX IF EXISTS idx_research_exports_project;

DROP INDEX IF EXISTS idx_research_collaborators_user;
DROP INDEX IF EXISTS idx_research_collaborators_project;

DROP INDEX IF EXISTS idx_research_citations_style;
DROP INDEX IF EXISTS idx_research_citations_source;

DROP INDEX IF EXISTS idx_research_findings_status;
DROP INDEX IF EXISTS idx_research_findings_type;
DROP INDEX IF EXISTS idx_research_findings_project;

DROP INDEX IF EXISTS idx_research_notes_tags;
DROP INDEX IF EXISTS idx_research_notes_type;
DROP INDEX IF EXISTS idx_research_notes_source;
DROP INDEX IF EXISTS idx_research_notes_project;

DROP INDEX IF EXISTS idx_research_sources_verified;
DROP INDEX IF EXISTS idx_research_sources_type;
DROP INDEX IF EXISTS idx_research_sources_project;

DROP INDEX IF EXISTS idx_research_projects_tags;
DROP INDEX IF EXISTS idx_research_projects_status;
DROP INDEX IF EXISTS idx_research_projects_owner;
DROP INDEX IF EXISTS idx_research_projects_org_bot;

DROP TABLE IF EXISTS research_exports;
DROP TABLE IF EXISTS research_collaborators;
DROP TABLE IF EXISTS research_citations;
DROP TABLE IF EXISTS research_findings;
DROP TABLE IF EXISTS research_notes;
DROP TABLE IF EXISTS research_sources;
DROP TABLE IF EXISTS research_projects;
