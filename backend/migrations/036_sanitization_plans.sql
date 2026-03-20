-- Sanitization plans for playlist grade/replace/publish workflow
CREATE TABLE IF NOT EXISTS sanitization_plans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL DEFAULT 'spotify',
    source_playlist_id VARCHAR(255) NOT NULL,
    source_playlist_name VARCHAR(500) NOT NULL,
    target_playlist_name VARCHAR(500),
    grade_data JSONB NOT NULL,
    replacements_data JSONB,
    selected_replacements JSONB,
    publish_result JSONB,
    status VARCHAR(50) NOT NULL DEFAULT 'draft',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    published_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_sanitization_plans_user ON sanitization_plans(user_id);
CREATE INDEX IF NOT EXISTS idx_sanitization_plans_status ON sanitization_plans(user_id, status);
