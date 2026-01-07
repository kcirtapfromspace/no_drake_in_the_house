-- Category Subscriptions
-- Users can subscribe to curated blocklists by offense category

CREATE TABLE category_subscriptions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    category offense_category NOT NULL,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT unique_user_category UNIQUE (user_id, category)
);

-- Index for fast lookups
CREATE INDEX idx_category_subs_user ON category_subscriptions(user_id);
CREATE INDEX idx_category_subs_category ON category_subscriptions(category);

-- View to get category stats (artist counts per category)
CREATE OR REPLACE VIEW category_stats AS
SELECT
    category,
    COUNT(DISTINCT artist_id) as artist_count,
    COUNT(*) as offense_count
FROM artist_offenses
WHERE status IN ('pending', 'verified')
GROUP BY category;
