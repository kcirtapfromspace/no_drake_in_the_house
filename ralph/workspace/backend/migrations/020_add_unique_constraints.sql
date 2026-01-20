-- Add unique constraints for platform IDs on albums and tracks

-- Albums unique constraints
CREATE UNIQUE INDEX IF NOT EXISTS idx_albums_apple_id_unique ON albums(apple_music_id) WHERE apple_music_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_albums_spotify_id_unique ON albums(spotify_id) WHERE spotify_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_albums_deezer_id_unique ON albums(deezer_id) WHERE deezer_id IS NOT NULL;

-- Tracks unique constraints
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_apple_id_unique ON tracks(apple_music_id) WHERE apple_music_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_spotify_id_unique ON tracks(spotify_id) WHERE spotify_id IS NOT NULL;
CREATE UNIQUE INDEX IF NOT EXISTS idx_tracks_deezer_id_unique ON tracks(deezer_id) WHERE deezer_id IS NOT NULL;
