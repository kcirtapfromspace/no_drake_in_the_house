-- Playcount and Revenue Tracking
-- Tracks user streaming activity and calculates revenue contribution to artists

-- Platform payout rates (updated periodically from industry reports)
CREATE TABLE platform_payout_rates (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL,

    -- Payout rates
    rate_per_stream DECIMAL(10, 6) NOT NULL,      -- e.g., 0.003 for Spotify
    rate_per_minute DECIMAL(10, 6),               -- Alternative time-based rate (YouTube)
    subscription_monthly DECIMAL(10, 2),          -- User subscription cost ($9.99, $10.99, etc.)

    -- Rate tiers (some platforms pay more for premium subscribers)
    rate_tier VARCHAR(50) DEFAULT 'standard',     -- 'free', 'standard', 'hifi', 'family'

    -- Validity period
    effective_date DATE NOT NULL,
    end_date DATE,                                -- NULL means currently active

    -- Source and notes
    source_url TEXT,                              -- Citation for rate
    notes TEXT,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_platform_rate UNIQUE (platform, rate_tier, effective_date)
);

-- Insert known payout rates (as of 2024)
-- Sources: Digital Music News, Music Business Worldwide, platform reports
INSERT INTO platform_payout_rates (platform, rate_per_stream, rate_per_minute, subscription_monthly, rate_tier, effective_date, source_url) VALUES
('spotify', 0.003, NULL, 10.99, 'standard', '2024-01-01', 'https://www.digitalmusicnews.com/spotify-payout-rates'),
('spotify', 0.004, NULL, 10.99, 'premium', '2024-01-01', 'https://www.digitalmusicnews.com/spotify-payout-rates'),
('apple_music', 0.01, NULL, 10.99, 'standard', '2024-01-01', 'https://www.apple.com/apple-music'),
('tidal', 0.0125, NULL, 10.99, 'standard', '2024-01-01', 'https://tidal.com'),
('tidal', 0.02, NULL, 19.99, 'hifi', '2024-01-01', 'https://tidal.com'),
('youtube_music', 0.002, 0.00007, 10.99, 'standard', '2024-01-01', 'https://music.youtube.com'),
('deezer', 0.0064, NULL, 10.99, 'standard', '2024-01-01', 'https://www.deezer.com'),
('amazon_music', 0.004, NULL, 9.99, 'standard', '2024-01-01', 'https://music.amazon.com'),
('pandora', 0.0013, NULL, 9.99, 'standard', '2024-01-01', 'https://www.pandora.com');

-- User playcount snapshots - tracks listening per artist per period
CREATE TABLE user_artist_playcounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,

    -- Play metrics
    play_count INTEGER NOT NULL DEFAULT 0,
    listening_time_ms BIGINT DEFAULT 0,          -- Total milliseconds listened
    unique_tracks_played INTEGER DEFAULT 0,

    -- Time period
    period_type VARCHAR(20) NOT NULL,            -- 'daily', 'weekly', 'monthly'
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,

    -- Calculated revenue
    estimated_revenue DECIMAL(10, 4) DEFAULT 0,
    rate_used DECIMAL(10, 6),                    -- The rate that was applied

    -- Sync metadata
    synced_from_api BOOLEAN DEFAULT FALSE,
    api_response_id VARCHAR(255),                -- For deduplication
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_artist_period UNIQUE (user_id, artist_id, platform, period_type, period_start)
);

-- Aggregated artist revenue across all users
CREATE TABLE artist_revenue_summary (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,

    -- Aggregate metrics
    total_streams BIGINT DEFAULT 0,
    total_listening_time_ms BIGINT DEFAULT 0,
    total_revenue DECIMAL(12, 4) DEFAULT 0,
    unique_listeners INTEGER DEFAULT 0,

    -- Period
    period_type VARCHAR(20) NOT NULL,            -- 'daily', 'weekly', 'monthly', 'alltime'
    period_date DATE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_artist_revenue_period UNIQUE (artist_id, platform, period_type, period_date)
);

