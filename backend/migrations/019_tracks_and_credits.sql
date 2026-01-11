-- Tracks and Credits Schema
-- Stores song credits, collaborations, and production information

-- Credit role type
CREATE TYPE credit_role AS ENUM (
    'primary_artist',
    'featured_artist',
    'producer',
    'writer',
    'composer',
    'lyricist',
    'arranger',
    'mixer',
    'mastering_engineer',
    'recording_engineer',
    'background_vocalist',
    'instrumentalist',
    'remixer',
    'sample_credit',
    'other'
);

-- Albums/Releases table
CREATE TABLE albums (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    release_date DATE,
    release_year INTEGER,
    album_type VARCHAR(50), -- 'album', 'single', 'ep', 'compilation'
    total_tracks INTEGER,
    label VARCHAR(255),
    upc VARCHAR(50),
    cover_art_url TEXT,
    -- Platform IDs
    apple_music_id VARCHAR(100),
    spotify_id VARCHAR(100),
    deezer_id VARCHAR(100),
    -- Metadata
    genres TEXT[],
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Primary artist for album
CREATE TABLE album_artists (
    album_id UUID NOT NULL REFERENCES albums(id) ON DELETE CASCADE,
    artist_id UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    is_primary BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    PRIMARY KEY (album_id, artist_id)
);

-- Tracks/Songs table
CREATE TABLE tracks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    title VARCHAR(500) NOT NULL,
    album_id UUID REFERENCES albums(id) ON DELETE SET NULL,
    track_number INTEGER,
    disc_number INTEGER DEFAULT 1,
    duration_ms INTEGER,
    explicit BOOLEAN DEFAULT false,
    isrc VARCHAR(20),
    preview_url TEXT,
    -- Platform IDs
    apple_music_id VARCHAR(100),
    spotify_id VARCHAR(100),
    deezer_id VARCHAR(100),
    -- Popularity/stats
    popularity INTEGER,
    play_count BIGINT,
    -- Metadata
    genres TEXT[],
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Track credits - who worked on each track
CREATE TABLE track_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    track_id UUID NOT NULL REFERENCES tracks(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE SET NULL,
    -- For credits where we don't have a matched artist yet
    credited_name VARCHAR(255) NOT NULL,
    role credit_role NOT NULL,
    role_detail VARCHAR(255), -- Additional detail like "drums", "guitar", etc.
    -- Source tracking
    source_platform VARCHAR(50), -- 'apple_music', 'spotify', 'discogs', etc.
    source_verified BOOLEAN DEFAULT false,
    confidence FLOAT DEFAULT 0.5,
    -- Metadata
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_track_credit UNIQUE (track_id, artist_id, credited_name, role)
);

-- Derived artist collaborations (for graph analysis)
CREATE TABLE artist_collaborations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id_1 UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    artist_id_2 UUID NOT NULL REFERENCES artists(id) ON DELETE CASCADE,
    collaboration_type VARCHAR(50) NOT NULL, -- 'featured', 'producer', 'writer', 'sample'
    track_count INTEGER DEFAULT 1,
    first_collab_date DATE,
    last_collab_date DATE,
    -- Sample tracks for reference
    sample_track_ids UUID[] DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_collab UNIQUE (artist_id_1, artist_id_2, collaboration_type),
    CONSTRAINT ordered_artists CHECK (artist_id_1 < artist_id_2)
);

-- Credits sync tracking
CREATE TABLE credits_sync_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    platform VARCHAR(50) NOT NULL,
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'running', 'completed', 'failed'
    albums_processed INTEGER DEFAULT 0,
    tracks_processed INTEGER DEFAULT 0,
    credits_added INTEGER DEFAULT 0,
    errors_count INTEGER DEFAULT 0,
    error_log JSONB DEFAULT '[]',
    started_at TIMESTAMP WITH TIME ZONE,
    completed_at TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_albums_apple_id ON albums(apple_music_id) WHERE apple_music_id IS NOT NULL;
CREATE INDEX idx_albums_spotify_id ON albums(spotify_id) WHERE spotify_id IS NOT NULL;
CREATE INDEX idx_albums_deezer_id ON albums(deezer_id) WHERE deezer_id IS NOT NULL;
CREATE INDEX idx_albums_release_date ON albums(release_date DESC);

CREATE INDEX idx_tracks_album ON tracks(album_id);
CREATE INDEX idx_tracks_apple_id ON tracks(apple_music_id) WHERE apple_music_id IS NOT NULL;
CREATE INDEX idx_tracks_spotify_id ON tracks(spotify_id) WHERE spotify_id IS NOT NULL;
CREATE INDEX idx_tracks_isrc ON tracks(isrc) WHERE isrc IS NOT NULL;

CREATE INDEX idx_track_credits_track ON track_credits(track_id);
CREATE INDEX idx_track_credits_artist ON track_credits(artist_id) WHERE artist_id IS NOT NULL;
CREATE INDEX idx_track_credits_role ON track_credits(role);
CREATE INDEX idx_track_credits_name ON track_credits(credited_name);

CREATE INDEX idx_artist_collabs_artist1 ON artist_collaborations(artist_id_1);
CREATE INDEX idx_artist_collabs_artist2 ON artist_collaborations(artist_id_2);
CREATE INDEX idx_artist_collabs_type ON artist_collaborations(collaboration_type);

CREATE INDEX idx_credits_sync_artist ON credits_sync_runs(artist_id, platform);
CREATE INDEX idx_credits_sync_status ON credits_sync_runs(status);

-- Function to update collaboration counts
CREATE OR REPLACE FUNCTION update_collaboration_counts()
RETURNS TRIGGER AS $$
BEGIN
    -- Update collaboration record when credits change
    -- This runs after insert/update/delete on track_credits
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
