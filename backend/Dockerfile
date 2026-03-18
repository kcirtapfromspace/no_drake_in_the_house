# syntax=docker/dockerfile:1.4
# Multi-stage build for production backend.
FROM rust:1.94-slim as builder

# Install system dependencies with cache mount
RUN --mount=type=cache,target=/var/cache/apt,sharing=locked \
    --mount=type=cache,target=/var/lib/apt,sharing=locked \
    apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libpq-dev \
    protobuf-compiler \
    g++ \
    cmake \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy source code and build the release binary in one pass. This avoids
# recompiling large workspace crates twice during first-time cross-builds.
COPY . .
ARG BACKEND_BIN=music-streaming-blocklist-backend
ARG BACKEND_CARGO_FEATURES=
ARG BACKEND_CARGO_NO_DEFAULT_FEATURES=false
ENV BACKEND_BIN="${BACKEND_BIN}" \
    BACKEND_CARGO_FEATURES="${BACKEND_CARGO_FEATURES}" \
    BACKEND_CARGO_NO_DEFAULT_FEATURES="${BACKEND_CARGO_NO_DEFAULT_FEATURES}"
ENV SQLX_OFFLINE=true
ENV SQLX_OFFLINE_DIR=/app/.sqlx
RUN sed -i 's#^target-dir = .*#target-dir = "/tmp/cargo-target"#' /app/.cargo/config.toml
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    --mount=type=cache,target=/tmp/cargo-target,sharing=locked \
    sh -lc 'set -eu; \
      build_args="--release --bin ${BACKEND_BIN}"; \
      if [ "${BACKEND_CARGO_NO_DEFAULT_FEATURES}" = "true" ]; then \
        build_args="${build_args} --no-default-features"; \
      fi; \
      if [ -n "${BACKEND_CARGO_FEATURES}" ]; then \
        build_args="${build_args} --features ${BACKEND_CARGO_FEATURES}"; \
      fi; \
      cargo build ${build_args}; \
      cp "/tmp/cargo-target/release/${BACKEND_BIN}" /tmp/backend'

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
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
