-- User management
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email VARCHAR(255) UNIQUE NOT NULL,
    password_hash VARCHAR(255),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    settings JSONB DEFAULT '{}'
);

-- Service connections with health tracking
CREATE TABLE connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    provider_user_id VARCHAR(255),
    scopes TEXT[],
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    token_version INTEGER DEFAULT 1,
    expires_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'active',
    last_health_check TIMESTAMP WITH TIME ZONE,
    error_code TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(user_id, provider)
);

-- Rate limiting state per provider
CREATE TABLE provider_rate_state (
    provider VARCHAR(50) PRIMARY KEY,
    remaining INTEGER DEFAULT 0,
    reset_at TIMESTAMP WITH TIME ZONE,
    window_size INTEGER DEFAULT 3600,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Artist catalog with disambiguation
CREATE TABLE artists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    canonical_name VARCHAR(255) NOT NULL,
    canonical_artist_id UUID REFERENCES artists(id), -- self-ref for aliases
    external_ids JSONB DEFAULT '{}', -- {spotify: "id", apple: "id", musicbrainz: "id", isni: "id"}
    metadata JSONB DEFAULT '{}', -- {image: "url", genres: ["string"], isrc: ["string"], upc: ["string"]}
    aliases JSONB DEFAULT '{}', -- {name: "string", source: "string", confidence: float}
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- User DNP lists
CREATE TABLE user_artist_blocks (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    tags TEXT[],
    note TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, artist_id)
);

-- Community lists with governance
CREATE TABLE community_lists (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    owner_user_id UUID REFERENCES users(id),
    name VARCHAR(255) NOT NULL,
    description TEXT,
    criteria TEXT NOT NULL, -- Required neutral criteria
    governance_url TEXT, -- Link to governance process
    update_cadence TEXT, -- "weekly", "monthly", "as-needed"
    version INTEGER DEFAULT 1,
    visibility VARCHAR(20) DEFAULT 'public',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE TABLE community_list_items (
    list_id UUID REFERENCES community_lists(id) ON DELETE CASCADE,
    artist_id UUID REFERENCES artists(id) ON DELETE CASCADE,
    rationale_link TEXT,
    added_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (list_id, artist_id)
);

-- List subscriptions
CREATE TABLE user_list_subscriptions (
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    list_id UUID REFERENCES community_lists(id) ON DELETE CASCADE,
    version_pinned INTEGER,
    auto_update BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    PRIMARY KEY (user_id, list_id)
);

-- Action tracking with idempotency
CREATE TABLE action_batches (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL,
    idempotency_key TEXT UNIQUE,
    dry_run BOOLEAN DEFAULT false,
    status VARCHAR(20) DEFAULT 'pending',
    options JSONB DEFAULT '{}', -- {block_collabs: true, block_featuring: true, aggressiveness: "moderate"}
    summary JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE
);

CREATE TABLE action_items (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    batch_id UUID REFERENCES action_batches(id) ON DELETE CASCADE,
    entity_type VARCHAR(50) NOT NULL,
    entity_id VARCHAR(255) NOT NULL,
    action VARCHAR(50) NOT NULL,
    idempotency_key TEXT,
    before_state JSONB,
    after_state JSONB,
    status VARCHAR(20) DEFAULT 'pending',
    error_message TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(batch_id, entity_type, entity_id, action, idempotency_key)
);

-- Audit log for SOC2 compliance
CREATE TABLE audit_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_user_id UUID REFERENCES users(id),
    action VARCHAR(100) NOT NULL,
    subject_type VARCHAR(50) NOT NULL,
    subject_id VARCHAR(255) NOT NULL,
    before_state JSONB,
    after_state JSONB,
    ip_address INET,
    user_agent TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);