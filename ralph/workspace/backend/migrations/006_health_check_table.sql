-- Health check table for database connectivity testing
CREATE TABLE IF NOT EXISTS health_check (
    id SERIAL PRIMARY KEY,
    status VARCHAR(50) NOT NULL DEFAULT 'ok',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert a default health check record
INSERT INTO health_check (status) VALUES ('ok') ON CONFLICT DO NOTHING;

-- Index for performance
CREATE INDEX IF NOT EXISTS idx_health_check_created_at ON health_check(created_at DESC);