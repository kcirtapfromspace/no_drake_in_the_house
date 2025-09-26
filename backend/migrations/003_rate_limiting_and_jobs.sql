-- Migration for rate limiting and job processing system
-- This migration adds tables for tracking rate limits and batch checkpoints

-- Provider rate limiting state
CREATE TABLE IF NOT EXISTS provider_rate_state (
    provider VARCHAR(50) PRIMARY KEY,
    requests_remaining INTEGER DEFAULT 0,
    window_reset_at TIMESTAMP WITH TIME ZONE,
    current_backoff_seconds INTEGER DEFAULT 0,
    consecutive_failures INTEGER DEFAULT 0,
    last_request_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Circuit breaker state for providers
CREATE TABLE IF NOT EXISTS circuit_breaker_state (
    provider VARCHAR(50) PRIMARY KEY,
    state VARCHAR(20) DEFAULT 'closed', -- 'closed', 'open', 'half_open'
    failure_count INTEGER DEFAULT 0,
    last_failure_at TIMESTAMP WITH TIME ZONE,
    next_attempt_at TIMESTAMP WITH TIME ZONE,
    success_count_in_half_open INTEGER DEFAULT 0,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Batch checkpoints for resumable processing
CREATE TABLE IF NOT EXISTS batch_checkpoints (
    batch_id UUID PRIMARY KEY,
    provider VARCHAR(50) NOT NULL,
    operation_type VARCHAR(100) NOT NULL,
    total_items INTEGER NOT NULL,
    processed_items INTEGER DEFAULT 0,
    failed_items INTEGER DEFAULT 0,
    current_position INTEGER DEFAULT 0,
    last_successful_item_id TEXT,
    checkpoint_data JSONB DEFAULT '{}',
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Rate limit configurations (optional, can be managed in code)
CREATE TABLE IF NOT EXISTS rate_limit_configs (
    provider VARCHAR(50) PRIMARY KEY,
    requests_per_window INTEGER NOT NULL,
    window_duration_seconds INTEGER NOT NULL,
    burst_allowance INTEGER DEFAULT 0,
    backoff_multiplier DECIMAL(3,2) DEFAULT 2.0,
    max_backoff_seconds INTEGER DEFAULT 300,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);

-- Batch configurations for optimal API usage
CREATE TABLE IF NOT EXISTS batch_configs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider VARCHAR(50) NOT NULL,
    operation_type VARCHAR(100) NOT NULL,
    max_batch_size INTEGER NOT NULL,
    optimal_batch_size INTEGER NOT NULL,
    min_delay_between_batches_ms BIGINT DEFAULT 100,
    supports_parallel_batches BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW(),
    UNIQUE(provider, operation_type)
);

-- Indexes for performance
CREATE INDEX IF NOT EXISTS idx_provider_rate_state_provider ON provider_rate_state(provider);
CREATE INDEX IF NOT EXISTS idx_circuit_breaker_state_provider ON circuit_breaker_state(provider);
CREATE INDEX IF NOT EXISTS idx_batch_checkpoints_batch_id ON batch_checkpoints(batch_id);
CREATE INDEX IF NOT EXISTS idx_batch_checkpoints_provider_operation ON batch_checkpoints(provider, operation_type);
CREATE INDEX IF NOT EXISTS idx_batch_configs_provider_operation ON batch_configs(provider, operation_type);

-- Insert default rate limit configurations
INSERT INTO rate_limit_configs (provider, requests_per_window, window_duration_seconds, burst_allowance, backoff_multiplier, max_backoff_seconds)
VALUES 
    ('spotify', 100, 60, 20, 1.5, 120),
    ('apple_music', 1000, 3600, 50, 2.0, 300),
    ('youtube_music', 100, 60, 10, 2.0, 180),
    ('tidal', 200, 60, 30, 1.8, 150)
ON CONFLICT (provider) DO NOTHING;

-- Insert default batch configurations
INSERT INTO batch_configs (provider, operation_type, max_batch_size, optimal_batch_size, min_delay_between_batches_ms, supports_parallel_batches)
VALUES 
    ('spotify', 'remove_tracks', 50, 25, 200, FALSE),
    ('spotify', 'unfollow_artists', 50, 20, 150, FALSE),
    ('spotify', 'playlist_operations', 100, 50, 300, FALSE),
    ('spotify', 'remove_albums', 50, 25, 200, FALSE),
    ('apple_music', 'remove_tracks', 25, 15, 400, FALSE),
    ('apple_music', 'playlist_operations', 50, 25, 500, FALSE),
    ('youtube_music', 'content_filtering', 1, 1, 1000, FALSE),
    ('tidal', 'remove_tracks', 30, 20, 300, FALSE)
ON CONFLICT (provider, operation_type) DO NOTHING;

-- Function to update the updated_at timestamp
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

-- Triggers to automatically update updated_at
CREATE TRIGGER update_provider_rate_state_updated_at 
    BEFORE UPDATE ON provider_rate_state 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_circuit_breaker_state_updated_at 
    BEFORE UPDATE ON circuit_breaker_state 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_batch_checkpoints_updated_at 
    BEFORE UPDATE ON batch_checkpoints 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_rate_limit_configs_updated_at 
    BEFORE UPDATE ON rate_limit_configs 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_batch_configs_updated_at 
    BEFORE UPDATE ON batch_configs 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add comments for documentation
COMMENT ON TABLE provider_rate_state IS 'Tracks current rate limiting state for each provider';
COMMENT ON TABLE circuit_breaker_state IS 'Circuit breaker state to prevent cascading failures';
COMMENT ON TABLE batch_checkpoints IS 'Checkpoints for resumable batch processing operations';
COMMENT ON TABLE rate_limit_configs IS 'Rate limiting configurations for each provider';
COMMENT ON TABLE batch_configs IS 'Optimal batching configurations for different operations';

COMMENT ON COLUMN provider_rate_state.requests_remaining IS 'Number of requests remaining in current window';
COMMENT ON COLUMN provider_rate_state.window_reset_at IS 'When the current rate limit window resets';
COMMENT ON COLUMN provider_rate_state.current_backoff_seconds IS 'Current exponential backoff delay';
COMMENT ON COLUMN provider_rate_state.consecutive_failures IS 'Number of consecutive failures for backoff calculation';

COMMENT ON COLUMN circuit_breaker_state.state IS 'Circuit breaker state: closed, open, or half_open';
COMMENT ON COLUMN circuit_breaker_state.failure_count IS 'Total number of failures recorded';
COMMENT ON COLUMN circuit_breaker_state.next_attempt_at IS 'When to next attempt a request in open state';
COMMENT ON COLUMN circuit_breaker_state.success_count_in_half_open IS 'Successes needed to close circuit from half-open';

COMMENT ON COLUMN batch_checkpoints.current_position IS 'Current position in the batch processing';
COMMENT ON COLUMN batch_checkpoints.last_successful_item_id IS 'ID of the last successfully processed item';
COMMENT ON COLUMN batch_checkpoints.checkpoint_data IS 'Additional data needed to resume processing';

COMMENT ON COLUMN rate_limit_configs.requests_per_window IS 'Maximum requests allowed per time window';
COMMENT ON COLUMN rate_limit_configs.window_duration_seconds IS 'Duration of the rate limiting window in seconds';
COMMENT ON COLUMN rate_limit_configs.burst_allowance IS 'Additional requests allowed for burst traffic';
COMMENT ON COLUMN rate_limit_configs.backoff_multiplier IS 'Multiplier for exponential backoff calculation';

COMMENT ON COLUMN batch_configs.max_batch_size IS 'Maximum number of items that can be processed in one batch';
COMMENT ON COLUMN batch_configs.optimal_batch_size IS 'Recommended batch size for best performance';
COMMENT ON COLUMN batch_configs.min_delay_between_batches_ms IS 'Minimum delay between batch executions in milliseconds';
COMMENT ON COLUMN batch_configs.supports_parallel_batches IS 'Whether this operation supports parallel batch execution';