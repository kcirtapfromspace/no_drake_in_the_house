# syntax=docker/dockerfile:1.4
# Multi-stage build for production frontend with optimized caching
FROM node:18-alpine as builder

WORKDIR /app

# Install system dependencies
RUN apk add --no-cache curl

# Copy package files first for optimal caching
COPY package*.json ./

# Install dependencies with cache mount
RUN --mount=type=cache,target=/root/.npm,sharing=locked \
    npm ci

# Copy source code
COPY . .

# Build the application
RUN npm run build

# Runtime stage with nginx
FROM nginx:alpine

# Install curl for health checks
RUN apk add --no-cache curl

# Copy custom nginx configuration
COPY nginx-dev.conf /etc/nginx/conf.d/default.conf

# Copy built application from builder stage
COPY --from=builder /app/public /usr/share/nginx/html

# Create nginx user and set permissions
RUN chown -R nginx:nginx /usr/share/nginx/html && \
    chown -R nginx:nginx /var/cache/nginx && \
    chown -R nginx:nginx /var/log/nginx && \
    chown -R nginx:nginx /etc/nginx/conf.d

# Switch to nginx user
USER nginx

# Expose port
EXPOSE 5000

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:5000/health || exit 1

# Start nginx
CMD ["nginx", "-g", "daemon off;"]