# Build stage
FROM rust:1.82-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy only Cargo files first for dependency caching
COPY backend/Cargo.toml ./Cargo.toml

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached)
RUN cargo build --release && rm -rf src

# Copy actual source code
COPY backend/src ./src
COPY backend/migrations ./migrations

# Build application (only rebuilds if source changed)
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/music-streaming-blocklist-backend /app/api

# Copy migrations
COPY backend/migrations ./migrations

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 3000 9090

CMD ["./api"]