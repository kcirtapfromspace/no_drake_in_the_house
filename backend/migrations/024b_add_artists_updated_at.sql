-- Add updated_at column to artists table (required by seed migrations)
ALTER TABLE artists ADD COLUMN IF NOT EXISTS updated_at TIMESTAMP WITH TIME ZONE DEFAULT NOW();

-- Add trigger to auto-update the updated_at column
CREATE OR REPLACE FUNCTION update_artists_updated_at()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS artists_updated_at_trigger ON artists;
CREATE TRIGGER artists_updated_at_trigger
    BEFORE UPDATE ON artists
    FOR EACH ROW
    EXECUTE FUNCTION update_artists_updated_at();

-- Backfill existing rows
UPDATE artists SET updated_at = created_at WHERE updated_at IS NULL;
