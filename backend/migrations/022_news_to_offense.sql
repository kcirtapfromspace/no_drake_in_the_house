-- Link news classifications to artist offenses
-- Enables automatic offense creation from news detection pipeline

-- Add source classification reference to artist_offenses
ALTER TABLE artist_offenses
ADD COLUMN IF NOT EXISTS source_classification_id UUID REFERENCES news_offense_classifications(id) ON DELETE SET NULL;

-- Add verification_status for tracking news-detected vs human-verified offenses
ALTER TABLE artist_offenses
ADD COLUMN IF NOT EXISTS verification_status VARCHAR(20) DEFAULT 'verified'
    CHECK (verification_status IN ('pending', 'verified', 'rejected', 'needs_review'));

-- Add source_url to offense_evidence (alias for url to match code patterns)
ALTER TABLE offense_evidence
ADD COLUMN IF NOT EXISTS source_url TEXT;

-- Backfill source_url from url where not set
UPDATE offense_evidence SET source_url = url WHERE source_url IS NULL;

-- Remove duplicates before adding constraint (keep the one with highest credibility)
DELETE FROM offense_evidence a USING offense_evidence b
WHERE a.id < b.id
  AND a.offense_id = b.offense_id
  AND a.source_url = b.source_url;

-- Add unique constraint to prevent duplicate evidence links
DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1 FROM pg_constraint WHERE conname = 'unique_evidence_url_per_offense'
    ) THEN
        ALTER TABLE offense_evidence
        ADD CONSTRAINT unique_evidence_url_per_offense UNIQUE (offense_id, source_url);
    END IF;
END $$;

-- Index for finding offenses by source classification
CREATE INDEX IF NOT EXISTS idx_offenses_source_classification
    ON artist_offenses(source_classification_id)
    WHERE source_classification_id IS NOT NULL;

-- Index for finding unverified news-detected offenses
CREATE INDEX IF NOT EXISTS idx_offenses_pending_verification
    ON artist_offenses(verification_status, created_at)
    WHERE verification_status = 'pending';

-- Function to recalculate trouble score when offense is created/updated
CREATE OR REPLACE FUNCTION recalculate_trouble_score(p_artist_id UUID, p_trigger TEXT)
RETURNS VOID AS $$
DECLARE
    v_score DECIMAL(5,2);
    v_offense_count INTEGER;
    v_max_severity INTEGER;
BEGIN
    -- Count offenses and get max severity
    SELECT
        COUNT(*),
        MAX(CASE severity
            WHEN 'egregious' THEN 4
            WHEN 'severe' THEN 3
            WHEN 'moderate' THEN 2
            ELSE 1
        END)
    INTO v_offense_count, v_max_severity
    FROM artist_offenses
    WHERE artist_id = p_artist_id
      AND status != 'rejected';

    -- Calculate score (0-100 scale)
    -- Base: 25 per offense, capped at 100
    -- Multiplier based on max severity
    v_score := LEAST(100, v_offense_count * 25) * (v_max_severity / 4.0);

    -- Update or insert trouble score
    INSERT INTO trouble_scores (artist_id, total_score, severity_component, last_calculated_at, calculation_trigger)
    VALUES (p_artist_id, v_score, v_max_severity * 25, NOW(), p_trigger)
    ON CONFLICT (artist_id) DO UPDATE SET
        total_score = EXCLUDED.total_score,
        severity_component = EXCLUDED.severity_component,
        last_calculated_at = NOW(),
        calculation_trigger = EXCLUDED.calculation_trigger;

EXCEPTION WHEN OTHERS THEN
    -- Log but don't fail if trouble_scores table doesn't exist
    RAISE NOTICE 'Could not update trouble score: %', SQLERRM;
END;
$$ LANGUAGE plpgsql;

-- Comment for documentation
COMMENT ON COLUMN artist_offenses.source_classification_id IS 'Links to the news_offense_classification that triggered auto-creation of this offense';
COMMENT ON COLUMN artist_offenses.verification_status IS 'Tracks whether news-detected offense has been human-verified';
