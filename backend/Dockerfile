# syntax=docker/dockerfile:1.4
# Multi-stage build for production backend with cargo-chef optimization
FROM rust:1.82-slim as chef

# Install cargo-chef for optimal dependency caching
RUN cargo install cargo-chef --locked
WORKDIR /app

FROM chef as planner
# Copy Cargo files and analyze dependencies
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo chef prepare --recipe-path recipe.json

FROM chef as builder

# Install system dependencies with cache mount
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Build dependencies first (heavily cached layer)
COPY --from=planner /app/recipe.json recipe.json
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo chef cook --release --recipe-path recipe.json

# Copy source code and build application
COPY . .
# Set offline mode for SQLx to avoid database connection requirement during build
ENV SQLX_OFFLINE=true
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/app/target,sharing=locked \
    cargo build --release --bin music-streaming-blocklist-backend && \
    cp target/release/music-streaming-blocklist-backend /tmp/backend

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libpq5 \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -m -u 1001 appuser

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /tmp/backend /app/backend

# Copy migrations
COPY migrations /app/migrations

# Change ownership to app user
RUN chown -R appuser:appuser /app

# Switch to app user
USER appuser

# Expose port
EXPOSE 3000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:3000/health || exit 1

# Run the application
CMD ["./backend"]