# Deploying to Kubernetes

Enterprise-grade deployment with Kubernetes orchestration.

## Overview

Kubernetes provides:
- Horizontal pod autoscaling
- Rolling deployments with zero downtime
- Service discovery and load balancing
- Secret and configuration management
- Self-healing capabilities

## Prerequisites

- Kubernetes cluster (1.25+)
- kubectl configured
- Helm 3 (optional but recommended)

## Quick Start

### 1. Create Namespace

```yaml
apiVersion: v1
kind: Namespace
metadata:
  name: riglr
```

### 2. Deploy Application

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: riglr-agent
  namespace: riglr
spec:
  replicas: 3
  selector:
    matchLabels:
      app: riglr-agent
  template:
    metadata:
      labels:
        app: riglr-agent
    spec:
      containers:
      - name: agent
        image: riglr-agent:latest
        ports:
        - containerPort: 8080
        env:
        - name: RUST_LOG
          value: "info"
        envFrom:
        - secretRef:
            name: riglr-secrets
        resources:
          requests:
            memory: "2Gi"
            cpu: "1"
          limits:
            memory: "4Gi"
            cpu: "2"
```

### 3. Create Service

```yaml
apiVersion: v1
kind: Service
metadata:
  name: riglr-agent
  namespace: riglr
spec:
  selector:
    app: riglr-agent
  ports:
  - protocol: TCP
    port: 80
    targetPort: 8080
  type: LoadBalancer
```

### 4. Configure Secrets

```bash
kubectl create secret generic riglr-secrets \
  --from-literal=ANTHROPIC_API_KEY=your-key \
  --from-literal=SOLANA_PRIVATE_KEY=your-key \
  -n riglr
```

## Helm Chart

Coming soon: Complete Helm chart with configurable values, dependencies, and production-ready defaults.

## See Also

- [Docker Deployment](docker.md)
- [Fly.io Deployment](fly-io.md)
- [Deployment Overview](index.md)