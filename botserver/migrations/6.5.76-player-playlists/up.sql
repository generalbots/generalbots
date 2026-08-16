DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS player_playlists (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        branch_id UUID NOT NULL,
        user_id UUID,
        name VARCHAR(255) NOT NULL,
        visibility VARCHAR(16) NOT NULL DEFAULT 'private',
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
        updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_player_playlists_branch ON player_playlists(branch_id);
    CREATE INDEX IF NOT EXISTS idx_player_playlists_user ON player_playlists(user_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating player_playlists table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS player_playlist_items (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        playlist_id UUID NOT NULL REFERENCES player_playlists(id) ON DELETE CASCADE,
        media_path TEXT NOT NULL,
        title VARCHAR(500) NOT NULL,
        position INTEGER NOT NULL DEFAULT 0,
        added_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_player_items_playlist ON player_playlist_items(playlist_id);
    CREATE INDEX IF NOT EXISTS idx_player_items_position ON player_playlist_items(playlist_id, position);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating player_playlist_items table: %', SQLERRM;
END $$;

DO $$
BEGIN
    CREATE TABLE IF NOT EXISTS player_playback_events (
        id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
        playlist_id UUID,
        item_id UUID,
        user_id UUID,
        media_path TEXT NOT NULL,
        event_type VARCHAR(16) NOT NULL DEFAULT 'play',
        position_seconds INTEGER NOT NULL DEFAULT 0,
        created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
    );

    CREATE INDEX IF NOT EXISTS idx_player_events_item ON player_playback_events(item_id);
    CREATE INDEX IF NOT EXISTS idx_player_events_playlist ON player_playback_events(playlist_id);
EXCEPTION
    WHEN OTHERS THEN
        RAISE WARNING 'Error creating player_playback_events table: %', SQLERRM;
END $$;
