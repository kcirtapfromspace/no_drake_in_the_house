#!/bin/bash

# Script to prepare SQLx offline data for compilation without database

echo "Preparing SQLx offline data..."

# Set environment variables
export DATABASE_URL="sqlite::memory:"
export SQLX_OFFLINE=true

# Create .sqlx directory if it doesn't exist
mkdir -p .sqlx

# Generate a minimal query metadata file
cat > .sqlx/query-metadata.json << 'EOF'
{
  "db": "SQLite",
  "queries": {}
}
EOF

echo "SQLx offline data prepared. You can now compile without a database connection."
echo "Note: Some queries may need to be updated to work with SQLite syntax."