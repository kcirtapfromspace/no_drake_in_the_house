# Simple API Dockerfile for testing
FROM rust:1.82-slim as builder

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy simple API source files
COPY docker/simple-api/Cargo.toml ./Cargo.toml
COPY docker/simple-api/main.rs ./src/main.rs

# Build the simple API
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary
COPY --from=builder /app/target/release/simple-music-api /app/api

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 3000

CMD ["./api"]