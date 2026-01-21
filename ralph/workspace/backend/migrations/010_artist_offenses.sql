-- Artist Offenses and Evidence Tracking
-- This creates a curated database of artist misconduct with supporting evidence

-- Offense categories
CREATE TYPE offense_category AS ENUM (
    'domestic_violence',
    'sexual_misconduct',
    'sexual_assault',
    'child_abuse',
    'hate_speech',
    'racism',
    'homophobia',
    'antisemitism',
    'violent_crime',
    'drug_trafficking',
    'fraud',
    'animal_abuse',
    'other'
);

-- Verification status for evidence
CREATE TYPE evidence_status AS ENUM (
    'pending',      -- Submitted, awaiting review
    'verified',     -- Confirmed by multiple sources
    'disputed',     -- Contested, needs more review
    'rejected'      -- False or unsubstantiated
);

-- Severity levels
CREATE TYPE offense_severity AS ENUM (
    'minor',        -- Controversial statements
    'moderate',     -- Arrests, allegations
    'severe',       -- Convictions, proven abuse
    'egregious'     -- Multiple severe offenses, ongoing patterns
);

-- Main offenses table - tracks documented incidents
CREATE TABLE artist_offenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,

    -- Offense details
    category offense_category NOT NULL,
    severity offense_severity NOT NULL DEFAULT 'moderate',
    title VARCHAR(255) NOT NULL,
    description TEXT NOT NULL,

    -- When it happened
    incident_date DATE,
    incident_date_approximate BOOLEAN DEFAULT FALSE,

    -- Legal outcomes
    arrested BOOLEAN DEFAULT FALSE,
    charged BOOLEAN DEFAULT FALSE,
    convicted BOOLEAN DEFAULT FALSE,
    settled BOOLEAN DEFAULT FALSE,

    -- Verification
    status evidence_status NOT NULL DEFAULT 'pending',
    verified_at TIMESTAMPTZ,
    verified_by UUID REFERENCES users(id),

    -- Metadata
    submitted_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    -- Prevent duplicate entries for same incident
    CONSTRAINT unique_offense UNIQUE (artist_id, category, incident_date, title)
);

-- Evidence links - news articles, court docs, etc.
CREATE TABLE offense_evidence (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    offense_id UUID NOT NULL REFERENCES artist_offenses(id) ON DELETE CASCADE,

    -- Source info
    url TEXT NOT NULL,
    source_name VARCHAR(255),  -- "New York Times", "TMZ", "Court Records", etc.
    source_type VARCHAR(50),   -- "news", "court", "video", "social_media", "official"

    -- Content
    title VARCHAR(500),
    excerpt TEXT,              -- Relevant quote from the article
    published_date DATE,
    archived_url TEXT,         -- Archive.org backup

    -- Credibility
    is_primary_source BOOLEAN DEFAULT FALSE,
    credibility_score INTEGER CHECK (credibility_score >= 1 AND credibility_score <= 5),

    -- Metadata
    submitted_by UUID REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- User library inventory - tracks what music users have
CREATE TABLE user_library_tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,

    -- Track info
    provider VARCHAR(50) NOT NULL,  -- 'spotify', 'apple', etc.
    provider_track_id VARCHAR(255) NOT NULL,
    track_name VARCHAR(500),
    album_name VARCHAR(500),

    -- Artist linkage
    artist_id UUID REFERENCES artists(id),
    artist_name VARCHAR(255),

    -- Where in library
    source_type VARCHAR(50),  -- 'saved', 'playlist', 'liked', 'album'
    playlist_name VARCHAR(255),

    -- Sync metadata
    added_at TIMESTAMPTZ,
    last_synced TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_track UNIQUE (user_id, provider, provider_track_id)
);

-- Library scan results - cached analysis
CREATE TABLE library_scan_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,

    -- Stats
    total_tracks INTEGER NOT NULL DEFAULT 0,
    total_artists INTEGER NOT NULL DEFAULT 0,
    flagged_artists INTEGER NOT NULL DEFAULT 0,
    flagged_tracks INTEGER NOT NULL DEFAULT 0,

    -- Breakdown by severity
    egregious_count INTEGER NOT NULL DEFAULT 0,
    severe_count INTEGER NOT NULL DEFAULT 0,
    moderate_count INTEGER NOT NULL DEFAULT 0,
    minor_count INTEGER NOT NULL DEFAULT 0,

    -- Timing
    scan_started_at TIMESTAMPTZ NOT NULL,
    scan_completed_at TIMESTAMPTZ,

    CONSTRAINT unique_user_provider_scan UNIQUE (user_id, provider)
);

-- Pre-populate with some well-documented cases
-- These are public knowledge from court records and major news outlets
-- Only insert if the artist doesn't already exist
INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'R. Kelly', '{"spotify": "2mxe0TnaNL039ysAj51xPQ"}'::jsonb, '{"genres": ["r&b", "soul"]}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'R. Kelly');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'Chris Brown', '{"spotify": "7bXgB6jMjp9ATFy66eO08Z"}'::jsonb, '{"genres": ["r&b", "pop"]}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'Chris Brown');

INSERT INTO artists (canonical_name, external_ids, metadata)
SELECT 'XXXTentacion', '{"spotify": "15UsOTVnJzReFVN1VCnxy4"}'::jsonb, '{"genres": ["hip hop", "emo rap"]}'::jsonb
WHERE NOT EXISTS (SELECT 1 FROM artists WHERE canonical_name = 'XXXTentacion');

-- Indexes for performance
CREATE INDEX idx_offenses_artist ON artist_offenses(artist_id);
CREATE INDEX idx_offenses_category ON artist_offenses(category);
CREATE INDEX idx_offenses_severity ON artist_offenses(severity);
CREATE INDEX idx_offenses_status ON artist_offenses(status);
CREATE INDEX idx_evidence_offense ON offense_evidence(offense_id);
CREATE INDEX idx_library_user ON user_library_tracks(user_id);
CREATE INDEX idx_library_artist ON user_library_tracks(artist_id);
CREATE INDEX idx_scan_user ON library_scan_results(user_id);

-- Trigger to update timestamps
CREATE OR REPLACE FUNCTION update_offense_timestamp()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER offense_updated
    BEFORE UPDATE ON artist_offenses
    FOR EACH ROW
    EXECUTE FUNCTION update_offense_timestamp();
