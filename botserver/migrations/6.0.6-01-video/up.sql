-- Video Projects
CREATE TABLE IF NOT EXISTS video_projects (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    organization_id UUID,
    created_by UUID,
    name TEXT NOT NULL,
    description TEXT,
    resolution_width INT NOT NULL DEFAULT 1920,
    resolution_height INT NOT NULL DEFAULT 1080,
    fps INT NOT NULL DEFAULT 30,
    total_duration_ms BIGINT NOT NULL DEFAULT 0,
    timeline_json JSONB NOT NULL DEFAULT '{"clips": []}',
    layers_json JSONB NOT NULL DEFAULT '[]',
    audio_tracks_json JSONB NOT NULL DEFAULT '[]',
    playhead_ms BIGINT NOT NULL DEFAULT 0,
    selection_json JSONB NOT NULL DEFAULT '{"type": "None"}',
    zoom_level REAL NOT NULL DEFAULT 1.0,
    thumbnail_url TEXT,
    status TEXT NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_projects_organization ON video_projects(organization_id);
CREATE INDEX IF NOT EXISTS idx_video_projects_status ON video_projects(status);
CREATE INDEX IF NOT EXISTS idx_video_projects_created_by ON video_projects(created_by);

-- Video Clips
CREATE TABLE IF NOT EXISTS video_clips (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES video_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    source_url TEXT NOT NULL,
    start_ms BIGINT NOT NULL DEFAULT 0,
    duration_ms BIGINT NOT NULL DEFAULT 0,
    trim_in_ms BIGINT NOT NULL DEFAULT 0,
    trim_out_ms BIGINT NOT NULL DEFAULT 0,
    volume REAL NOT NULL DEFAULT 1.0,
    clip_order INT NOT NULL DEFAULT 0,
    transition_in TEXT,
    transition_out TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_clips_project ON video_clips(project_id);
CREATE INDEX IF NOT EXISTS idx_video_clips_order ON video_clips(project_id, clip_order);

-- Video Layers (Text, Shapes, Images)
CREATE TABLE IF NOT EXISTS video_layers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES video_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    layer_type TEXT NOT NULL,
    track_index INT NOT NULL DEFAULT 0,
    start_ms BIGINT NOT NULL DEFAULT 0,
    end_ms BIGINT NOT NULL DEFAULT 5000,
    x REAL NOT NULL DEFAULT 0.5,
    y REAL NOT NULL DEFAULT 0.5,
    width REAL NOT NULL DEFAULT 0.5,
    height REAL NOT NULL DEFAULT 0.2,
    rotation REAL NOT NULL DEFAULT 0.0,
    opacity REAL NOT NULL DEFAULT 1.0,
    properties_json JSONB NOT NULL DEFAULT '{}',
    animation_in TEXT,
    animation_out TEXT,
    locked BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_layers_project ON video_layers(project_id);
CREATE INDEX IF NOT EXISTS idx_video_layers_track ON video_layers(project_id, track_index);

-- Video Audio Tracks
CREATE TABLE IF NOT EXISTS video_audio_tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES video_projects(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    source_url TEXT NOT NULL,
    track_type TEXT NOT NULL,
    start_ms BIGINT NOT NULL DEFAULT 0,
    duration_ms BIGINT NOT NULL DEFAULT 0,
    volume REAL NOT NULL DEFAULT 1.0,
    fade_in_ms BIGINT NOT NULL DEFAULT 0,
    fade_out_ms BIGINT NOT NULL DEFAULT 0,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_audio_tracks_project ON video_audio_tracks(project_id);

-- Video Exports
CREATE TABLE IF NOT EXISTS video_exports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES video_projects(id) ON DELETE CASCADE,
    format TEXT NOT NULL DEFAULT 'mp4',
    quality TEXT NOT NULL DEFAULT 'high',
    status TEXT NOT NULL DEFAULT 'pending',
    progress INT NOT NULL DEFAULT 0,
    output_url TEXT,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_video_exports_project ON video_exports(project_id);
CREATE INDEX IF NOT EXISTS idx_video_exports_status ON video_exports(status);

-- Video Command History (for undo/redo)
CREATE TABLE IF NOT EXISTS video_command_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    project_id UUID NOT NULL REFERENCES video_projects(id) ON DELETE CASCADE,
    user_id UUID,
    command_type TEXT NOT NULL,
    command_json JSONB NOT NULL,
    executed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_video_command_history_project ON video_command_history(project_id);
CREATE INDEX IF NOT EXISTS idx_video_command_history_executed ON video_command_history(project_id, executed_at DESC);
