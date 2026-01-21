-- Migration runner script
-- This can be executed manually if needed

-- Ensure we're in the correct database
\c kiro_dev;

-- Show current migration status
SELECT 
    version,
    description,
    installed_on,
    success
FROM _sqlx_migrations 
ORDER BY version;

-- Note: Actual migrations are run by SQLx migrate command
-- This script is for manual inspection and troubleshooting