# Migration runner Dockerfile
FROM rust:1.82-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Install sqlx-cli
RUN cargo install sqlx-cli --no-default-features --features postgres

# Copy migrations
COPY backend/migrations ./migrations

# Create migration script inline
RUN echo '#!/bin/bash\n\
set -e\n\
echo "🔄 Running database migrations..."\n\
echo "Waiting for database to be ready..."\n\
until pg_isready -h music-blocklist-manager-postgresql -p 5432 -U postgres; do\n\
    echo "Waiting for postgres..."\n\
    sleep 2\n\
done\n\
echo "Database is ready!"\n\
echo "Running migrations..."\n\
cd /app\n\
sqlx migrate run --database-url "$DATABASE_URL"\n\
echo "✅ Migrations completed successfully!"' > ./run-migrations.sh && \
chmod +x ./run-migrations.sh

CMD ["./run-migrations.sh"]