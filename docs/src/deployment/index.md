# Deployment Guides

Learn how to deploy your riglr agents to production environments with proper security, monitoring, and scalability.

## Deployment Options

### [Docker](docker.md)
Containerize your agent for consistent deployment across environments. Docker provides isolation, reproducibility, and easy scaling.

### [Fly.io](fly-io.md)
Deploy globally distributed agents with automatic SSL, load balancing, and edge computing capabilities.

### [Kubernetes](kubernetes.md)
Enterprise-grade deployment with horizontal scaling, rolling updates, and comprehensive orchestration.

## Pre-Deployment Checklist

Before deploying to production, ensure:

### Security
- [ ] Private keys stored securely (KMS, Vault, etc.)
- [ ] Environment variables properly configured
- [ ] Network endpoints use TLS/SSL
- [ ] Rate limiting implemented
- [ ] Input validation on all user inputs

### Monitoring
- [ ] Logging configured (structured JSON logs)
- [ ] Metrics collection enabled
- [ ] Error tracking set up (Sentry, etc.)
- [ ] Alerting rules defined
- [ ] Health checks implemented

### Performance
- [ ] Connection pooling configured
- [ ] Caching strategy implemented
- [ ] Database indexes optimized
- [ ] Resource limits defined
- [ ] Auto-scaling policies set

### Testing
- [ ] Unit tests passing
- [ ] Integration tests passing
- [ ] Load testing completed
- [ ] Disaster recovery tested
- [ ] Rollback procedure documented

## Environment Configuration

### Required Environment Variables

```env
# Blockchain Configuration
SOLANA_RPC_URL=https://api.mainnet-beta.solana.com
RPC_URL_1=https://eth-mainnet.alchemyapi.io/v2/your-key
RPC_URL_137=https://polygon-mainnet.alchemyapi.io/v2/your-key

# AI Provider
ANTHROPIC_API_KEY=your-anthropic-key
# OR
OPENAI_API_KEY=your-openai-key

# Database
REDIS_URL=redis://redis:6379
NEO4J_URI=bolt://neo4j:7687
NEO4J_USER=neo4j
NEO4J_PASSWORD=secure-password

# Monitoring
LOG_LEVEL=info
SENTRY_DSN=https://your-sentry-dsn
METRICS_PORT=9090

# Security
USE_KMS=true
KMS_KEY_ID=your-kms-key
```

### Configuration Management

Use separate configuration files for each environment:

```
config/
├── development.env
├── staging.env
└── production.env
```

Load the appropriate configuration:

```rust
let config_file = match std::env::var("ENV") {
    Ok(env) => format!("config/{}.env", env),
    Err(_) => "config/development.env".to_string(),
};
dotenv::from_filename(config_file).ok();
```

## High Availability Setup

### Multi-Region Deployment

Deploy agents across multiple regions for redundancy:

```yaml
regions:
  - us-east-1: Primary
  - eu-west-1: Secondary
  - ap-southeast-1: Secondary
```

### Load Balancing

Use a load balancer to distribute traffic:

```nginx
upstream riglr_agents {
    least_conn;
    server agent1.internal:8080;
    server agent2.internal:8080;
    server agent3.internal:8080;
}
```

### Database Replication

Set up Redis replication for job queue reliability:

```yaml
redis-master:
  image: redis:7
  command: redis-server --appendonly yes

redis-replica:
  image: redis:7
  command: redis-server --replicaof redis-master 6379
```

## Monitoring & Observability

### Logging

Structured logging with correlation IDs:

```rust
use tracing::{info, instrument};

#[instrument(skip(signer))]
async fn process_transaction(tx_id: &str, signer: &Signer) {
    info!(
        transaction_id = %tx_id,
        event = "transaction.started",
        "Processing transaction"
    );
}
```

### Metrics

Export Prometheus metrics:

```rust
use prometheus::{register_counter, Counter};

lazy_static! {
    static ref TRANSACTIONS_TOTAL: Counter = register_counter!(
        "riglr_transactions_total",
        "Total number of transactions processed"
    ).unwrap();
}
```

### Distributed Tracing

Implement OpenTelemetry for request tracing:

```rust
use opentelemetry::trace::Tracer;

let tracer = opentelemetry_jaeger::new_pipeline()
    .with_service_name("riglr-agent")
    .install_simple()?;
```

## Security Best Practices

### Key Management

Never store private keys in:
- Environment variables (production)
- Configuration files
- Container images
- Git repositories

Instead use:
- AWS KMS
- HashiCorp Vault
- Azure Key Vault
- Hardware Security Modules (HSM)

### Network Security

- Use VPC/private networks for internal communication
- Implement API rate limiting
- Enable DDoS protection
- Use Web Application Firewall (WAF)
- Rotate API keys regularly

### Audit Logging

Log all critical operations:

```rust
audit_log!({
    "action": "swap_tokens",
    "user": user_id,
    "amount": amount,
    "timestamp": Utc::now(),
    "ip_address": request.ip(),
});
```

## Scaling Strategies

### Horizontal Scaling

Scale based on metrics:

```yaml
autoscaling:
  min_replicas: 2
  max_replicas: 10
  metrics:
    - type: cpu
      target: 70%
    - type: memory
      target: 80%
    - type: custom
      metric: job_queue_depth
      target: 100
```

### Vertical Scaling

Resource recommendations by workload:

| Workload | CPU | Memory | Storage |
|----------|-----|--------|---------|
| Light | 0.5 vCPU | 1 GB | 10 GB |
| Medium | 2 vCPU | 4 GB | 50 GB |
| Heavy | 8 vCPU | 16 GB | 200 GB |

## Disaster Recovery

### Backup Strategy

- **Database**: Daily snapshots with 30-day retention
- **Configuration**: Version controlled in Git
- **Logs**: Streamed to external storage
- **State**: Replicated across regions

### Recovery Procedures

1. **Service Failure**: Auto-restart with exponential backoff
2. **Region Failure**: Failover to secondary region
3. **Data Corruption**: Restore from latest snapshot
4. **Security Breach**: Rotate all keys, audit access logs

## Cost Optimization

### Resource Optimization

- Use spot instances for non-critical workloads
- Implement request caching
- Optimize container images (multi-stage builds)
- Use CDN for static assets
- Implement connection pooling

### Monitoring Costs

Track costs by component:

```sql
SELECT 
    service,
    SUM(cost) as total_cost,
    COUNT(*) as request_count
FROM usage_metrics
GROUP BY service
ORDER BY total_cost DESC;
```

## Next Steps

Choose your deployment platform:
- [Docker](docker.md) - Best for containerized deployments
- [Fly.io](fly-io.md) - Best for global distribution
- [Kubernetes](kubernetes.md) - Best for enterprise scale

Each guide includes platform-specific configurations, best practices, and example deployments.