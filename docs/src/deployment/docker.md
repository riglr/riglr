# Deploying with Docker

Docker is the recommended way to package and deploy your `riglr`-based agent for production. It provides a consistent, isolated environment for your application.

## Creating a Dockerfile

Here is a multi-stage `Dockerfile` optimized for building and running a riglr agent:

```dockerfile
# Stage 1: Builder
# Use the official Rust image as a build environment
FROM rust:1.75 as builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Create app directory
WORKDIR /usr/src/riglr-agent

# Copy manifests
COPY Cargo.toml Cargo.lock ./
COPY riglr-*/Cargo.toml riglr-*/

# Build dependencies (this is cached if manifests don't change)
RUN mkdir -p riglr-core/src && echo "fn main() {}" > riglr-core/src/lib.rs
RUN cargo build --release --workspace
RUN rm -rf riglr-*/src

# Copy source code
COPY . .

# Build the release binary
RUN cargo build --release --workspace

# Stage 2: Runner
# Use a minimal base image for smaller and more secure final image
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    libssl-dev \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user for security
RUN useradd -m -u 1001 -s /bin/bash riglr

# Copy the compiled binary from builder
COPY --from=builder /usr/src/riglr-agent/target/release/riglr-showcase /usr/local/bin/riglr-agent

# Copy configuration files
COPY chains.toml /etc/riglr/

# Set ownership
RUN chown -R riglr:riglr /etc/riglr

# Switch to non-root user
USER riglr

# Set the entrypoint
ENTRYPOINT ["riglr-agent"]

# Default command (can be overridden)
CMD ["--interactive"]
```

## Building the Docker Image

Build your Docker image with proper tagging:

```bash
# Development build
docker build -t riglr-agent:dev .

# Production build with version tag
docker build -t riglr-agent:v1.0.0 -t riglr-agent:latest .

# Multi-platform build (for ARM and x86)
docker buildx build --platform linux/amd64,linux/arm64 \
  -t riglr-agent:latest .
```

## Running the Container

### Basic Run

```bash
docker run --rm -it \
  --env-file .env \
  riglr-agent:latest
```

### With Volume Mounts

```bash
docker run --rm -it \
  --env-file .env \
  -v $(pwd)/data:/data \
  -v $(pwd)/logs:/logs \
  riglr-agent:latest
```

### With Network Configuration

```bash
docker run --rm -it \
  --env-file .env \
  --network host \  # Use host network for better performance
  riglr-agent:latest
```

## Docker Compose Setup

For managing multiple services, use `docker-compose.yml`:

```yaml
version: '3.8'

services:
  agent:
    build: .
    image: riglr-agent:latest
    container_name: riglr-agent
    restart: unless-stopped
    env_file:
      - .env
    volumes:
      - ./data:/data
      - ./logs:/logs
    depends_on:
      - redis
      - neo4j
    networks:
      - riglr-network
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
        reservations:
          cpus: '1'
          memory: 2G

  redis:
    image: redis:7-alpine
    container_name: riglr-redis
    restart: unless-stopped
    command: redis-server --appendonly yes
    volumes:
      - redis-data:/data
    networks:
      - riglr-network
    ports:
      - "127.0.0.1:6379:6379"

  neo4j:
    image: neo4j:5
    container_name: riglr-neo4j
    restart: unless-stopped
    environment:
      - NEO4J_AUTH=neo4j/your-secure-password
      - NEO4J_dbms_memory_pagecache_size=1G
      - NEO4J_dbms_memory_heap_max__size=2G
    volumes:
      - neo4j-data:/data
      - neo4j-logs:/logs
    networks:
      - riglr-network
    ports:
      - "127.0.0.1:7474:7474"  # HTTP
      - "127.0.0.1:7687:7687"  # Bolt

networks:
  riglr-network:
    driver: bridge

volumes:
  redis-data:
  neo4j-data:
  neo4j-logs:
```

Start all services:

```bash
docker-compose up -d

# View logs
docker-compose logs -f agent

# Stop all services
docker-compose down
```

## Environment Configuration

Create environment-specific files:

### `.env.development`
```env
# Development configuration
RUST_LOG=debug
SOLANA_RPC_URL=https://api.devnet.solana.com
ANTHROPIC_API_KEY=your-dev-key
REDIS_URL=redis://redis:6379
```

