-- Live news tracking and offense detection pipeline
-- Supports RSS feeds, NewsAPI, Twitter, Reddit, and web scraping

-- News source registry
CREATE TABLE news_sources (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(50) NOT NULL, -- 'rss', 'newsapi', 'twitter', 'reddit', 'scraper'
    url TEXT NOT NULL,
    config JSONB NOT NULL DEFAULT '{}', -- {poll_interval, selectors, auth_config, filters}
    credibility_score INTEGER CHECK (credibility_score >= 1 AND credibility_score <= 5),
    category VARCHAR(100), -- 'music_news', 'entertainment', 'celebrity_gossip', 'legal'
    language VARCHAR(10) DEFAULT 'en',
    is_active BOOLEAN DEFAULT true,
    last_polled_at TIMESTAMP WITH TIME ZONE,
    last_successful_poll_at TIMESTAMP WITH TIME ZONE,
    poll_interval_minutes INTEGER DEFAULT 60,
    consecutive_failures INTEGER DEFAULT 0,
    error_message TEXT,
    articles_fetched_total INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Raw news articles
CREATE TABLE news_articles (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID REFERENCES news_sources(id) ON DELETE SET NULL,
    external_id VARCHAR(255), -- Original article ID from source
    url TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT,
    excerpt TEXT,
    author VARCHAR(255),
    published_at TIMESTAMP WITH TIME ZONE,
    fetched_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    language VARCHAR(10) DEFAULT 'en',
    image_url TEXT,
    raw_data JSONB DEFAULT '{}', -- Original response from source
    word_count INTEGER,
    reading_time_minutes INTEGER,
    processing_status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'processing', 'completed', 'failed', 'skipped'
    processed_at TIMESTAMP WITH TIME ZONE,
    embedding_generated BOOLEAN DEFAULT false,
    embedding_model VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_article_url UNIQUE (url)
);

-- Extracted entities from news articles
CREATE TABLE news_article_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES news_articles(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE SET NULL,
    entity_name VARCHAR(255) NOT NULL,
    entity_type VARCHAR(50) NOT NULL, -- 'artist', 'person', 'organization', 'venue', 'label'
    confidence FLOAT NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    context_snippet TEXT, -- Surrounding text for context
    position_start INTEGER,
    position_end INTEGER,
    mention_count INTEGER DEFAULT 1,
    sentiment_score FLOAT CHECK (sentiment_score >= -1.0 AND sentiment_score <= 1.0),
    extraction_model VARCHAR(100),
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Offense classifications from news
CREATE TABLE news_offense_classifications (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    article_id UUID NOT NULL REFERENCES news_articles(id) ON DELETE CASCADE,
    entity_id UUID REFERENCES news_article_entities(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE SET NULL,
    offense_category offense_category NOT NULL,
    severity offense_severity NOT NULL DEFAULT 'moderate',
    confidence FLOAT NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    evidence_snippet TEXT,
    keywords_matched TEXT[],
    classification_model VARCHAR(100),
    human_verified BOOLEAN DEFAULT FALSE,
    verified_by UUID REFERENCES users(id),
    verified_at TIMESTAMP WITH TIME ZONE,
    verification_notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Social media posts (Twitter, Reddit, etc.)
CREATE TABLE social_media_posts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    platform VARCHAR(50) NOT NULL, -- 'twitter', 'reddit', 'instagram', 'tiktok'
    external_id VARCHAR(255) NOT NULL,
    url TEXT,
    author_handle VARCHAR(255),
    author_display_name VARCHAR(255),
    author_verified BOOLEAN DEFAULT false,
    author_followers INTEGER,
    content TEXT NOT NULL,
    content_type VARCHAR(50) DEFAULT 'text', -- 'text', 'image', 'video', 'link'
    engagement_metrics JSONB DEFAULT '{}', -- {likes, retweets, comments, shares, views}
    hashtags TEXT[],
    mentioned_handles TEXT[],
    mentioned_entities TEXT[],
    sentiment_score FLOAT CHECK (sentiment_score >= -1.0 AND sentiment_score <= 1.0),
    is_reply BOOLEAN DEFAULT false,
    is_repost BOOLEAN DEFAULT false,
    parent_post_id VARCHAR(255),
    fetched_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
    posted_at TIMESTAMP WITH TIME ZONE,
    processing_status VARCHAR(20) DEFAULT 'pending',
    CONSTRAINT unique_social_post UNIQUE (platform, external_id)
);

-- Link social posts to extracted entities
CREATE TABLE social_post_entities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    post_id UUID NOT NULL REFERENCES social_media_posts(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE SET NULL,
    entity_name VARCHAR(255) NOT NULL,
    entity_type VARCHAR(50) NOT NULL,
    confidence FLOAT NOT NULL CHECK (confidence >= 0.0 AND confidence <= 1.0),
    context_snippet TEXT,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- News source fetch history for monitoring
CREATE TABLE news_fetch_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    source_id UUID NOT NULL REFERENCES news_sources(id) ON DELETE CASCADE,
    fetch_started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    fetch_completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) NOT NULL, -- 'success', 'partial', 'failed', 'rate_limited'
    articles_found INTEGER DEFAULT 0,
    articles_new INTEGER DEFAULT 0,
    articles_updated INTEGER DEFAULT 0,
    error_message TEXT,
    response_time_ms INTEGER,
    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_news_sources_active ON news_sources(is_active, source_type);
CREATE INDEX idx_news_sources_poll ON news_sources(is_active, last_polled_at) WHERE is_active = true;
CREATE INDEX idx_news_articles_status ON news_articles(processing_status);
CREATE INDEX idx_news_articles_published ON news_articles(published_at DESC);
CREATE INDEX idx_news_articles_source ON news_articles(source_id, published_at DESC);
CREATE INDEX idx_news_articles_embedding ON news_articles(embedding_generated) WHERE NOT embedding_generated;
CREATE INDEX idx_news_entities_artist ON news_article_entities(artist_id) WHERE artist_id IS NOT NULL;
CREATE INDEX idx_news_entities_article ON news_article_entities(article_id);
CREATE INDEX idx_offense_class_artist ON news_offense_classifications(artist_id) WHERE artist_id IS NOT NULL;
CREATE INDEX idx_offense_class_verified ON news_offense_classifications(human_verified, confidence DESC);
CREATE INDEX idx_social_posts_platform ON social_media_posts(platform, posted_at DESC);
CREATE INDEX idx_social_posts_status ON social_media_posts(processing_status);
CREATE INDEX idx_social_entities_artist ON social_post_entities(artist_id) WHERE artist_id IS NOT NULL;
CREATE INDEX idx_fetch_log_source ON news_fetch_log(source_id, created_at DESC);

-- Full text search on articles
CREATE INDEX idx_news_articles_fts ON news_articles USING gin(to_tsvector('english', title || ' ' || COALESCE(content, '')));

-- Seed default RSS news sources
INSERT INTO news_sources (name, source_type, url, config, credibility_score, category, poll_interval_minutes)
VALUES
    ('Pitchfork News', 'rss', 'https://pitchfork.com/rss/news/', '{}', 4, 'music_news', 30),
    ('Rolling Stone Music', 'rss', 'https://www.rollingstone.com/music/music-news/feed/', '{}', 4, 'music_news', 30),
    ('Billboard', 'rss', 'https://www.billboard.com/feed/', '{}', 4, 'music_news', 30),
    ('The Guardian Music', 'rss', 'https://www.theguardian.com/music/rss', '{}', 5, 'music_news', 60),
    ('TMZ', 'rss', 'https://www.tmz.com/rss.xml', '{}', 2, 'celebrity_gossip', 15),
    ('NME News', 'rss', 'https://www.nme.com/news/music/feed', '{}', 3, 'music_news', 30),
    ('Consequence of Sound', 'rss', 'https://consequenceofsound.net/feed/', '{}', 3, 'music_news', 30),
    ('Variety Music', 'rss', 'https://variety.com/v/music/feed/', '{}', 4, 'entertainment', 60),
    ('Complex Music', 'rss', 'https://www.complex.com/music/rss', '{}', 3, 'music_news', 30),
    ('Stereogum', 'rss', 'https://www.stereogum.com/feed/', '{}', 3, 'music_news', 60)
ON CONFLICT DO NOTHING;
