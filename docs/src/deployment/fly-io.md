# Deploying to Fly.io

Deploy your riglr agents globally with Fly.io's edge computing platform.

## Overview

Fly.io provides:
- Global deployment in 30+ regions
- Automatic SSL certificates
- Built-in load balancing
- Persistent storage volumes
- Private networking between services

## Prerequisites

Install the Fly CLI:
```bash
curl -L https://fly.io/install.sh | sh
fly auth login
```

## Quick Start

### 1. Initialize Fly App

```bash
fly launch --name riglr-agent
```

### 2. Configure fly.toml

```toml
app = "riglr-agent"
primary_region = "iad"

[build]
  dockerfile = "Dockerfile"

[env]
  RUST_LOG = "info"
  
[experimental]
  auto_rollback = true

[[services]]
  http_checks = []
  internal_port = 8080
  protocol = "tcp"
  script_checks = []
  
  [services.concurrency]
    hard_limit = 25
    soft_limit = 20
    type = "connections"
    
  [[services.ports]]
    force_https = true
    handlers = ["http"]
    port = 80
    
  [[services.ports]]
    handlers = ["tls", "http"]
    port = 443
    
  [[services.tcp_checks]]
    grace_period = "1s"
    interval = "15s"
    restart_limit = 0
    timeout = "2s"
```

### 3. Set Secrets

```bash
fly secrets set ANTHROPIC_API_KEY=your-key
fly secrets set SOLANA_PRIVATE_KEY=your-key
```

### 4. Deploy

```bash
fly deploy
```

## Advanced Configuration

Coming soon: Full deployment guide with multi-region setup, scaling, and monitoring.

## See Also

- [Docker Deployment](docker.md)
- [Kubernetes Deployment](kubernetes.md)
- [Deployment Overview](index.md)