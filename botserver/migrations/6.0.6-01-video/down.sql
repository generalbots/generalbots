DROP INDEX IF EXISTS idx_video_command_history_executed;
DROP INDEX IF EXISTS idx_video_command_history_project;
DROP TABLE IF EXISTS video_command_history;

DROP INDEX IF EXISTS idx_video_exports_status;
DROP INDEX IF EXISTS idx_video_exports_project;
DROP TABLE IF EXISTS video_exports;

DROP INDEX IF EXISTS idx_video_audio_tracks_project;
DROP TABLE IF EXISTS video_audio_tracks;

DROP INDEX IF EXISTS idx_video_layers_track;
DROP INDEX IF EXISTS idx_video_layers_project;
DROP TABLE IF EXISTS video_layers;

DROP INDEX IF EXISTS idx_video_clips_order;
DROP INDEX IF EXISTS idx_video_clips_project;
DROP TABLE IF EXISTS video_clips;

DROP INDEX IF EXISTS idx_video_projects_created_by;
DROP INDEX IF EXISTS idx_video_projects_status;
DROP INDEX IF EXISTS idx_video_projects_organization;
DROP TABLE IF EXISTS video_projects;
