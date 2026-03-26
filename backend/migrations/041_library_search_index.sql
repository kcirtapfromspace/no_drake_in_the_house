CREATE EXTENSION IF NOT EXISTS pg_trgm;
CREATE INDEX IF NOT EXISTS idx_user_library_tracks_search
ON user_library_tracks USING GIN (
    (COALESCE(track_name, '') || ' ' || COALESCE(artist_name, '') || ' ' || COALESCE(album_name, ''))
    gin_trgm_ops
);
