-- Normalized playlist tables.
-- Playlists become first-class entities with their own identity, metadata,
-- and a junction table for track membership with position ordering.
-- user_library_tracks is NOT dropped — it continues to serve offense scanning
-- during the transition period.

-- ============================================================
-- Table: playlists
-- ============================================================
CREATE TABLE IF NOT EXISTS playlists (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id         UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider        VARCHAR(50)  NOT NULL,
    provider_playlist_id VARCHAR(500) NOT NULL,
    name            VARCHAR(500) NOT NULL,
    description     TEXT,
    image_url       TEXT,
    owner_name      VARCHAR(255),
    owner_id        VARCHAR(255),
    is_public       BOOLEAN,
    is_collaborative BOOLEAN NOT NULL DEFAULT FALSE,
    source_type     VARCHAR(50) NOT NULL DEFAULT 'playlist',
    provider_track_count INTEGER,
    snapshot_id     VARCHAR(255),
    last_synced     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_playlists_user_provider UNIQUE (user_id, provider, provider_playlist_id)
);

CREATE INDEX IF NOT EXISTS idx_playlists_user_id       ON playlists(user_id);
CREATE INDEX IF NOT EXISTS idx_playlists_user_provider  ON playlists(user_id, provider);
CREATE INDEX IF NOT EXISTS idx_playlists_source_type    ON playlists(user_id, source_type);

-- ============================================================
-- Table: playlist_tracks
-- ============================================================
CREATE TABLE IF NOT EXISTS playlist_tracks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    playlist_id     UUID NOT NULL REFERENCES playlists(id) ON DELETE CASCADE,
    provider_track_id VARCHAR(500) NOT NULL,
    track_name      VARCHAR(500),
    album_name      VARCHAR(500),
    artist_id       UUID REFERENCES artists(id),
    artist_name     VARCHAR(255),
    position        INTEGER NOT NULL DEFAULT 0,
    added_at        TIMESTAMPTZ,
    last_synced     TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT uq_playlist_tracks UNIQUE (playlist_id, provider_track_id, position)
);

CREATE INDEX IF NOT EXISTS idx_playlist_tracks_playlist ON playlist_tracks(playlist_id);
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_artist   ON playlist_tracks(artist_id);
CREATE INDEX IF NOT EXISTS idx_playlist_tracks_synced   ON playlist_tracks(playlist_id, last_synced);

-- ============================================================
-- Backfill: extract playlists from existing user_library_tracks
-- ============================================================

-- Step 1: Create playlist entities from distinct (user, provider, playlist) combos
INSERT INTO playlists (user_id, provider, provider_playlist_id, name, source_type, last_synced)
SELECT
    ult.user_id,
    ult.provider,
    provider_pid,
    playlist_display_name,
    derived_source_type,
    MAX(ult.last_synced)
FROM user_library_tracks ult,
LATERAL (
    SELECT
        CASE
            WHEN ult.source_type = 'playlist_track' AND ult.provider_track_id LIKE 'playlist:%'
                THEN split_part(ult.provider_track_id, ':', 2)
            WHEN ult.source_type = 'playlist' AND ult.provider_track_id LIKE 'playlist:%'
                THEN split_part(ult.provider_track_id, ':', 2)
            WHEN ult.source_type = 'library_playlist' AND ult.provider_track_id LIKE 'playlist:%'
                THEN split_part(ult.provider_track_id, ':', 2)
            WHEN ult.source_type = 'playlist_item' AND ult.provider_track_id LIKE 'playlist_item:%'
                THEN COALESCE(ult.playlist_name, '__yt_playlist__')
            WHEN ult.source_type IN ('liked', 'favorite_track', 'liked_video')
                THEN '__liked_songs__'
            WHEN ult.source_type IN ('saved_album', 'library_album', 'favorite_album')
                THEN '__saved_albums__'
            WHEN ult.source_type IN ('followed_artist', 'favorite_artist', 'subscription')
                THEN '__followed_artists__'
            WHEN ult.source_type = 'library_song'
                THEN '__library_songs__'
            ELSE COALESCE(ult.playlist_name, '__unknown__')
        END AS provider_pid,
        CASE
            WHEN ult.playlist_name IS NOT NULL AND ult.playlist_name <> ''
                THEN ult.playlist_name
            WHEN ult.source_type IN ('liked', 'favorite_track', 'liked_video')
                THEN 'Liked Songs'
            WHEN ult.source_type IN ('saved_album', 'library_album', 'favorite_album')
                THEN 'Saved Albums'
            WHEN ult.source_type IN ('followed_artist', 'favorite_artist', 'subscription')
                THEN 'Followed Artists'
            WHEN ult.source_type = 'library_song'
                THEN 'Library Songs'
            ELSE 'Unknown'
        END AS playlist_display_name,
        COALESCE(ult.source_type, 'playlist') AS derived_source_type
) AS derived
GROUP BY ult.user_id, ult.provider, provider_pid, playlist_display_name, derived_source_type
ON CONFLICT (user_id, provider, provider_playlist_id) DO NOTHING;

