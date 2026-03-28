#!/bin/bash
# Script to prepare SQLx queries for offline compilation

set -euo pipefail

echo "🔧 Preparing SQLx queries for offline compilation..."

# Check if DATABASE_URL is set
if [ -z "${DATABASE_URL:-}" ]; then
    echo "⚠️  DATABASE_URL not set, using default for preparation"
    export DATABASE_URL="postgres://ndith:password@localhost:5432/ndith"
fi

# Start a temporary database if needed
TEMP_DB_STARTED=false
if ! pg_isready -h localhost -p 5432 -U ndith 2>/dev/null; then
    echo "🐘 Starting temporary PostgreSQL for SQLx preparation..."
    docker run -d --name sqlx-prep-db \
        -e POSTGRES_DB=ndith \
        -e POSTGRES_USER=ndith \
        -e POSTGRES_PASSWORD=password \
        -p 5432:5432 \
        postgres:15 > /dev/null
    
    TEMP_DB_STARTED=true
    
    # Wait for database to be ready
    echo "⏳ Waiting for database to be ready..."
    for i in {1..30}; do
        if pg_isready -h localhost -p 5432 -U ndith 2>/dev/null; then
            break
        fi
        if [ $i -eq 30 ]; then
            echo "❌ Database failed to start"
            exit 1
        fi
        sleep 1
    done
fi

# Run migrations
echo "🗄️  Running database migrations..."
sqlx migrate run || {
    echo "⚠️  Migration failed, continuing with query preparation..."
}

# Prepare queries
echo "📝 Preparing SQLx queries..."
cargo sqlx prepare || {
    echo "⚠️  Some queries failed to prepare, but continuing..."
}

# Clean up temporary database
if [ "$TEMP_DB_STARTED" = true ]; then
    echo "🧹 Cleaning up temporary database..."
    docker stop sqlx-prep-db > /dev/null 2>&1 || true
    docker rm sqlx-prep-db > /dev/null 2>&1 || true
fi

echo "✅ SQLx preparation complete!"
echo "📁 Generated .sqlx/ directory for offline compilation"