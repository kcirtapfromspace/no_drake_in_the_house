-- Token Vault Persistence Migration
-- Adds data_key_id for encrypted token decryption lookup and updated_at for tracking

-- Add data_key_id column to track which encryption key was used
ALTER TABLE connections
    ADD COLUMN IF NOT EXISTS data_key_id TEXT;

-- Add updated_at column for tracking token updates
ALTER TABLE connections
    ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Create index for data_key_id lookups (for key rotation)
CREATE INDEX IF NOT EXISTS idx_connections_data_key_id ON connections(data_key_id)
    WHERE data_key_id IS NOT NULL;

-- Add trigger to automatically update updated_at timestamp
CREATE OR REPLACE FUNCTION update_connections_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS connections_updated_at_trigger ON connections;
CREATE TRIGGER connections_updated_at_trigger
    BEFORE UPDATE ON connections
    FOR EACH ROW
    EXECUTE FUNCTION update_connections_updated_at();

-- Add comment for documentation
COMMENT ON COLUMN connections.data_key_id IS 'Key ID used for envelope encryption of tokens (e.g., user-{uuid}-spotify)';
COMMENT ON COLUMN connections.updated_at IS 'Timestamp of last connection update';
