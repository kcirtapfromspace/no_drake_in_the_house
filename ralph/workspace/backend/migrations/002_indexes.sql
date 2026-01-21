-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS pg_trgm;

-- Indexes for fast lookups
CREATE INDEX idx_artists_external_ids_spotify ON artists ((external_ids->>'spotify'));
CREATE INDEX idx_artists_external_ids_apple ON artists ((external_ids->>'apple'));
CREATE INDEX idx_artists_external_ids_musicbrainz ON artists ((external_ids->>'musicbrainz'));
CREATE INDEX idx_artists_canonical ON artists(canonical_artist_id) WHERE canonical_artist_id IS NOT NULL;
CREATE INDEX idx_artists_name_search ON artists USING GIN (canonical_name gin_trgm_ops);

CREATE INDEX idx_action_items_batch ON action_items(batch_id);
CREATE INDEX idx_action_items_provider_entity ON action_items(entity_type, entity_id);
CREATE INDEX idx_connections_user_provider ON connections(user_id, provider);
CREATE INDEX idx_connections_status ON connections(status);

CREATE INDEX idx_user_artist_blocks_user ON user_artist_blocks(user_id);
CREATE INDEX idx_user_artist_blocks_artist ON user_artist_blocks(artist_id);

CREATE INDEX idx_community_list_items_list ON community_list_items(list_id);
CREATE INDEX idx_user_list_subscriptions_user ON user_list_subscriptions(user_id);

CREATE INDEX idx_audit_log_actor ON audit_log(actor_user_id);
CREATE INDEX idx_audit_log_subject ON audit_log(subject_type, subject_id);
CREATE INDEX idx_audit_log_created ON audit_log(created_at);