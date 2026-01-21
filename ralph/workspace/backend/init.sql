-- Database initialization script for development
-- This script runs when the PostgreSQL container starts for the first time

-- Enable required extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";
CREATE EXTENSION IF NOT EXISTS "pgcrypto";
CREATE EXTENSION IF NOT EXISTS "pg_trgm";

-- Create a health check table for monitoring
CREATE TABLE IF NOT EXISTS health_check (
    id SERIAL PRIMARY KEY,
    status TEXT DEFAULT 'ok',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Insert initial health check record
INSERT INTO health_check (status) VALUES ('initialized') ON CONFLICT DO NOTHING;

-- Create development database user if it doesn't exist
DO $$
BEGIN
    IF NOT EXISTS (SELECT FROM pg_catalog.pg_roles WHERE rolname = 'kiro_dev') THEN
        CREATE ROLE kiro_dev WITH LOGIN PASSWORD 'kiro_dev_password';
    END IF;
END
$$;

-- Grant necessary permissions
GRANT CONNECT ON DATABASE kiro_dev TO kiro_dev;
GRANT USAGE ON SCHEMA public TO kiro_dev;
GRANT CREATE ON SCHEMA public TO kiro_dev;

-- Set up proper timezone
SET timezone = 'UTC';