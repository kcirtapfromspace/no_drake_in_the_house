-- Performance indexes for playlist offense name matching.
-- Split from 039 to avoid migration checksum mismatch.

CREATE INDEX IF NOT EXISTS idx_artists_canonical_lower
    ON artists (LOWER(canonical_name));

CREATE INDEX IF NOT EXISTS idx_playlist_tracks_artist_name_lower
    ON playlist_tracks (LOWER(artist_name))
    WHERE artist_name IS NOT NULL;
