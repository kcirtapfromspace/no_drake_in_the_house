-- User Track Blocks: Allow users to block individual tracks
-- This enables fine-grained control over what content is blocked

CREATE TABLE IF NOT EXISTS user_track_blocks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    track_id VARCHAR(255) NOT NULL,
    track_title TEXT NOT NULL DEFAULT '',
    track_role VARCHAR(50) NOT NULL DEFAULT 'main',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Each user can only block a track once
    CONSTRAINT unique_user_track_block UNIQUE (user_id, track_id)
);

-- Indexes for efficient lookups
CREATE INDEX IF NOT EXISTS idx_user_track_blocks_user_id ON user_track_blocks(user_id);
CREATE INDEX IF NOT EXISTS idx_user_track_blocks_artist_id ON user_track_blocks(artist_id);
CREATE INDEX IF NOT EXISTS idx_user_track_blocks_track_id ON user_track_blocks(track_id);
CREATE INDEX IF NOT EXISTS idx_user_track_blocks_user_artist ON user_track_blocks(user_id, artist_id);
