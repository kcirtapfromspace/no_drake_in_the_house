-- Apple Music Enforcement Tables
-- Tracks enforcement runs and actions for rollback support

-- Enforcement runs table
CREATE TABLE IF NOT EXISTS apple_music_enforcement_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    connection_id UUID NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'running',
    options JSONB NOT NULL DEFAULT '{}',
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    songs_scanned INT NOT NULL DEFAULT 0,
    albums_scanned INT NOT NULL DEFAULT 0,
    songs_disliked INT NOT NULL DEFAULT 0,
    albums_disliked INT NOT NULL DEFAULT 0,
    errors INT NOT NULL DEFAULT 0,
    error_details JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for user enforcement history queries
CREATE INDEX IF NOT EXISTS idx_apple_music_enforcement_runs_user
    ON apple_music_enforcement_runs(user_id, started_at DESC);

-- Index for status filtering
CREATE INDEX IF NOT EXISTS idx_apple_music_enforcement_runs_status
    ON apple_music_enforcement_runs(status);

-- Enforcement actions table (for rollback support)
CREATE TABLE IF NOT EXISTS apple_music_enforcement_actions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    run_id UUID NOT NULL REFERENCES apple_music_enforcement_runs(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    resource_type VARCHAR(20) NOT NULL,
    resource_id VARCHAR(255) NOT NULL,
    resource_name TEXT,
    artist_name TEXT,
    action VARCHAR(20) NOT NULL,
    previous_rating SMALLINT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Index for run actions lookup
CREATE INDEX IF NOT EXISTS idx_apple_music_enforcement_actions_run
    ON apple_music_enforcement_actions(run_id);

-- Index for user actions lookup
CREATE INDEX IF NOT EXISTS idx_apple_music_enforcement_actions_user
    ON apple_music_enforcement_actions(user_id);

-- Index for resource lookup (to check if already rated)
CREATE INDEX IF NOT EXISTS idx_apple_music_enforcement_actions_resource
    ON apple_music_enforcement_actions(user_id, resource_type, resource_id);

-- Comments for documentation
COMMENT ON TABLE apple_music_enforcement_runs IS 'Tracks Apple Music enforcement runs for each user';
COMMENT ON TABLE apple_music_enforcement_actions IS 'Individual rating actions taken during enforcement, enables rollback';
COMMENT ON COLUMN apple_music_enforcement_runs.status IS 'Status: pending, running, completed, failed, rolled_back';
COMMENT ON COLUMN apple_music_enforcement_actions.resource_type IS 'Type: song, library_song, album, library_album, playlist';
COMMENT ON COLUMN apple_music_enforcement_actions.action IS 'Action taken: dislike, remove_rating';
COMMENT ON COLUMN apple_music_enforcement_actions.previous_rating IS 'Previous rating value (if any) for rollback: 1=like, -1=dislike';
