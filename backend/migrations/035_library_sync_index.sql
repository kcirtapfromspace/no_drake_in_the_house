-- Index for incremental sync stale-deletion query:
-- DELETE FROM user_library_tracks WHERE user_id = $1 AND provider = $2 AND last_synced < $3
CREATE INDEX IF NOT EXISTS idx_library_user_provider_synced
  ON user_library_tracks (user_id, provider, last_synced);
