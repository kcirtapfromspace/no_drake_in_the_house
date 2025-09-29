# Multi-stage build for Rust Worker
FROM rust:1.75-alpine AS builder

# Install build dependencies
RUN apk add --no-cache musl-dev openssl-dev openssl-libs-static pkgconfig

# Create app directory
WORKDIR /app

# Copy dependency files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release --bin kiro-worker
RUN rm -rf src

# Copy source code
COPY backend/src ./src

# Build the worker application
RUN touch src/main.rs && cargo build --release --bin kiro-worker

# Runtime stage
FROM alpine:3.18

# Install runtime dependencies
RUN apk add --no-cache ca-certificates tzdata

# Create non-root user
RUN addgroup -g 1000 kiro && \
    adduser -D -s /bin/sh -u 1000 -G kiro kiro

# Create app directory
WORKDIR /app

# Copy binary from builder stage
COPY --from=builder /app/target/release/kiro-worker ./kiro-worker

# Create cache directory
RUN mkdir -p /app/cache && chown -R kiro:kiro /app

# Switch to non-root user
USER kiro

# Expose metrics port
EXPOSE 9090

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:9090/health || exit 1

# Run the worker
CMD ["./kiro-worker"]