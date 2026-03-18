-- Add JSONB column to persist full flagged artist details in library_scan_results
ALTER TABLE library_scan_results
  ADD COLUMN IF NOT EXISTS flagged_artists_json JSONB NOT NULL DEFAULT '[]'::jsonb;
