-- no-transaction
-- Index to speed up the Rust get_library_offenders query that joins
-- user_library_tracks against artist_offenses via LOWER(artist_name).

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ult_artist_name_lower
  ON user_library_tracks (LOWER(artist_name)) WHERE artist_name IS NOT NULL;
