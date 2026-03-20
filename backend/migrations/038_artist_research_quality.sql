-- Artist research quality tracking for autoresearch loop
CREATE TABLE IF NOT EXISTS artist_research_quality (
    artist_id UUID PRIMARY KEY REFERENCES artists(id) ON DELETE CASCADE,
    quality_score DECIMAL(5,2) NOT NULL DEFAULT 0,
    source_diversity_score DECIMAL(5,2),
    temporal_coverage_score DECIMAL(5,2),
    corroboration_score DECIMAL(5,2),
    confidence_score DECIMAL(5,2),
    completeness_score DECIMAL(5,2),
    sources_searched JSONB DEFAULT '[]'::jsonb,
    last_research_at TIMESTAMPTZ,
    research_iterations INTEGER DEFAULT 0,
    needs_more_research BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_research_quality_score
ON artist_research_quality(quality_score);

CREATE INDEX IF NOT EXISTS idx_research_needs_more
ON artist_research_quality(needs_more_research) WHERE needs_more_research = TRUE;

CREATE INDEX IF NOT EXISTS idx_research_last_at
ON artist_research_quality(last_research_at);
