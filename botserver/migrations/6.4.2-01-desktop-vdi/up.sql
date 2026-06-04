CREATE TABLE IF NOT EXISTS desktop_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL,
    organization_id UUID,
    name VARCHAR(255) NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL DEFAULT 5900,
    protocol VARCHAR(10) NOT NULL DEFAULT 'vnc',
    auth_type VARCHAR(20),
    auto_connect BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS desktop_connection_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    connection_id UUID REFERENCES desktop_connections(id) ON DELETE SET NULL,
    user_id UUID NOT NULL,
    session_id UUID NOT NULL,
    host VARCHAR(255) NOT NULL,
    port INTEGER NOT NULL,
    protocol VARCHAR(10) NOT NULL,
    connected_at TIMESTAMPTZ NOT NULL,
    disconnected_at TIMESTAMPTZ,
    bytes_transferred BIGINT DEFAULT 0,
    disconnect_reason VARCHAR(50)
);

CREATE INDEX idx_desktop_connections_user_id ON desktop_connections(user_id);
CREATE INDEX idx_desktop_connection_log_user_id ON desktop_connection_log(user_id);
CREATE INDEX idx_desktop_connection_log_session_id ON desktop_connection_log(session_id);