-- User revenue distribution - shows where their money goes
CREATE TABLE user_revenue_distribution (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,

    -- Totals for this user
    total_streams BIGINT DEFAULT 0,
    total_revenue DECIMAL(10, 4) DEFAULT 0,
    subscription_cost DECIMAL(10, 2),

    -- Breakdown
    revenue_to_clean_artists DECIMAL(10, 4) DEFAULT 0,        -- Artists with low/no trouble score
    revenue_to_problematic_artists DECIMAL(10, 4) DEFAULT 0,  -- Artists with high trouble score
    problematic_percentage FLOAT DEFAULT 0,

    -- Period
    period_type VARCHAR(20) NOT NULL,
    period_start DATE NOT NULL,
    period_end DATE NOT NULL,

    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_revenue_period UNIQUE (user_id, platform, period_type, period_start)
);

-- Top problematic artists for each user (pre-computed for dashboard)
CREATE TABLE user_top_problematic_artists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,

    -- Rank
    rank INTEGER NOT NULL,                       -- 1 = highest revenue to problematic artist

    -- Revenue details
    play_count INTEGER DEFAULT 0,
    estimated_revenue DECIMAL(10, 4) DEFAULT 0,
    percentage_of_user_spend FLOAT DEFAULT 0,

    -- Trouble info
    trouble_tier trouble_tier,
    trouble_score FLOAT,

    -- Period
    period_type VARCHAR(20) NOT NULL,
    period_date DATE NOT NULL,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_top_artist UNIQUE (user_id, artist_id, period_type, period_date)
);

-- Indexes for performance
CREATE INDEX idx_playcounts_user ON user_artist_playcounts(user_id, period_start DESC);
CREATE INDEX idx_playcounts_artist ON user_artist_playcounts(artist_id);
CREATE INDEX idx_playcounts_platform ON user_artist_playcounts(platform, period_start);
CREATE INDEX idx_playcounts_user_period ON user_artist_playcounts(user_id, period_type, period_start);

CREATE INDEX idx_revenue_summary_artist ON artist_revenue_summary(artist_id, period_date DESC);
CREATE INDEX idx_revenue_summary_platform ON artist_revenue_summary(platform, period_type, period_date);

CREATE INDEX idx_user_revenue_user ON user_revenue_distribution(user_id, period_start DESC);
CREATE INDEX idx_user_revenue_problematic ON user_revenue_distribution(problematic_percentage DESC);

CREATE INDEX idx_top_problematic_user ON user_top_problematic_artists(user_id, period_date DESC, rank);
CREATE INDEX idx_top_problematic_artist ON user_top_problematic_artists(artist_id);

CREATE INDEX idx_payout_rates_platform ON platform_payout_rates(platform, effective_date DESC);

-- Triggers for timestamp updates
CREATE OR REPLACE FUNCTION update_playcount_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER playcount_updated
    BEFORE UPDATE ON user_artist_playcounts
    FOR EACH ROW
    EXECUTE FUNCTION update_playcount_timestamp();

CREATE TRIGGER revenue_summary_updated
    BEFORE UPDATE ON artist_revenue_summary
    FOR EACH ROW
    EXECUTE FUNCTION update_playcount_timestamp();

-- Function to get current payout rate for a platform
CREATE OR REPLACE FUNCTION get_payout_rate(p_platform VARCHAR, p_tier VARCHAR DEFAULT 'standard')
RETURNS DECIMAL AS $$
DECLARE
    rate DECIMAL;
BEGIN
    SELECT rate_per_stream INTO rate
    FROM platform_payout_rates
    WHERE platform = p_platform
    AND rate_tier = p_tier
    AND effective_date <= CURRENT_DATE
    AND (end_date IS NULL OR end_date >= CURRENT_DATE)
    ORDER BY effective_date DESC
    LIMIT 1;

    RETURN COALESCE(rate, 0.003); -- Default to Spotify rate if not found
END;
$$ LANGUAGE plpgsql;

