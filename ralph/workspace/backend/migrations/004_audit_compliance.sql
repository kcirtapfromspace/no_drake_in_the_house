-- Create custom types for audit and compliance
CREATE TYPE access_status AS ENUM ('active', 'inactive', 'suspended', 'pending_review');
CREATE TYPE data_request_type AS ENUM ('export', 'deletion');
CREATE TYPE data_request_status AS ENUM ('pending', 'processing', 'completed', 'failed', 'expired');

-- Access review table for SOC2 compliance
CREATE TABLE access_reviews (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    reviewer_id UUID REFERENCES users(id),
    access_level VARCHAR(50) NOT NULL DEFAULT 'user',
    permissions TEXT[] DEFAULT '{}',
    last_login TIMESTAMP WITH TIME ZONE,
    status access_status DEFAULT 'active',
    review_date TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    next_review_date TIMESTAMP WITH TIME ZONE DEFAULT (NOW() + INTERVAL '90 days'),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Data export/deletion requests for GDPR/CCPA compliance
CREATE TABLE data_export_requests (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE,
    request_type data_request_type NOT NULL,
    status data_request_status DEFAULT 'pending',
    requested_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    completed_at TIMESTAMP WITH TIME ZONE,
    export_url TEXT,
    expires_at TIMESTAMP WITH TIME ZONE,
    verification_token TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Security monitoring events (in-memory, but can be persisted for long-term analysis)
CREATE TABLE security_events (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    event_type VARCHAR(50) NOT NULL,
    user_id UUID REFERENCES users(id),
    ip_address INET,
    user_agent TEXT,
    details JSONB DEFAULT '{}',
    severity VARCHAR(20) NOT NULL DEFAULT 'low',
    timestamp TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_audit_log_actor_user_id ON audit_log(actor_user_id);
CREATE INDEX idx_audit_log_action ON audit_log(action);
CREATE INDEX idx_audit_log_subject_type ON audit_log(subject_type);
CREATE INDEX idx_audit_log_created_at ON audit_log(created_at DESC);
CREATE INDEX idx_audit_log_composite ON audit_log(actor_user_id, action, created_at DESC);

CREATE INDEX idx_access_reviews_user_id ON access_reviews(user_id);
CREATE INDEX idx_access_reviews_status ON access_reviews(status);
CREATE INDEX idx_access_reviews_next_review ON access_reviews(next_review_date);

CREATE INDEX idx_data_export_requests_user_id ON data_export_requests(user_id);
CREATE INDEX idx_data_export_requests_status ON data_export_requests(status);
CREATE INDEX idx_data_export_requests_expires_at ON data_export_requests(expires_at);

CREATE INDEX idx_security_events_user_id ON security_events(user_id);
CREATE INDEX idx_security_events_event_type ON security_events(event_type);
CREATE INDEX idx_security_events_severity ON security_events(severity);
CREATE INDEX idx_security_events_timestamp ON security_events(timestamp DESC);

-- Add security headers configuration table
CREATE TABLE security_headers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    header_name VARCHAR(100) NOT NULL,
    header_value TEXT NOT NULL,
    enabled BOOLEAN DEFAULT true,
    description TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default security headers
INSERT INTO security_headers (header_name, header_value, description) VALUES
('X-Content-Type-Options', 'nosniff', 'Prevents MIME type sniffing'),
('X-Frame-Options', 'DENY', 'Prevents clickjacking attacks'),
('X-XSS-Protection', '1; mode=block', 'Enables XSS filtering'),
('Strict-Transport-Security', 'max-age=31536000; includeSubDomains', 'Enforces HTTPS'),
('Content-Security-Policy', 'default-src ''self''; script-src ''self'' ''unsafe-inline''; style-src ''self'' ''unsafe-inline''; img-src ''self'' data: https:; connect-src ''self'' https://api.spotify.com https://api.music.apple.com', 'Content Security Policy'),
('Referrer-Policy', 'strict-origin-when-cross-origin', 'Controls referrer information'),
('Permissions-Policy', 'geolocation=(), microphone=(), camera=()', 'Controls browser features');

-- Add vulnerability scan results table
CREATE TABLE vulnerability_scans (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    scan_type VARCHAR(50) NOT NULL, -- 'dependency', 'code', 'infrastructure'
    scan_tool VARCHAR(100) NOT NULL,
    scan_version VARCHAR(50),
    started_at TIMESTAMP WITH TIME ZONE NOT NULL,
    completed_at TIMESTAMP WITH TIME ZONE,
    status VARCHAR(20) DEFAULT 'running', -- 'running', 'completed', 'failed'
    total_vulnerabilities INTEGER DEFAULT 0,
    critical_count INTEGER DEFAULT 0,
    high_count INTEGER DEFAULT 0,
    medium_count INTEGER DEFAULT 0,
    low_count INTEGER DEFAULT 0,
    results JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

CREATE INDEX idx_vulnerability_scans_scan_type ON vulnerability_scans(scan_type);
CREATE INDEX idx_vulnerability_scans_status ON vulnerability_scans(status);
CREATE INDEX idx_vulnerability_scans_completed_at ON vulnerability_scans(completed_at DESC);

-- Add compliance checklist table
CREATE TABLE compliance_checks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    check_name VARCHAR(200) NOT NULL,
    check_category VARCHAR(100) NOT NULL, -- 'SOC2', 'GDPR', 'CCPA', 'Security'
    description TEXT NOT NULL,
    status VARCHAR(20) DEFAULT 'pending', -- 'pending', 'compliant', 'non_compliant', 'not_applicable'
    last_checked TIMESTAMP WITH TIME ZONE,
    next_check_due TIMESTAMP WITH TIME ZONE,
    evidence_url TEXT,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert default compliance checks
INSERT INTO compliance_checks (check_name, check_category, description, next_check_due) VALUES
('Audit Logging Enabled', 'SOC2', 'All user actions are logged for audit purposes', NOW() + INTERVAL '30 days'),
('Access Reviews Conducted', 'SOC2', 'Regular access reviews are conducted for all users', NOW() + INTERVAL '90 days'),
('Data Encryption at Rest', 'SOC2', 'All sensitive data is encrypted at rest', NOW() + INTERVAL '30 days'),
('Data Encryption in Transit', 'SOC2', 'All data transmission uses TLS encryption', NOW() + INTERVAL '30 days'),
('GDPR Data Export Available', 'GDPR', 'Users can export their personal data', NOW() + INTERVAL '30 days'),
('GDPR Data Deletion Available', 'GDPR', 'Users can request deletion of their personal data', NOW() + INTERVAL '30 days'),
('CCPA Data Export Available', 'CCPA', 'California residents can export their personal data', NOW() + INTERVAL '30 days'),
('CCPA Data Deletion Available', 'CCPA', 'California residents can request deletion of their personal data', NOW() + INTERVAL '30 days'),
('Security Headers Configured', 'Security', 'All required security headers are configured', NOW() + INTERVAL '7 days'),
('Vulnerability Scanning Active', 'Security', 'Regular vulnerability scans are performed', NOW() + INTERVAL '7 days');

CREATE INDEX idx_compliance_checks_category ON compliance_checks(check_category);
CREATE INDEX idx_compliance_checks_status ON compliance_checks(status);
CREATE INDEX idx_compliance_checks_next_check_due ON compliance_checks(next_check_due);