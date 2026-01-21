-- Migration for audit logging and rate limiting support
-- This migration adds the necessary tables and types for security middleware

-- Create audit event type enum
CREATE TYPE audit_event_type AS ENUM (
    'user_registration',
    'user_login',
    'user_login_failed',
    'user_logout',
    'password_change',
    'totp_enabled',
    'totp_disabled',
    'totp_verification_failed',
    'token_refresh',
    'token_revoked',
    'rate_limit_exceeded',
    'suspicious_activity',
    'security_violation',
    'data_access',
    'data_modification',
    'admin_action',
    'system_event'
);

-- Create audit severity enum
CREATE TYPE audit_severity AS ENUM (
    'info',
    'warning',
    'error',
    'critical'
);

-- Add new columns to existing audit_log table for enhanced security logging
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS event_type audit_event_type;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS severity audit_severity DEFAULT 'info';
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS session_id VARCHAR(255);
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS resource_type VARCHAR(100);
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS resource_id VARCHAR(255);
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS details JSONB DEFAULT '{}'::jsonb;
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW();
ALTER TABLE audit_log ADD COLUMN IF NOT EXISTS correlation_id VARCHAR(255);

-- First, make the existing NOT NULL columns nullable temporarily
ALTER TABLE audit_log ALTER COLUMN subject_type DROP NOT NULL;
ALTER TABLE audit_log ALTER COLUMN subject_id DROP NOT NULL;

-- Rename existing columns to match new schema
ALTER TABLE audit_log RENAME COLUMN actor_user_id TO user_id;
ALTER TABLE audit_log RENAME COLUMN subject_type TO old_subject_type;
ALTER TABLE audit_log RENAME COLUMN subject_id TO old_subject_id;
ALTER TABLE audit_log RENAME COLUMN created_at TO old_created_at;

-- Update timestamp column to use the new timestamp if available, otherwise use old created_at
UPDATE audit_log SET timestamp = old_created_at WHERE timestamp IS NULL;

