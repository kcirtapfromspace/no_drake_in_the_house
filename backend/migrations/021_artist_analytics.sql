-- Artist Analytics Schema
-- Stores streaming data and estimated revenue for revenue impact calculations

-- Platform streaming rates (approximate revenue per stream)
-- These are industry averages and may vary by contract
CREATE TABLE platform_streaming_rates (
    platform VARCHAR(50) PRIMARY KEY,
    rate_per_stream DECIMAL(10, 6) NOT NULL,
    rate_currency VARCHAR(3) DEFAULT 'USD',
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    notes TEXT
);

-- Insert default rates (2024 industry averages)
INSERT INTO platform_streaming_rates (platform, rate_per_stream, notes) VALUES
    ('apple_music', 0.0100, 'Apple Music pays ~$0.01 per stream on average'),
    ('spotify', 0.0035, 'Spotify pays ~$0.003-0.005 per stream'),
    ('tidal', 0.0125, 'Tidal pays ~$0.0125 per stream (higher quality tier)'),
    ('youtube_music', 0.0020, 'YouTube Music pays ~$0.002 per stream'),
    ('deezer', 0.0064, 'Deezer pays ~$0.0064 per stream'),
    ('amazon_music', 0.0040, 'Amazon Music pays ~$0.004 per stream');

-- Artist streaming stats (aggregated from various sources)
CREATE TABLE artist_streaming_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    -- Monthly stats
    monthly_streams BIGINT,
    monthly_listeners BIGINT,
    -- All-time stats
    total_streams BIGINT,
    -- Popularity metrics
    popularity_score INTEGER, -- 0-100 scale
    chart_peak_position INTEGER,
    -- Revenue estimates
    estimated_monthly_revenue DECIMAL(12, 2),
    estimated_total_revenue DECIMAL(14, 2),
    -- Data freshness
    data_source VARCHAR(100), -- 'apple_music_api', 'chartmetric', 'manual', etc.
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_artist_platform_stats UNIQUE (artist_id, platform)
);

-- Track streaming stats
CREATE TABLE track_streaming_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    track_id UUID NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    -- Stream counts
    total_streams BIGINT,
    streams_last_30_days BIGINT,
    -- Popularity
    popularity_score INTEGER,
    -- Revenue
    estimated_revenue DECIMAL(12, 2),
    -- Metadata
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    CONSTRAINT unique_track_platform_stats UNIQUE (track_id, platform)
);

-- Indexes for efficient queries
CREATE INDEX idx_artist_stats_artist ON artist_streaming_stats(artist_id);
CREATE INDEX idx_artist_stats_platform ON artist_streaming_stats(platform);
CREATE INDEX idx_artist_stats_revenue ON artist_streaming_stats(estimated_monthly_revenue DESC);
CREATE INDEX idx_track_stats_track ON track_streaming_stats(track_id);
CREATE INDEX idx_track_stats_streams ON track_streaming_stats(total_streams DESC);

-- View for easy artist revenue summary
CREATE OR REPLACE VIEW artist_revenue_summary_view AS
SELECT
    a.id AS artist_id,
    a.canonical_name,
    COALESCE(SUM(ass.monthly_streams), 0) AS total_monthly_streams,
    COALESCE(SUM(ass.total_streams), 0) AS total_all_time_streams,
    COALESCE(SUM(ass.estimated_monthly_revenue), 0) AS total_monthly_revenue,
    COALESCE(SUM(ass.estimated_total_revenue), 0) AS total_all_time_revenue,
    COUNT(DISTINCT ass.platform) AS platforms_with_data,
    MAX(ass.last_updated) AS stats_last_updated
FROM artists a
LEFT JOIN artist_streaming_stats ass ON a.id = ass.artist_id
GROUP BY a.id, a.canonical_name;
