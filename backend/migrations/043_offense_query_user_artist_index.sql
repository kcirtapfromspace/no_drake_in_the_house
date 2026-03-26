-- no-transaction
-- Index to speed up offender lookups that can fall back to artist_id joins.

CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_ult_user_artist
  ON user_library_tracks (user_id, artist_id) WHERE artist_id IS NOT NULL;
