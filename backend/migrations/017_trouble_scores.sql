-- Artist Trouble Score System
-- Aggregates offense data, evidence credibility, community consensus, and revenue impact
-- into a composite "trouble score" for ranking problematic artists

-- Trouble tier enum for categorization
CREATE TYPE trouble_tier AS ENUM (
    'low',       -- Score 0.00-0.24: Minor issues, old incidents
    'moderate',  -- Score 0.25-0.49: Some concerns, mixed evidence
    'high',      -- Score 0.50-0.74: Significant issues, credible evidence
    'critical'   -- Score 0.75-1.00: Severe documented offenses
);

-- Main trouble scores table
CREATE TABLE artist_trouble_scores (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,

    -- Component scores (0.0 - 1.0 normalized)
    severity_score FLOAT NOT NULL DEFAULT 0,      -- Based on max offense severity (35% weight)
    evidence_score FLOAT NOT NULL DEFAULT 0,      -- Average evidence credibility (20% weight)
    recency_score FLOAT NOT NULL DEFAULT 0,       -- How recent the offenses are (15% weight)
    community_score FLOAT NOT NULL DEFAULT 0,     -- User block consensus (15% weight)
    revenue_score FLOAT NOT NULL DEFAULT 0,       -- Revenue contribution factor (15% weight)

    -- Composite score (weighted combination of components)
    total_score FLOAT NOT NULL DEFAULT 0,
    trouble_tier trouble_tier DEFAULT 'low',

    -- Raw metrics for transparency
    offense_count INTEGER DEFAULT 0,
    verified_offense_count INTEGER DEFAULT 0,
    block_count INTEGER DEFAULT 0,              -- Total users blocking this artist
    subscription_block_count INTEGER DEFAULT 0, -- Users blocking via category subscription

    -- Offense breakdown by severity
    egregious_count INTEGER DEFAULT 0,
    severe_count INTEGER DEFAULT 0,
    moderate_count INTEGER DEFAULT 0,
    minor_count INTEGER DEFAULT 0,

    -- Temporal data
    first_offense_date DATE,
    last_offense_date DATE,
    last_calculated_at TIMESTAMPTZ DEFAULT NOW(),

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_artist_trouble_score UNIQUE(artist_id)
);

