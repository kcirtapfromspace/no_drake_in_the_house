-- OAuth provider accounts for social authentication
CREATE TABLE oauth_accounts (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider VARCHAR(50) NOT NULL, -- 'google', 'apple', 'github'
    provider_user_id VARCHAR(255) NOT NULL,
    email VARCHAR(255),
    display_name VARCHAR(255),
    avatar_url TEXT,
    access_token_encrypted BYTEA, -- AES-GCM encrypted
    refresh_token_encrypted BYTEA, -- AES-GCM encrypted
    token_expires_at TIMESTAMPTZ,
    scopes TEXT[], -- OAuth scopes granted
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    last_used_at TIMESTAMPTZ,
    
    -- Ensure unique provider accounts
    CONSTRAINT oauth_accounts_provider_user_unique UNIQUE(provider, provider_user_id),
    
    -- Prevent duplicate provider accounts per user
    CONSTRAINT oauth_accounts_user_provider_unique UNIQUE(user_id, provider)
);

-- Account merge audit trail for compliance and debugging
CREATE TABLE account_merges (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    primary_user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    merged_user_id UUID NOT NULL, -- User ID that was merged (may no longer exist)
    merged_oauth_accounts JSONB NOT NULL, -- Snapshot of OAuth accounts that were merged
    merged_connections JSONB, -- Snapshot of service connections that were merged
    merged_data JSONB, -- Other user data that was merged (lists, blocks, etc.)
    merge_reason TEXT NOT NULL,
    merged_at TIMESTAMPTZ DEFAULT NOW(),
    merged_by UUID REFERENCES users(id), -- Admin user who performed merge, if applicable
    
    -- Audit fields
    ip_address INET,
    user_agent TEXT
);

-- Indexes for efficient lookups
CREATE INDEX idx_oauth_accounts_user_id ON oauth_accounts(user_id);
CREATE INDEX idx_oauth_accounts_provider ON oauth_accounts(provider);
CREATE INDEX idx_oauth_accounts_provider_user_id ON oauth_accounts(provider, provider_user_id);
CREATE INDEX idx_oauth_accounts_email ON oauth_accounts(email) WHERE email IS NOT NULL;
CREATE INDEX idx_oauth_accounts_updated_at ON oauth_accounts(updated_at);
CREATE INDEX idx_oauth_accounts_last_used_at ON oauth_accounts(last_used_at) WHERE last_used_at IS NOT NULL;

-- Account merges indexes
CREATE INDEX idx_account_merges_primary_user_id ON account_merges(primary_user_id);
CREATE INDEX idx_account_merges_merged_user_id ON account_merges(merged_user_id);
CREATE INDEX idx_account_merges_merged_at ON account_merges(merged_at);

-- Add trigger to update updated_at timestamp
CREATE OR REPLACE FUNCTION update_oauth_accounts_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER oauth_accounts_updated_at_trigger
    BEFORE UPDATE ON oauth_accounts
    FOR EACH ROW
    EXECUTE FUNCTION update_oauth_accounts_updated_at();

-- Add audit logging trigger for OAuth account changes
CREATE OR REPLACE FUNCTION audit_oauth_account_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_log (
            actor_user_id,
            action,
            subject_type,
            subject_id,
            after_state,
            created_at
        ) VALUES (
            NEW.user_id,
            'oauth_account_created',
            'oauth_account',
            NEW.id::text,
            jsonb_build_object(
                'provider', NEW.provider,
                'provider_user_id', NEW.provider_user_id,
                'email', NEW.email,
                'display_name', NEW.display_name
            ),
            NOW()
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_log (
            actor_user_id,
            action,
            subject_type,
            subject_id,
            before_state,
            after_state,
            created_at
        ) VALUES (
            NEW.user_id,
            'oauth_account_updated',
            'oauth_account',
            NEW.id::text,
            jsonb_build_object(
                'provider', OLD.provider,
                'provider_user_id', OLD.provider_user_id,
                'email', OLD.email,
                'display_name', OLD.display_name,
                'last_used_at', OLD.last_used_at
            ),
            jsonb_build_object(
                'provider', NEW.provider,
                'provider_user_id', NEW.provider_user_id,
                'email', NEW.email,
                'display_name', NEW.display_name,
                'last_used_at', NEW.last_used_at
            ),
            NOW()
        );
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_log (
            actor_user_id,
            action,
            subject_type,
            subject_id,
            before_state,
            created_at
        ) VALUES (
            OLD.user_id,
            'oauth_account_deleted',
            'oauth_account',
            OLD.id::text,
            jsonb_build_object(
                'provider', OLD.provider,
                'provider_user_id', OLD.provider_user_id,
                'email', OLD.email,
                'display_name', OLD.display_name
            ),
            NOW()
        );
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER oauth_account_audit_trigger
    AFTER INSERT OR UPDATE OR DELETE ON oauth_accounts
    FOR EACH ROW
    EXECUTE FUNCTION audit_oauth_account_changes();

-- Add audit logging for account merges
CREATE OR REPLACE FUNCTION audit_account_merge()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_log (
        actor_user_id,
        action,
        subject_type,
        subject_id,
        after_state,
        ip_address,
        user_agent,
        created_at
    ) VALUES (
        COALESCE(NEW.merged_by, NEW.primary_user_id),
        'account_merge',
        'user',
        NEW.primary_user_id::text,
        jsonb_build_object(
            'merged_user_id', NEW.merged_user_id,
            'merge_reason', NEW.merge_reason,
            'oauth_accounts_count', jsonb_array_length(NEW.merged_oauth_accounts),
            'connections_count', CASE 
                WHEN NEW.merged_connections IS NOT NULL 
                THEN jsonb_array_length(NEW.merged_connections) 
                ELSE 0 
            END
        ),
        NEW.ip_address,
        NEW.user_agent,
        NOW()
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER account_merge_audit_trigger
    AFTER INSERT ON account_merges
    FOR EACH ROW
    EXECUTE FUNCTION audit_account_merge();