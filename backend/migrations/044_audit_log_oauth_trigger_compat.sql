-- Ensure OAuth audit triggers write correctly across both legacy and
-- migrated audit_log schemas.

CREATE OR REPLACE FUNCTION insert_audit_log_compat(
    p_actor_user_id UUID,
    p_action TEXT,
    p_subject_type TEXT,
    p_subject_id TEXT,
    p_before_state JSONB DEFAULT NULL,
    p_after_state JSONB DEFAULT NULL,
    p_ip_address INET DEFAULT NULL,
    p_user_agent TEXT DEFAULT NULL
)
RETURNS VOID AS $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM information_schema.columns
        WHERE table_schema = 'public'
          AND table_name = 'audit_log'
          AND column_name = 'actor_user_id'
    ) THEN
        EXECUTE '
            INSERT INTO audit_log (
                actor_user_id, action, subject_type, subject_id,
                before_state, after_state, ip_address, user_agent, created_at
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, NOW())
        '
        USING
            p_actor_user_id,
            p_action,
            p_subject_type,
            p_subject_id,
            p_before_state,
            p_after_state,
            p_ip_address,
            p_user_agent;
    ELSE
        EXECUTE '
            INSERT INTO audit_log (
                user_id, action, old_subject_type, old_subject_id,
                before_state, after_state, ip_address, user_agent, details, timestamp
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, NOW())
        '
        USING
            p_actor_user_id,
            p_action,
            p_subject_type,
            p_subject_id,
            p_before_state,
            p_after_state,
            p_ip_address,
            p_user_agent,
            jsonb_strip_nulls(
                jsonb_build_object(
                    'subject_type', p_subject_type,
                    'subject_id', p_subject_id
                )
            );
    END IF;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_oauth_account_changes()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        PERFORM insert_audit_log_compat(
            NEW.user_id,
            'oauth_account_created',
            'oauth_account',
            NEW.id::text,
            NULL,
            jsonb_build_object(
                'provider', NEW.provider,
                'provider_user_id', NEW.provider_user_id,
                'email', NEW.email,
                'display_name', NEW.display_name
            ),
            NULL,
            NULL
        );
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        PERFORM insert_audit_log_compat(
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
            NULL,
            NULL
        );
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        PERFORM insert_audit_log_compat(
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
            NULL,
            NULL,
            NULL
        );
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

CREATE OR REPLACE FUNCTION audit_account_merge()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM insert_audit_log_compat(
        COALESCE(NEW.merged_by, NEW.primary_user_id),
        'account_merge',
        'user',
        NEW.primary_user_id::text,
        NULL,
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
        NEW.user_agent
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
