-- Platform catalog sync tracking
-- Supports multi-platform artist synchronization (Spotify, Apple Music, Tidal, YouTube, Deezer)

-- Track sync operations across all platforms
CREATE TABLE platform_sync_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL, -- 'spotify', 'apple', 'tidal', 'youtube', 'deezer', 'musicbrainz'
    sync_type VARCHAR(50) NOT NULL, -- 'full', 'incremental', 'delta'
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed', 'cancelled'
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    artists_processed INTEGER DEFAULT 0,
    artists_created INTEGER DEFAULT 0,
    artists_updated INTEGER DEFAULT 0,
    artists_merged INTEGER DEFAULT 0,
    errors_count INTEGER DEFAULT 0,
    error_log JSONB DEFAULT '[]',
    checkpoint_data JSONB DEFAULT '{}', -- Resume point for failed syncs: {last_id, page_token, offset}
    rate_limit_delays_ms BIGINT DEFAULT 0,
    api_calls_made INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Multi-platform artist ID mappings (normalized from external_ids JSONB)
-- This provides faster lookups and proper foreign key constraints
CREATE TABLE artist_platform_ids (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL, -- 'spotify', 'apple', 'tidal', 'youtube', 'deezer', 'musicbrainz', 'isni'
    platform_id VARCHAR(255) NOT NULL,
    platform_url TEXT,
    last_verified_at TIMESTAMP WITH TIME ZONE,
    verification_status VARCHAR(20) DEFAULT 'unverified', -- 'verified', 'unverified', 'disputed', 'invalid'
    confidence_score FLOAT DEFAULT 0.5 CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    metadata JSONB DEFAULT '{}', -- Platform-specific data: {followers, monthly_listeners, popularity}
    sync_run_id UUID REFERENCES platform_sync_runs(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_artist_platform UNIQUE (artist_id, platform),
    CONSTRAINT unique_platform_id UNIQUE (platform, platform_id)
);

-- Platform API credentials (encrypted storage)
CREATE TABLE platform_credentials (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL UNIQUE,
    credential_type VARCHAR(50) NOT NULL, -- 'oauth_client', 'api_key', 'jwt_key'
    client_id_encrypted BYTEA,
    client_secret_encrypted BYTEA,
    api_key_encrypted BYTEA,
    scopes TEXT[],
    rate_limit_config JSONB NOT NULL DEFAULT '{}', -- {requests_per_window, window_seconds, burst_allowance}
    base_url TEXT,
    auth_url TEXT,
    token_url TEXT,
    last_refreshed_at TIMESTAMP WITH TIME ZONE,
    expires_at TIMESTAMP WITH TIME ZONE,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Identity resolution suggestions for manual review
CREATE TABLE artist_merge_suggestions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    target_artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    confidence_score FLOAT NOT NULL CHECK (confidence_score >= 0.0 AND confidence_score <= 1.0),
    match_reasons JSONB NOT NULL DEFAULT '[]', -- [{reason: "name_match", score: 0.9}, ...]
    resolution_status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'approved', 'rejected', 'auto_merged'
    resolved_by UUID REFERENCES users(id),
    resolved_at TIMESTAMP WITH TIME ZONE,
    sync_run_id UUID REFERENCES platform_sync_runs(id),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT different_artists CHECK (source_artist_id != target_artist_id)
);

-- Batch sync checkpoints for resumable operations
CREATE TABLE sync_checkpoints (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    sync_run_id UUID NOT NULL REFERENCES platform_sync_runs(id) ON DELETE CASCADE,
    checkpoint_type VARCHAR(50) NOT NULL, -- 'artist_list', 'album_scan', 'collab_build'
    last_processed_id VARCHAR(255),
    page_token TEXT,
    offset_value INTEGER DEFAULT 0,
    items_in_batch INTEGER DEFAULT 0,
    total_items INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_sync_runs_platform_status ON platform_sync_runs(platform, status);
CREATE INDEX idx_sync_runs_created_at ON platform_sync_runs(created_at DESC);
CREATE INDEX idx_artist_platform_lookup ON artist_platform_ids(platform, platform_id);
CREATE INDEX idx_artist_platform_artist ON artist_platform_ids(artist_id);
CREATE INDEX idx_artist_platform_verified ON artist_platform_ids(verification_status, last_verified_at);
CREATE INDEX idx_merge_suggestions_status ON artist_merge_suggestions(resolution_status, confidence_score DESC);
CREATE INDEX idx_sync_checkpoints_run ON sync_checkpoints(sync_run_id, checkpoint_type);

-- Seed default platform credentials (with placeholder encrypted values)
INSERT INTO platform_credentials (platform, credential_type, rate_limit_config, base_url, is_active)
VALUES
    ('spotify', 'oauth_client', '{"requests_per_window": 100, "window_seconds": 60, "burst_allowance": 10}', 'https://api.spotify.com/v1', true),
    ('apple', 'api_key', '{"requests_per_window": 1000, "window_seconds": 3600, "burst_allowance": 50}', 'https://api.music.apple.com/v1', true),
    ('tidal', 'oauth_client', '{"requests_per_window": 500, "window_seconds": 300, "burst_allowance": 20}', 'https://openapi.tidal.com/v2', true),
    ('youtube', 'api_key', '{"requests_per_window": 10000, "window_seconds": 86400, "burst_allowance": 100}', 'https://www.googleapis.com/youtube/v3', true),
    ('deezer', 'oauth_client', '{"requests_per_window": 50, "window_seconds": 5, "burst_allowance": 5}', 'https://api.deezer.com', true),
    ('musicbrainz', 'api_key', '{"requests_per_window": 50, "window_seconds": 60, "burst_allowance": 1}', 'https://musicbrainz.org/ws/2', true)
ON CONFLICT (platform) DO NOTHING;