-- Function to calculate revenue for a playcount record
CREATE OR REPLACE FUNCTION calculate_playcount_revenue(p_playcount_id UUID)
RETURNS DECIMAL AS $$
DECLARE
    v_platform VARCHAR;
    v_play_count INTEGER;
    v_rate DECIMAL;
    v_revenue DECIMAL;
BEGIN
    SELECT platform, play_count INTO v_platform, v_play_count
    FROM user_artist_playcounts
    WHERE id = p_playcount_id;

    v_rate := get_payout_rate(v_platform);
    v_revenue := v_play_count * v_rate;

    UPDATE user_artist_playcounts
    SET estimated_revenue = v_revenue, rate_used = v_rate
    WHERE id = p_playcount_id;

    RETURN v_revenue;
END;
$$ LANGUAGE plpgsql;

-- Function to recalculate user's revenue distribution
CREATE OR REPLACE FUNCTION recalculate_user_revenue(
    p_user_id UUID,
    p_platform VARCHAR,
    p_period_type VARCHAR,
    p_period_start DATE,
    p_period_end DATE
)
RETURNS user_revenue_distribution AS $$
DECLARE
    v_total_streams BIGINT;
    v_total_revenue DECIMAL;
    v_clean_revenue DECIMAL;
    v_problematic_revenue DECIMAL;
    v_subscription DECIMAL;
    result user_revenue_distribution;
BEGIN
    -- Get user's subscription cost
    SELECT subscription_monthly INTO v_subscription
    FROM platform_payout_rates
    WHERE platform = p_platform
    AND effective_date <= CURRENT_DATE
    AND (end_date IS NULL OR end_date >= CURRENT_DATE)
    ORDER BY effective_date DESC
    LIMIT 1;

    -- Calculate total streams and revenue
    SELECT
        COALESCE(SUM(play_count), 0),
        COALESCE(SUM(estimated_revenue), 0)
    INTO v_total_streams, v_total_revenue
    FROM user_artist_playcounts
    WHERE user_id = p_user_id
    AND platform = p_platform
    AND period_type = p_period_type
    AND period_start >= p_period_start
    AND period_end <= p_period_end;

    -- Calculate revenue to clean vs problematic artists
    -- Clean = trouble_tier in ('low', NULL)
    -- Problematic = trouble_tier in ('moderate', 'high', 'critical')
    SELECT
        COALESCE(SUM(CASE WHEN ts.trouble_tier IS NULL OR ts.trouble_tier = 'low' THEN pc.estimated_revenue ELSE 0 END), 0),
        COALESCE(SUM(CASE WHEN ts.trouble_tier IN ('moderate', 'high', 'critical') THEN pc.estimated_revenue ELSE 0 END), 0)
    INTO v_clean_revenue, v_problematic_revenue
    FROM user_artist_playcounts pc
    LEFT JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
    WHERE pc.user_id = p_user_id
    AND pc.platform = p_platform
    AND pc.period_type = p_period_type
    AND pc.period_start >= p_period_start
    AND pc.period_end <= p_period_end;

    -- Upsert distribution record
    INSERT INTO user_revenue_distribution (
        user_id, platform, total_streams, total_revenue, subscription_cost,
        revenue_to_clean_artists, revenue_to_problematic_artists, problematic_percentage,
        period_type, period_start, period_end
    ) VALUES (
        p_user_id, p_platform, v_total_streams, v_total_revenue, v_subscription,
        v_clean_revenue, v_problematic_revenue,
        CASE WHEN v_total_revenue > 0 THEN (v_problematic_revenue / v_total_revenue * 100) ELSE 0 END,
        p_period_type, p_period_start, p_period_end
    )
    ON CONFLICT (user_id, platform, period_type, period_start) DO UPDATE SET
        total_streams = EXCLUDED.total_streams,
        total_revenue = EXCLUDED.total_revenue,
        subscription_cost = EXCLUDED.subscription_cost,
        revenue_to_clean_artists = EXCLUDED.revenue_to_clean_artists,
        revenue_to_problematic_artists = EXCLUDED.revenue_to_problematic_artists,
        problematic_percentage = EXCLUDED.problematic_percentage,
        calculated_at = NOW()
    RETURNING * INTO result;

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Function to update user's top problematic artists
CREATE OR REPLACE FUNCTION update_user_top_problematic_artists(
    p_user_id UUID,
    p_period_type VARCHAR DEFAULT 'monthly',
    p_period_date DATE DEFAULT CURRENT_DATE,
    p_limit INTEGER DEFAULT 10
)
RETURNS VOID AS $$
BEGIN
    -- Delete old rankings for this period
    DELETE FROM user_top_problematic_artists
    WHERE user_id = p_user_id
    AND period_type = p_period_type
    AND period_date = p_period_date;

    -- Insert new rankings
    INSERT INTO user_top_problematic_artists (
        user_id, artist_id, rank, play_count, estimated_revenue,
        percentage_of_user_spend, trouble_tier, trouble_score, period_type, period_date
    )
    SELECT
        p_user_id,
        pc.artist_id,
        ROW_NUMBER() OVER (ORDER BY SUM(pc.estimated_revenue) DESC),
        SUM(pc.play_count),
        SUM(pc.estimated_revenue),
        CASE
            WHEN total.total_revenue > 0
            THEN (SUM(pc.estimated_revenue) / total.total_revenue * 100)
            ELSE 0
        END,
        ts.trouble_tier,
        ts.total_score,
        p_period_type,
        p_period_date
    FROM user_artist_playcounts pc
    INNER JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
    CROSS JOIN (
        SELECT COALESCE(SUM(estimated_revenue), 1) as total_revenue
        FROM user_artist_playcounts
        WHERE user_id = p_user_id
        AND period_type = p_period_type
        AND DATE_TRUNC('month', period_start) = DATE_TRUNC('month', p_period_date)
    ) total
    WHERE pc.user_id = p_user_id
    AND pc.period_type = p_period_type
    AND DATE_TRUNC('month', pc.period_start) = DATE_TRUNC('month', p_period_date)
    AND ts.trouble_tier IN ('moderate', 'high', 'critical')
    GROUP BY pc.artist_id, ts.trouble_tier, ts.total_score, total.total_revenue
    ORDER BY SUM(pc.estimated_revenue) DESC
    LIMIT p_limit;