-- Step 2: Backfill playlist_tracks for actual track-level rows
INSERT INTO playlist_tracks (
    playlist_id, provider_track_id, track_name, album_name,
    artist_id, artist_name, position, added_at, last_synced
)
SELECT
    p.id,
    CASE
        WHEN ult.provider_track_id LIKE 'playlist:%'
            THEN split_part(ult.provider_track_id, ':', 3)
        WHEN ult.provider_track_id LIKE 'liked:%'
            THEN substring(ult.provider_track_id FROM 7)
        WHEN ult.provider_track_id LIKE 'album:%'
            THEN substring(ult.provider_track_id FROM 7)
        WHEN ult.provider_track_id LIKE 'playlist_item:%'
            THEN substring(ult.provider_track_id FROM 15)
        WHEN ult.provider_track_id LIKE 'artist:%'
            THEN substring(ult.provider_track_id FROM 8)
        ELSE ult.provider_track_id
    END,
    ult.track_name,
    ult.album_name,
    ult.artist_id,
    ult.artist_name,
    CASE
        WHEN ult.provider_track_id LIKE 'playlist:%'
             AND split_part(ult.provider_track_id, ':', 4) ~ '^\d+$'
            THEN split_part(ult.provider_track_id, ':', 4)::integer
        ELSE (ROW_NUMBER() OVER (
            PARTITION BY p.id ORDER BY ult.added_at NULLS LAST, ult.id
        ) - 1)::integer
    END,
    ult.added_at,
    ult.last_synced
FROM user_library_tracks ult
JOIN playlists p
    ON p.user_id = ult.user_id
   AND p.provider = ult.provider
   AND p.provider_playlist_id = (
       CASE
           WHEN ult.source_type = 'playlist_track' AND ult.provider_track_id LIKE 'playlist:%'
               THEN split_part(ult.provider_track_id, ':', 2)
           WHEN ult.source_type = 'playlist' AND ult.provider_track_id LIKE 'playlist:%'
               THEN split_part(ult.provider_track_id, ':', 2)
           WHEN ult.source_type = 'library_playlist' AND ult.provider_track_id LIKE 'playlist:%'
               THEN split_part(ult.provider_track_id, ':', 2)
           WHEN ult.source_type = 'playlist_item' AND ult.provider_track_id LIKE 'playlist_item:%'
               THEN COALESCE(ult.playlist_name, '__yt_playlist__')
           WHEN ult.source_type IN ('liked', 'favorite_track', 'liked_video')
               THEN '__liked_songs__'
           WHEN ult.source_type IN ('saved_album', 'library_album', 'favorite_album')
               THEN '__saved_albums__'
           WHEN ult.source_type IN ('followed_artist', 'favorite_artist', 'subscription')
               THEN '__followed_artists__'
           WHEN ult.source_type = 'library_song'
               THEN '__library_songs__'
           ELSE COALESCE(ult.playlist_name, '__unknown__')
       END
   )
-- Exclude meta-rows that represent the playlist/album/artist itself (not tracks)
WHERE ult.track_name NOT LIKE '[Album] %'
  AND ult.track_name NOT LIKE '[Artist] %'
  AND ult.track_name NOT LIKE '[Playlist] %'
  AND ult.track_name NOT LIKE '[Subscription] %'
  AND NOT (ult.source_type = 'playlist' AND ult.provider_track_id LIKE 'playlist:%')
  AND NOT (ult.source_type = 'library_playlist')
ON CONFLICT (playlist_id, provider_track_id, position) DO NOTHING;