### `.env.production`
```env
# Production configuration
RUST_LOG=info
SOLANA_RPC_URL=https://your-production-rpc.com
ANTHROPIC_API_KEY=${ANTHROPIC_API_KEY}  # From secrets manager
REDIS_URL=redis://redis:6379
USE_KMS=true
KMS_KEY_ID=${KMS_KEY_ID}
```

## Security Best Practices

### 1. Use Non-Root User

Always run containers as non-root:

```dockerfile
RUN useradd -m -u 1001 riglr
USER riglr
```

### 2. Scan for Vulnerabilities

```bash
# Scan image for vulnerabilities
docker scan riglr-agent:latest

# Use Trivy for comprehensive scanning
trivy image riglr-agent:latest
```

### 3. Sign Images

```bash
# Sign with Docker Content Trust
export DOCKER_CONTENT_TRUST=1
docker push riglr-agent:latest
```

### 4. Use Secrets Management

Never embed secrets in images:

```yaml
# docker-compose with secrets
services:
  agent:
    image: riglr-agent:latest
    secrets:
      - anthropic_key
      - solana_key

secrets:
  anthropic_key:
    external: true
  solana_key:
    external: true
```

## Optimization Tips

### 1. Multi-Stage Builds

Reduce image size by excluding build dependencies:

```dockerfile
# Final image only contains runtime dependencies
FROM debian:bookworm-slim
COPY --from=builder /app/target/release/agent /usr/local/bin/
```

### 2. Layer Caching

Order Dockerfile commands for optimal caching:

```dockerfile
# Dependencies change less frequently
COPY Cargo.toml Cargo.lock ./
RUN cargo build --release

# Source code changes more frequently
COPY src ./src
RUN cargo build --release
```

### 3. Use Alpine for Smaller Images

```dockerfile
FROM alpine:3.18
RUN apk add --no-cache libgcc
```

## Health Checks

Add health checks to your container:

```dockerfile
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1
```

In docker-compose:

```yaml
services:
  agent:
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 3s
      retries: 3
      start_period: 40s
```

## Monitoring

### Container Metrics

Monitor with Prometheus:

```yaml
services:
  prometheus:
    image: prom/prometheus
    volumes:
      - ./prometheus.yml:/etc/prometheus/prometheus.yml
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
    ports:
      - "9090:9090"
```

### Logging

Configure structured logging:

```yaml
services:
  agent:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
        labels: "service=riglr-agent"
```

Or use external logging:

```yaml
logging:
  driver: "fluentd"
  options:
    fluentd-address: "localhost:24224"
    tag: "riglr.{{.Name}}"
```

## Deployment Strategies

### Rolling Updates

```bash
# Update with zero downtime
docker-compose up -d --no-deps --build agent
```

### Blue-Green Deployment

```bash
# Deploy green version
docker-compose -f docker-compose.green.yml up -d

# Switch traffic to green
docker-compose -f docker-compose.blue.yml down

# Rename green to blue
mv docker-compose.green.yml docker-compose.blue.yml
```

## Troubleshooting

### Debug Running Container

```bash
# Execute shell in running container
docker exec -it riglr-agent /bin/bash

# View real-time logs
docker logs -f riglr-agent

# Inspect container
docker inspect riglr-agent
```

### Common Issues

**Container exits immediately:**
```bash
# Check exit code and logs
docker ps -a
docker logs riglr-agent
```

**Permission denied errors:**
```bash
# Ensure proper file ownership
docker exec riglr-agent ls -la /data
```

**Out of memory:**
```bash
# Increase memory limits
docker run -m 4g riglr-agent:latest
```

## Production Checklist

- [ ] Use specific version tags (not `:latest`)
- [ ] Implement health checks
- [ ] Configure resource limits
- [ ] Set up logging aggregation
- [ ] Enable metrics collection
- [ ] Use secrets management
- [ ] Scan for vulnerabilities
- [ ] Document rollback procedure
- [ ] Test disaster recovery

## Next Steps

- Review [Deployment Overview](index.md) for more options
- Deploy to orchestration platform: Kubernetes *(Coming Soon)*
- Deploy globally: Fly.io *(Coming Soon)*