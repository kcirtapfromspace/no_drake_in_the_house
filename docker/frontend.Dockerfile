# Build stage
FROM node:18-alpine as builder

WORKDIR /app

# Copy package files
COPY frontend/package*.json ./

# Install dependencies
RUN npm ci

# Copy source code
COPY frontend/ ./

# Build application
RUN npm run build

# Runtime stage
FROM nginx:alpine

# Copy built application
COPY --from=builder /app/public /usr/share/nginx/html

# Copy nginx configuration
COPY docker/nginx.conf /etc/nginx/nginx.conf

EXPOSE 80

CMD ["nginx", "-g", "daemon off;"]