END;
$$ LANGUAGE plpgsql;

-- View for easy dashboard queries
CREATE OR REPLACE VIEW v_user_streaming_impact AS
SELECT
    u.id as user_id,
    u.email,
    urd.platform,
    urd.total_streams,
    urd.total_revenue,
    urd.subscription_cost,
    urd.revenue_to_clean_artists,
    urd.revenue_to_problematic_artists,
    urd.problematic_percentage,
    urd.period_type,
    urd.period_start,
    urd.period_end,
    (
        SELECT COUNT(DISTINCT artist_id)
        FROM user_artist_playcounts pc
        INNER JOIN artist_trouble_scores ts ON ts.artist_id = pc.artist_id
        WHERE pc.user_id = u.id
        AND ts.trouble_tier IN ('moderate', 'high', 'critical')
    ) as problematic_artist_count
FROM users u
LEFT JOIN user_revenue_distribution urd ON urd.user_id = u.id;

-- View for artist revenue leaderboard
CREATE OR REPLACE VIEW v_artist_revenue_leaderboard AS
SELECT
    a.id as artist_id,
    a.canonical_name,
    ts.trouble_tier,
    ts.total_score as trouble_score,
    ars.platform,
    ars.total_streams,
    ars.total_revenue,
    ars.unique_listeners,
    ars.period_type,
    ars.period_date
FROM artists a
LEFT JOIN artist_trouble_scores ts ON ts.artist_id = a.id
LEFT JOIN artist_revenue_summary ars ON ars.artist_id = a.id
WHERE ts.trouble_tier IS NOT NULL
ORDER BY ars.total_revenue DESC NULLS LAST;