-- Create indexes for audit log queries (only if they don't exist)
CREATE INDEX IF NOT EXISTS idx_audit_log_timestamp ON audit_log(timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_audit_log_event_type ON audit_log(event_type) WHERE event_type IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_severity ON audit_log(severity) WHERE severity IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_user_id_new ON audit_log(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_ip_address ON audit_log(ip_address) WHERE ip_address IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_correlation_id ON audit_log(correlation_id) WHERE correlation_id IS NOT NULL;

-- Composite index for common queries
CREATE INDEX IF NOT EXISTS idx_audit_log_user_timestamp ON audit_log(user_id, timestamp DESC) WHERE user_id IS NOT NULL;
CREATE INDEX IF NOT EXISTS idx_audit_log_ip_timestamp ON audit_log(ip_address, timestamp DESC) WHERE ip_address IS NOT NULL;

-- Create rate limiting table for persistent rate limit state
-- Note: This is a backup to Redis - Redis is primary, this is for persistence
CREATE TABLE rate_limit_state (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    identifier VARCHAR(255) NOT NULL, -- IP address or user ID
    endpoint_type VARCHAR(100) NOT NULL, -- 'auth', 'api', etc.
    window_start TIMESTAMP WITH TIME ZONE NOT NULL,
    request_count INTEGER NOT NULL DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    
    -- Unique constraint to prevent duplicates
    UNIQUE(identifier, endpoint_type, window_start)
);

-- Indexes for rate limiting queries
CREATE INDEX idx_rate_limit_identifier ON rate_limit_state(identifier);
CREATE INDEX idx_rate_limit_endpoint_type ON rate_limit_state(endpoint_type);
CREATE INDEX idx_rate_limit_window_start ON rate_limit_state(window_start);
CREATE INDEX idx_rate_limit_updated_at ON rate_limit_state(updated_at);

-- Composite index for rate limit lookups
CREATE INDEX idx_rate_limit_lookup ON rate_limit_state(identifier, endpoint_type, window_start);

-- Create security incidents table for tracking security events
CREATE TABLE security_incidents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    incident_type VARCHAR(100) NOT NULL,
    severity audit_severity NOT NULL,
    title VARCHAR(255) NOT NULL,
    description TEXT,
    ip_address INET,
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    status VARCHAR(50) DEFAULT 'open', -- 'open', 'investigating', 'resolved', 'false_positive'
    metadata JSONB DEFAULT '{}'::jsonb,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    resolved_at TIMESTAMP WITH TIME ZONE,
    resolved_by UUID REFERENCES users(id) ON DELETE SET NULL,
    
    CONSTRAINT security_incidents_status_check CHECK (
        status IN ('open', 'investigating', 'resolved', 'false_positive')
    )
);

-- Indexes for security incidents
CREATE INDEX idx_security_incidents_created_at ON security_incidents(created_at DESC);
CREATE INDEX idx_security_incidents_status ON security_incidents(status);
CREATE INDEX idx_security_incidents_severity ON security_incidents(severity);
CREATE INDEX idx_security_incidents_ip_address ON security_incidents(ip_address) WHERE ip_address IS NOT NULL;
CREATE INDEX idx_security_incidents_user_id ON security_incidents(user_id) WHERE user_id IS NOT NULL;
CREATE INDEX idx_security_incidents_incident_type ON security_incidents(incident_type);

-- Create function to automatically update updated_at timestamps
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Create triggers for updated_at columns
CREATE TRIGGER update_rate_limit_state_updated_at 
    BEFORE UPDATE ON rate_limit_state 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_security_incidents_updated_at 
    BEFORE UPDATE ON security_incidents 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Create view for recent security events (last 24 hours)
CREATE VIEW recent_security_events AS
SELECT 
    al.id,
    al.event_type,
    al.severity,
    al.user_id,
    u.email as user_email,
    al.ip_address,
    al.action,
    al.details,
    al.timestamp,
    al.correlation_id
FROM audit_log al
LEFT JOIN users u ON al.user_id = u.id
WHERE al.timestamp >= NOW() - INTERVAL '24 hours'
  AND al.severity IN ('warning', 'error', 'critical')
ORDER BY al.timestamp DESC;

-- Create view for rate limit violations
CREATE VIEW rate_limit_violations AS
SELECT 
    al.id,
    al.ip_address,
    al.details->>'endpoint' as endpoint,
    al.details->>'current_count' as current_count,
    al.details->>'limit' as rate_limit,
    al.timestamp,
    al.correlation_id
FROM audit_log al
WHERE al.event_type = 'rate_limit_exceeded'
  AND al.timestamp >= NOW() - INTERVAL '1 hour'
ORDER BY al.timestamp DESC;

-- Create function to clean up old audit logs (for GDPR compliance)
CREATE OR REPLACE FUNCTION cleanup_old_audit_logs(retention_days INTEGER DEFAULT 90)
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    DELETE FROM audit_log 
    WHERE timestamp < NOW() - (retention_days || ' days')::INTERVAL;
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    -- Log the cleanup operation
    INSERT INTO audit_log (event_type, severity, action, details)
    VALUES (
        'system_event',
        'info',
        'Audit log cleanup completed',
        jsonb_build_object(
            'deleted_count', deleted_count,
            'retention_days', retention_days,
            'cleanup_timestamp', NOW()
        )
    );
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Create function to clean up old rate limit state
CREATE OR REPLACE FUNCTION cleanup_old_rate_limit_state()
RETURNS INTEGER AS $$
DECLARE
    deleted_count INTEGER;
BEGIN
    -- Delete rate limit entries older than 1 hour
    DELETE FROM rate_limit_state 
    WHERE updated_at < NOW() - INTERVAL '1 hour';
    
    GET DIAGNOSTICS deleted_count = ROW_COUNT;
    
    RETURN deleted_count;
END;
$$ LANGUAGE plpgsql;

-- Insert initial audit log entry for migration
INSERT INTO audit_log (event_type, severity, action, details)
VALUES (
    'system_event',
    'info',
    'Audit logging system initialized',
    jsonb_build_object(
        'migration', '008_audit_logging_and_rate_limiting',
        'timestamp', NOW(),
        'tables_created', ARRAY['audit_log', 'rate_limit_state', 'security_incidents'],
        'views_created', ARRAY['recent_security_events', 'rate_limit_violations']
    )
);

-- Grant necessary permissions (adjust as needed for your user)
-- GRANT SELECT, INSERT ON audit_log TO your_app_user;
-- GRANT SELECT, INSERT, UPDATE, DELETE ON rate_limit_state TO your_app_user;
-- GRANT SELECT, INSERT, UPDATE ON security_incidents TO your_app_user;
-- GRANT SELECT ON recent_security_events TO your_app_user;
-- GRANT SELECT ON rate_limit_violations TO your_app_user;