-- Score calculation history for auditing
CREATE TABLE trouble_score_history (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,

    -- Snapshot of all scores
    severity_score FLOAT NOT NULL,
    evidence_score FLOAT NOT NULL,
    recency_score FLOAT NOT NULL,
    community_score FLOAT NOT NULL,
    revenue_score FLOAT NOT NULL,
    total_score FLOAT NOT NULL,
    trouble_tier trouble_tier NOT NULL,

    -- What triggered recalculation
    trigger_reason VARCHAR(100),  -- 'new_offense', 'evidence_update', 'scheduled', 'manual'

    calculated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Score weights configuration (allows tuning without code changes)
CREATE TABLE trouble_score_weights (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(100) NOT NULL UNIQUE,

    severity_weight FLOAT NOT NULL DEFAULT 0.35,
    evidence_weight FLOAT NOT NULL DEFAULT 0.20,
    recency_weight FLOAT NOT NULL DEFAULT 0.15,
    community_weight FLOAT NOT NULL DEFAULT 0.15,
    revenue_weight FLOAT NOT NULL DEFAULT 0.15,

    is_active BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT weights_sum_check CHECK (
        ABS(severity_weight + evidence_weight + recency_weight + community_weight + revenue_weight - 1.0) < 0.01
    )
);

-- Insert default weight configuration
INSERT INTO trouble_score_weights (name, severity_weight, evidence_weight, recency_weight, community_weight, revenue_weight, is_active)
VALUES ('default', 0.35, 0.20, 0.15, 0.15, 0.15, TRUE);

-- Indexes for efficient queries
CREATE INDEX idx_trouble_scores_artist ON artist_trouble_scores(artist_id);
CREATE INDEX idx_trouble_scores_total ON artist_trouble_scores(total_score DESC);
CREATE INDEX idx_trouble_scores_tier ON artist_trouble_scores(trouble_tier);
CREATE INDEX idx_trouble_scores_block_count ON artist_trouble_scores(block_count DESC);
CREATE INDEX idx_trouble_history_artist ON trouble_score_history(artist_id, calculated_at DESC);

-- Trigger to update timestamps
CREATE OR REPLACE FUNCTION update_trouble_score_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trouble_score_updated
    BEFORE UPDATE ON artist_trouble_scores
    FOR EACH ROW
    EXECUTE FUNCTION update_trouble_score_timestamp();

-- Function to calculate severity score from offenses
CREATE OR REPLACE FUNCTION calculate_severity_score(p_artist_id UUID)
RETURNS FLOAT AS $$
DECLARE
    max_severity offense_severity;
    score FLOAT := 0;
BEGIN
    -- Get the most severe offense for this artist
    SELECT MAX(severity) INTO max_severity
    FROM artist_offenses
    WHERE artist_id = p_artist_id
    AND status IN ('verified', 'pending');

    -- Convert severity to score
    score := CASE max_severity
        WHEN 'egregious' THEN 1.0
        WHEN 'severe' THEN 0.75
        WHEN 'moderate' THEN 0.5
        WHEN 'minor' THEN 0.25
        ELSE 0
    END;

    RETURN score;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate evidence score from credibility ratings
CREATE OR REPLACE FUNCTION calculate_evidence_score(p_artist_id UUID)
RETURNS FLOAT AS $$
DECLARE
    avg_credibility FLOAT;
    verified_count INTEGER;
    total_count INTEGER;
BEGIN
    -- Get average credibility of evidence for this artist's offenses
    SELECT
        COALESCE(AVG(e.credibility_score), 0),
        COUNT(*) FILTER (WHERE o.status = 'verified'),
        COUNT(*)
    INTO avg_credibility, verified_count, total_count
    FROM artist_offenses o
    LEFT JOIN offense_evidence e ON e.offense_id = o.id
    WHERE o.artist_id = p_artist_id;

    IF total_count = 0 THEN
        RETURN 0;
    END IF;

    -- Normalize credibility (1-5 scale) to 0-1
    -- Also factor in verification status
    RETURN (avg_credibility / 5.0) * (0.5 + 0.5 * (verified_count::FLOAT / GREATEST(total_count, 1)));
END;
$$ LANGUAGE plpgsql;

-- Function to calculate recency score (more recent = higher score)
CREATE OR REPLACE FUNCTION calculate_recency_score(p_artist_id UUID)
RETURNS FLOAT AS $$
DECLARE
    most_recent DATE;
    years_ago FLOAT;
BEGIN
    SELECT MAX(incident_date) INTO most_recent
    FROM artist_offenses
    WHERE artist_id = p_artist_id
    AND status IN ('verified', 'pending');

    IF most_recent IS NULL THEN
        RETURN 0;
    END IF;

    years_ago := EXTRACT(EPOCH FROM (CURRENT_DATE - most_recent)) / (365.25 * 24 * 60 * 60);

    -- Decay: 1.0 if <1yr, 0.75 if <3yr, 0.5 if <5yr, 0.25 if <10yr, 0.1 if older
    RETURN CASE
        WHEN years_ago < 1 THEN 1.0
        WHEN years_ago < 3 THEN 0.75
        WHEN years_ago < 5 THEN 0.5
        WHEN years_ago < 10 THEN 0.25
        ELSE 0.1
    END;
END;
$$ LANGUAGE plpgsql;

-- Function to calculate community score from block counts
CREATE OR REPLACE FUNCTION calculate_community_score(p_artist_id UUID)
RETURNS FLOAT AS $$
DECLARE
    block_count INTEGER;
    max_blocks INTEGER;
BEGIN
    -- Count direct blocks for this artist
    SELECT COUNT(*) INTO block_count
    FROM user_artist_blocks
    WHERE artist_id = p_artist_id;

    -- Get max blocks across all artists for normalization
    SELECT COALESCE(MAX(cnt), 1) INTO max_blocks
    FROM (
        SELECT COUNT(*) as cnt
        FROM user_artist_blocks
        GROUP BY artist_id
    ) sub;

    RETURN block_count::FLOAT / max_blocks;
END;
$$ LANGUAGE plpgsql;

-- Main function to recalculate all scores for an artist
CREATE OR REPLACE FUNCTION recalculate_trouble_score(p_artist_id UUID, p_trigger_reason VARCHAR DEFAULT 'manual')
RETURNS artist_trouble_scores AS $$
DECLARE
    result artist_trouble_scores;
    weights trouble_score_weights;
    v_severity FLOAT;
    v_evidence FLOAT;
    v_recency FLOAT;
    v_community FLOAT;
    v_revenue FLOAT := 0; -- Will be populated by revenue service
    v_total FLOAT;
    v_tier trouble_tier;
    v_offense_count INTEGER;
    v_verified_count INTEGER;
    v_block_count INTEGER;
    v_egregious INTEGER;
    v_severe INTEGER;
    v_moderate INTEGER;
    v_minor INTEGER;
    v_first_date DATE;
    v_last_date DATE;
BEGIN
    -- Get active weights
    SELECT * INTO weights FROM trouble_score_weights WHERE is_active = TRUE LIMIT 1;

    -- Calculate component scores
    v_severity := calculate_severity_score(p_artist_id);
    v_evidence := calculate_evidence_score(p_artist_id);
    v_recency := calculate_recency_score(p_artist_id);
    v_community := calculate_community_score(p_artist_id);

    -- Calculate total weighted score
    v_total := v_severity * weights.severity_weight +
               v_evidence * weights.evidence_weight +
               v_recency * weights.recency_weight +
               v_community * weights.community_weight +
               v_revenue * weights.revenue_weight;

    -- Determine tier
    v_tier := CASE
        WHEN v_total >= 0.75 THEN 'critical'
        WHEN v_total >= 0.50 THEN 'high'
        WHEN v_total >= 0.25 THEN 'moderate'
        ELSE 'low'
    END;

    -- Get offense counts
    SELECT
        COUNT(*),
        COUNT(*) FILTER (WHERE status = 'verified'),
        COUNT(*) FILTER (WHERE severity = 'egregious'),
        COUNT(*) FILTER (WHERE severity = 'severe'),
        COUNT(*) FILTER (WHERE severity = 'moderate'),
        COUNT(*) FILTER (WHERE severity = 'minor'),
        MIN(incident_date),
        MAX(incident_date)
    INTO v_offense_count, v_verified_count, v_egregious, v_severe, v_moderate, v_minor, v_first_date, v_last_date
    FROM artist_offenses
    WHERE artist_id = p_artist_id;

    -- Get block count
    SELECT COUNT(*) INTO v_block_count
    FROM user_artist_blocks
    WHERE artist_id = p_artist_id;

    -- Upsert the score record
    INSERT INTO artist_trouble_scores (
        artist_id, severity_score, evidence_score, recency_score, community_score, revenue_score,
        total_score, trouble_tier, offense_count, verified_offense_count, block_count,
        egregious_count, severe_count, moderate_count, minor_count,
        first_offense_date, last_offense_date, last_calculated_at
    ) VALUES (
        p_artist_id, v_severity, v_evidence, v_recency, v_community, v_revenue,
        v_total, v_tier, v_offense_count, v_verified_count, v_block_count,
        v_egregious, v_severe, v_moderate, v_minor,
        v_first_date, v_last_date, NOW()
    )
    ON CONFLICT (artist_id) DO UPDATE SET
        severity_score = EXCLUDED.severity_score,
        evidence_score = EXCLUDED.evidence_score,
        recency_score = EXCLUDED.recency_score,
        community_score = EXCLUDED.community_score,
        revenue_score = EXCLUDED.revenue_score,
        total_score = EXCLUDED.total_score,
        trouble_tier = EXCLUDED.trouble_tier,
        offense_count = EXCLUDED.offense_count,
        verified_offense_count = EXCLUDED.verified_offense_count,
        block_count = EXCLUDED.block_count,
        egregious_count = EXCLUDED.egregious_count,
        severe_count = EXCLUDED.severe_count,
        moderate_count = EXCLUDED.moderate_count,
        minor_count = EXCLUDED.minor_count,
        first_offense_date = EXCLUDED.first_offense_date,
        last_offense_date = EXCLUDED.last_offense_date,
        last_calculated_at = NOW()
    RETURNING * INTO result;

    -- Record history
    INSERT INTO trouble_score_history (
        artist_id, severity_score, evidence_score, recency_score, community_score, revenue_score,
        total_score, trouble_tier, trigger_reason
    ) VALUES (
        p_artist_id, v_severity, v_evidence, v_recency, v_community, v_revenue,
        v_total, v_tier, p_trigger_reason
    );

    RETURN result;
END;
$$ LANGUAGE plpgsql;

-- Trigger to recalculate scores when offenses change
CREATE OR REPLACE FUNCTION trigger_trouble_score_update()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'DELETE' THEN
        PERFORM recalculate_trouble_score(OLD.artist_id, 'offense_deleted');
        RETURN OLD;
    ELSE
        PERFORM recalculate_trouble_score(NEW.artist_id, 'offense_' || LOWER(TG_OP));
        RETURN NEW;
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER offense_triggers_score_update
    AFTER INSERT OR UPDATE OR DELETE ON artist_offenses
    FOR EACH ROW
    EXECUTE FUNCTION trigger_trouble_score_update();

-- Trigger to recalculate when evidence is added
CREATE OR REPLACE FUNCTION trigger_evidence_score_update()
RETURNS TRIGGER AS $$
DECLARE
    v_artist_id UUID;
BEGIN
    SELECT artist_id INTO v_artist_id
    FROM artist_offenses
    WHERE id = COALESCE(NEW.offense_id, OLD.offense_id);

    IF v_artist_id IS NOT NULL THEN
        PERFORM recalculate_trouble_score(v_artist_id, 'evidence_update');
    END IF;

    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER evidence_triggers_score_update
    AFTER INSERT OR UPDATE OR DELETE ON offense_evidence
    FOR EACH ROW
    EXECUTE FUNCTION trigger_evidence_score_update();

-- Initialize scores for all artists with offenses
DO $$
DECLARE
    r RECORD;
BEGIN
    FOR r IN SELECT DISTINCT artist_id FROM artist_offenses
    LOOP
        PERFORM recalculate_trouble_score(r.artist_id, 'migration_init');
    END LOOP;
END $$;
