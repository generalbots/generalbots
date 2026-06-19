DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS monitoring_cameras (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        name VARCHAR(200) NOT NULL,
        rtsp_url TEXT NOT NULL,
        username VARCHAR(100),
        password_encrypted TEXT,
        location VARCHAR(200),
        zone VARCHAR(100),
        enabled BOOLEAN NOT NULL DEFAULT true,
        detection_zones JSONB NOT NULL DEFAULT '[]',
        fps_target INTEGER NOT NULL DEFAULT 5,
        resolution VARCHAR(20) NOT NULL DEFAULT '1280x720',
        status VARCHAR(30) NOT NULL DEFAULT 'offline',
        last_frame_at TIMESTAMPTZ,
        last_seen_at TIMESTAMPTZ,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_cameras_bot ON monitoring_cameras(bot_id);
    CREATE INDEX IF NOT EXISTS idx_cameras_status ON monitoring_cameras(status);
    CREATE INDEX IF NOT EXISTS idx_cameras_zone ON monitoring_cameras(zone);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating monitoring_cameras table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS monitoring_events (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        camera_id UUID NOT NULL,
        organization_id UUID NOT NULL,
        event_type VARCHAR(50) NOT NULL,
        severity VARCHAR(20) NOT NULL DEFAULT 'info',
        confidence NUMERIC(5,4) NOT NULL DEFAULT 0,
        description TEXT,
        snapshot_url TEXT,
        metadata JSONB NOT NULL DEFAULT '{}',
        detected_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        acknowledged_at TIMESTAMPTZ,
        acknowledged_by UUID,
        resolved_at TIMESTAMPTZ
    );

    CREATE INDEX IF NOT EXISTS idx_events_camera ON monitoring_events(camera_id);
    CREATE INDEX IF NOT EXISTS idx_events_detected ON monitoring_events(detected_at);
    CREATE INDEX IF NOT EXISTS idx_events_severity ON monitoring_events(severity);
    CREATE INDEX IF NOT EXISTS idx_events_type ON monitoring_events(event_type);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating monitoring_events table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS monitoring_snapshots (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        camera_id UUID NOT NULL,
        event_id UUID,
        storage_path TEXT NOT NULL,
        thumbnail_path TEXT,
        width INTEGER NOT NULL,
        height INTEGER NOT NULL,
        file_size BIGINT,
        captured_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_snapshots_camera ON monitoring_snapshots(camera_id);
    CREATE INDEX IF NOT EXISTS idx_snapshots_captured ON monitoring_snapshots(captured_at);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating monitoring_snapshots table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS monitoring_alert_rules (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        bot_id UUID NOT NULL,
        name VARCHAR(200) NOT NULL,
        event_type VARCHAR(50) NOT NULL,
        severity_threshold VARCHAR(20) NOT NULL DEFAULT 'medium',
        camera_ids JSONB NOT NULL DEFAULT '[]',
        zones JSONB NOT NULL DEFAULT '[]',
        schedule JSONB NOT NULL DEFAULT '{}',
        actions JSONB NOT NULL DEFAULT '[]',
        enabled BOOLEAN NOT NULL DEFAULT true,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_alert_rules_bot ON monitoring_alert_rules(bot_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating monitoring_alert_rules table: %', SQLERRM;
END $$;
