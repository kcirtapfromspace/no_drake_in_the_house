#!/bin/bash

set -e

echo "🔄 Running database migrations..."

# Wait for database to be ready
echo "Waiting for database to be ready..."
until pg_isready -h postgres.music-blocklist-dev.svc.cluster.local -p 5432 -U postgres; do
    echo "Waiting for postgres..."
    sleep 2
done

echo "Database is ready!"

# Install sqlx-cli if not present
if ! command -v sqlx &> /dev/null; then
    echo "Installing sqlx-cli..."
    cargo install sqlx-cli --no-default-features --features postgres
fi

# Run migrations
echo "Running migrations..."
cd /app
sqlx migrate run --database-url "$DATABASE_URL"

echo "✅ Migrations completed successfully!"