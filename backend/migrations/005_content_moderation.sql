-- Content moderation and appeals system

-- Create custom types for moderation
DO $$ BEGIN
    CREATE TYPE moderation_status AS ENUM ('pending', 'under_review', 'approved', 'rejected', 'requires_changes');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE moderation_priority AS ENUM ('low', 'normal', 'high', 'urgent');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

DO $$ BEGIN
    CREATE TYPE appeal_status AS ENUM ('pending', 'under_review', 'upheld', 'denied', 'escalated');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Moderation queue for community list content
CREATE TABLE IF NOT EXISTS moderation_queue (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    list_id UUID REFERENCES community_lists(id) ON DELETE CASCADE,
    submitter_id UUID REFERENCES users(id) ON DELETE CASCADE,
    content_type VARCHAR(50) NOT NULL, -- 'list_creation', 'list_update', 'artist_addition'
    content_data JSONB NOT NULL,
    status moderation_status DEFAULT 'pending',
    priority moderation_priority DEFAULT 'normal',
    assigned_moderator_id UUID REFERENCES users(id),
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reviewed_at TIMESTAMP WITH TIME ZONE,
    review_notes TEXT,
    auto_moderation_result JSONB,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Appeals system for moderation decisions
CREATE TABLE IF NOT EXISTS appeals (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    moderation_entry_id UUID REFERENCES moderation_queue(id) ON DELETE CASCADE,
    appellant_id UUID REFERENCES users(id) ON DELETE CASCADE,
    appeal_reason TEXT NOT NULL,
    additional_evidence JSONB,
    status appeal_status DEFAULT 'pending',
    assigned_reviewer_id UUID REFERENCES users(id),
    submitted_at TIMESTAMP WITH TIME ZONE NOT NULL,
    reviewed_at TIMESTAMP WITH TIME ZONE,
    resolution TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Content policy violations tracking
CREATE TABLE IF NOT EXISTS content_violations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    moderation_entry_id UUID REFERENCES moderation_queue(id) ON DELETE CASCADE,
    violation_type VARCHAR(50) NOT NULL,
    description TEXT NOT NULL,
    severity VARCHAR(20) NOT NULL,
    matched_text TEXT,
    suggested_replacement TEXT,
    auto_detected BOOLEAN DEFAULT true,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Moderator performance tracking
CREATE TABLE IF NOT EXISTS moderator_stats (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    moderator_id UUID REFERENCES users(id) ON DELETE CASCADE,
    period_start TIMESTAMP WITH TIME ZONE NOT NULL,
    period_end TIMESTAMP WITH TIME ZONE NOT NULL,
    total_reviews INTEGER DEFAULT 0,
    approved_count INTEGER DEFAULT 0,
    rejected_count INTEGER DEFAULT 0,
    average_review_time_minutes FLOAT DEFAULT 0,
    appeals_overturned INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(moderator_id, period_start, period_end)
);

-- Content policy rules (configurable)
CREATE TABLE IF NOT EXISTS content_policy_rules (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    rule_name VARCHAR(100) NOT NULL UNIQUE,
    rule_type VARCHAR(50) NOT NULL, -- 'regex', 'keyword', 'ml_model'
    rule_pattern TEXT NOT NULL,
    violation_type VARCHAR(50) NOT NULL,
    severity VARCHAR(20) NOT NULL,
    enabled BOOLEAN DEFAULT true,
    auto_reject BOOLEAN DEFAULT false,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default content policy rules
INSERT INTO content_policy_rules (rule_name, rule_type, rule_pattern, violation_type, severity, auto_reject, description)
SELECT * FROM (VALUES
    ('personal_attacks', 'regex', '\b(evil|bad|terrible|awful|horrible)\s+(person|artist|individual)\b', 'personal_attack', 'high', true, 'Detects personal attacks and character judgments'),
    ('legal_claims', 'regex', '\b(guilty|convicted|criminal|illegal|lawsuit|sued)\b', 'unsubstantiated_claim', 'critical', true, 'Detects unsubstantiated legal claims'),
    ('offensive_language', 'regex', '\b(hate|despise|loathe)\b', 'offensive_language', 'high', false, 'Detects offensive language'),
    ('non_neutral_language', 'regex', '\b(obviously|clearly|definitely)\s+(bad|wrong|evil)\b', 'non_neutral_language', 'medium', false, 'Detects non-neutral language'),
    ('subjective_opinions', 'regex', '\b(everyone knows|it''s obvious)\b', 'non_neutral_language', 'medium', false, 'Detects subjective opinion statements')
) AS v(rule_name, rule_type, rule_pattern, violation_type, severity, auto_reject, description)
WHERE NOT EXISTS (SELECT 1 FROM content_policy_rules);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_moderation_queue_status ON moderation_queue(status);
CREATE INDEX IF NOT EXISTS idx_moderation_queue_priority ON moderation_queue(priority);
CREATE INDEX IF NOT EXISTS idx_moderation_queue_submitted_at ON moderation_queue(submitted_at);
CREATE INDEX IF NOT EXISTS idx_moderation_queue_assigned_moderator ON moderation_queue(assigned_moderator_id);
CREATE INDEX IF NOT EXISTS idx_moderation_queue_list_id ON moderation_queue(list_id);

CREATE INDEX IF NOT EXISTS idx_appeals_status ON appeals(status);
CREATE INDEX IF NOT EXISTS idx_appeals_submitted_at ON appeals(submitted_at);
CREATE INDEX IF NOT EXISTS idx_appeals_moderation_entry ON appeals(moderation_entry_id);
CREATE INDEX IF NOT EXISTS idx_appeals_appellant ON appeals(appellant_id);

CREATE INDEX IF NOT EXISTS idx_content_violations_moderation_entry ON content_violations(moderation_entry_id);
CREATE INDEX IF NOT EXISTS idx_content_violations_type ON content_violations(violation_type);
CREATE INDEX IF NOT EXISTS idx_content_violations_severity ON content_violations(severity);

CREATE INDEX IF NOT EXISTS idx_moderator_stats_moderator_id ON moderator_stats(moderator_id);
CREATE INDEX IF NOT EXISTS idx_moderator_stats_period ON moderator_stats(period_start, period_end);

CREATE INDEX IF NOT EXISTS idx_content_policy_rules_enabled ON content_policy_rules(enabled);
CREATE INDEX IF NOT EXISTS idx_content_policy_rules_type ON content_policy_rules(rule_type);

-- Add moderation-related columns to community_lists if not exists
DO $$ BEGIN
    ALTER TABLE community_lists ADD COLUMN moderation_status moderation_status DEFAULT 'approved';
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

DO $$ BEGIN
    ALTER TABLE community_lists ADD COLUMN moderated_at TIMESTAMP WITH TIME ZONE;
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

DO $$ BEGIN
    ALTER TABLE community_lists ADD COLUMN moderator_id UUID REFERENCES users(id);
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

-- Add user roles for moderation
DO $$ BEGIN
    ALTER TABLE users ADD COLUMN role VARCHAR(20) DEFAULT 'user';
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

DO $$ BEGIN
    ALTER TABLE users ADD COLUMN moderator_permissions TEXT[] DEFAULT '{}';
EXCEPTION
    WHEN duplicate_column THEN null;
END $$;

-- Create view for moderation dashboard
CREATE OR REPLACE VIEW moderation_dashboard AS
SELECT 
    mq.id,
    mq.list_id,
    cl.name as list_name,
    u.email as submitter_email,
    mq.content_type,
    mq.status,
    mq.priority,
    mq.submitted_at,
    mq.reviewed_at,
    mod_user.email as moderator_email,
    EXTRACT(EPOCH FROM (COALESCE(mq.reviewed_at, NOW()) - mq.submitted_at)) / 3600.0 as hours_pending,
    (SELECT COUNT(*) FROM content_violations cv WHERE cv.moderation_entry_id = mq.id) as violation_count,
    (SELECT COUNT(*) FROM appeals a WHERE a.moderation_entry_id = mq.id) as appeal_count
FROM moderation_queue mq
JOIN community_lists cl ON mq.list_id = cl.id
JOIN users u ON mq.submitter_id = u.id
LEFT JOIN users mod_user ON mq.assigned_moderator_id = mod_user.id;

-- Create view for appeal dashboard
CREATE OR REPLACE VIEW appeal_dashboard AS
SELECT 
    a.id,
    a.moderation_entry_id,
    mq.content_type,
    cl.name as list_name,
    u.email as appellant_email,
    a.appeal_reason,
    a.status,
    a.submitted_at,
    a.reviewed_at,
    reviewer.email as reviewer_email,
    EXTRACT(EPOCH FROM (COALESCE(a.reviewed_at, NOW()) - a.submitted_at)) / 3600.0 as hours_pending
FROM appeals a
JOIN moderation_queue mq ON a.moderation_entry_id = mq.id
JOIN community_lists cl ON mq.list_id = cl.id
JOIN users u ON a.appellant_id = u.id
LEFT JOIN users reviewer ON a.assigned_reviewer_id = reviewer.id;