# Deployment Guide

This guide covers deploying the BitSacco WhatsApp Bot to various platforms and environments.

## Prerequisites

- Docker and Docker Compose
- Environment variables configured
- WhatsApp Cloud API access
- BitSacco.com API access
- Domain name (for production)

## Environment Configuration

### Required Environment Variables

Create a `.env` file with the following variables:

```env
# WhatsApp Cloud API Configuration
WHATSAPP_ACCESS_TOKEN=your_whatsapp_access_token_here
WHATSAPP_PHONE_NUMBER_ID=your_phone_number_id_here
WHATSAPP_WEBHOOK_VERIFY_TOKEN=your_webhook_verify_token_here

# BitSacco API Configuration
BITSACCO_API_BASE_URL=https://api.bitsacco.com
BITSACCO_API_TOKEN=your_bitsacco_api_token_here

# Server Configuration
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info

# Security Configuration
RATE_LIMIT_REQUESTS_PER_MINUTE=60
MAX_MESSAGE_LENGTH=4096

# BTC Service Configuration
BTC_API_BASE_URL=https://api.coinbase.com/v2/prices/BTC-USD/spot
BTC_API_KEY=your_btc_api_key_here
```

### Environment-Specific Configurations

#### Development
```env
RUST_LOG=debug
SERVER_PORT=8080
```

#### Staging
```env
RUST_LOG=info
SERVER_PORT=8080
BITSACCO_API_BASE_URL=https://staging-api.bitsacco.com
```

#### Production
```env
RUST_LOG=warn
SERVER_PORT=8080
BITSACCO_API_BASE_URL=https://api.bitsacco.com
```

## Local Development

### Using Docker Compose

```bash
# Clone the repository
git clone https://github.com/bitsacco/whatsapp-bot.git
cd whatsapp-bot

# Copy environment file
cp env.example .env
# Edit .env with your configuration

# Build and run
docker-compose up --build

# Run in background
docker-compose up -d --build
```

### Using Cargo

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Build the project
cargo build --release

# Run the application
cargo run
```

## Cloud Deployment

### Railway

Railway provides easy deployment with automatic scaling and monitoring.

#### Setup

1. **Install Railway CLI**
   ```bash
   npm install -g @railway/cli
   ```

2. **Login to Railway**
   ```bash
   railway login
   ```

3. **Initialize Project**
   ```bash
   railway init
   ```

4. **Configure Environment Variables**
   ```bash
   railway variables set WHATSAPP_ACCESS_TOKEN=your_token
   railway variables set WHATSAPP_PHONE_NUMBER_ID=your_phone_id
   railway variables set WHATSAPP_WEBHOOK_VERIFY_TOKEN=your_verify_token
   railway variables set BITSACCO_API_TOKEN=your_bitsacco_token
   ```

5. **Deploy**
   ```bash
   railway up
   ```

#### Railway Configuration

Create `railway.toml`:

```toml
[build]
builder = "dockerfile"

[deploy]
startCommand = "./bitsacco-whatsapp-bot"
healthcheckPath = "/health"
healthcheckTimeout = 100
restartPolicyType = "always"

[environments.production]
variables = { RUST_LOG = "warn" }

[environments.staging]
variables = { RUST_LOG = "info" }
```

### Fly.io

Fly.io provides global deployment with edge computing capabilities.

#### Setup

1. **Install Fly CLI**
   ```bash
   curl -L https://fly.io/install.sh | sh
   ```

2. **Login to Fly**
   ```bash
   fly auth login
   ```

3. **Create Fly App**
   ```bash
   fly launch
   ```

4. **Configure Secrets**
   ```bash
   fly secrets set WHATSAPP_ACCESS_TOKEN=your_token
   fly secrets set WHATSAPP_PHONE_NUMBER_ID=your_phone_id
   fly secrets set WHATSAPP_WEBHOOK_VERIFY_TOKEN=your_verify_token
   fly secrets set BITSACCO_API_TOKEN=your_bitsacco_token
   ```

5. **Deploy**
   ```bash
   fly deploy
   ```

#### Fly Configuration

Create `fly.toml`:

```toml
app = "bitsacco-whatsapp-bot"
primary_region = "iad"

[build]

[env]
  RUST_LOG = "info"

[http_service]
  internal_port = 8080
  force_https = true
  auto_stop_machines = true
  auto_start_machines = true
  min_machines_running = 1
  processes = ["app"]

[[vm]]
  cpu_kind = "shared"
  cpus = 1
  memory_mb = 256

[metrics]
  port = 9091
  path = "/metrics"
```

### Render

Render provides simple deployment with automatic SSL and scaling.

#### Setup

1. **Connect Repository**
   - Go to [render.com](https://render.com)
   - Connect your GitHub repository
   - Select "Web Service"

2. **Configure Build**
   - Build Command: `docker build -t bitsacco-whatsapp-bot .`
   - Start Command: `./bitsacco-whatsapp-bot`

3. **Set Environment Variables**
   - Add all required environment variables in the dashboard

4. **Deploy**
   - Click "Create Web Service"

### AWS ECS

For enterprise deployments on AWS.

#### Setup

1. **Create ECR Repository**
   ```bash
   aws ecr create-repository --repository-name bitsacco-whatsapp-bot
   ```

2. **Build and Push Image**
   ```bash
   # Get login token
   aws ecr get-login-password --region us-east-1 | docker login --username AWS --password-stdin 123456789012.dkr.ecr.us-east-1.amazonaws.com

   # Build and tag
   docker build -t bitsacco-whatsapp-bot .
   docker tag bitsacco-whatsapp-bot:latest 123456789012.dkr.ecr.us-east-1.amazonaws.com/bitsacco-whatsapp-bot:latest

   # Push
   docker push 123456789012.dkr.ecr.us-east-1.amazonaws.com/bitsacco-whatsapp-bot:latest
   ```

3. **Create ECS Task Definition**
   ```json
   {
     "family": "bitsacco-whatsapp-bot",
     "networkMode": "awsvpc",
     "requiresCompatibilities": ["FARGATE"],
     "cpu": "256",
     "memory": "512",
     "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
     "containerDefinitions": [
       {
         "name": "bitsacco-whatsapp-bot",
         "image": "123456789012.dkr.ecr.us-east-1.amazonaws.com/bitsacco-whatsapp-bot:latest",
         "portMappings": [
           {
             "containerPort": 8080,
             "protocol": "tcp"
           }
         ],
         "environment": [
           {
             "name": "RUST_LOG",
             "value": "info"
           }
         ],
         "secrets": [
           {
             "name": "WHATSAPP_ACCESS_TOKEN",
             "valueFrom": "arn:aws:secretsmanager:us-east-1:123456789012:secret:bitsacco/whatsapp-token"
           }
         ],
         "logConfiguration": {
           "logDriver": "awslogs",
           "options": {
             "awslogs-group": "/ecs/bitsacco-whatsapp-bot",
             "awslogs-region": "us-east-1",
             "awslogs-stream-prefix": "ecs"
           }
         }
       }
     ]
   }
   ```

## CI/CD Pipeline

### GitHub Actions

The project includes a comprehensive CI/CD pipeline:

```yaml
name: CI/CD Pipeline

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run tests
        run: cargo test

  build:
    needs: test
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build Docker image
        run: docker build -t bitsacco-whatsapp-bot .

  deploy:
    needs: build
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - name: Deploy to production
        run: |
          # Deployment commands here
```

### GitLab CI

```yaml
stages:
  - test
  - build
  - deploy

test:
  stage: test
  image: rust:1.75
  script:
    - cargo test

build:
  stage: build
  image: docker:latest
  services:
    - docker:dind
  script:
    - docker build -t $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA .
    - docker push $CI_REGISTRY_IMAGE:$CI_COMMIT_SHA

deploy:
  stage: deploy
  script:
    - echo "Deploy to production"
  only:
    - main
```

## Monitoring and Logging

### Health Checks

Configure health checks for your deployment:

```bash
# Docker health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

# Kubernetes health check
livenessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 30
  periodSeconds: 10

readinessProbe:
  httpGet:
    path: /health
    port: 8080
  initialDelaySeconds: 5
  periodSeconds: 5
```

### Logging

Configure centralized logging:

```yaml
# Docker Compose with logging
version: '3.8'
services:
  bitsacco-whatsapp-bot:
    image: bitsacco-whatsapp-bot:latest
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### Metrics

Expose metrics for monitoring:

```rust
// Add to main.rs
use prometheus::{Counter, Histogram, Registry};

let registry = Registry::new();
let request_counter = Counter::new("http_requests_total", "Total HTTP requests").unwrap();
let request_duration = Histogram::new("http_request_duration_seconds", "HTTP request duration").unwrap();

registry.register(Box::new(request_counter.clone())).unwrap();
registry.register(Box::new(request_duration.clone())).unwrap();
```

## Scaling

### Horizontal Scaling

For high-traffic deployments:

```yaml
# Docker Compose scaling
version: '3.8'
services:
  bitsacco-whatsapp-bot:
    image: bitsacco-whatsapp-bot:latest
    deploy:
      replicas: 3
      resources:
        limits:
          cpus: '0.5'
          memory: 512M
        reservations:
          cpus: '0.25'
          memory: 256M
```

### Load Balancing

Use nginx for load balancing:

```nginx
upstream bitsacco_bot {
    server bitsacco-whatsapp-bot-1:8080;
    server bitsacco-whatsapp-bot-2:8080;
    server bitsacco-whatsapp-bot-3:8080;
}

server {
    listen 80;
    server_name api.bitsacco.com;

    location / {
        proxy_pass http://bitsacco_bot;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

## Security

### SSL/TLS

Configure SSL certificates:

```bash
# Let's Encrypt with Certbot
certbot --nginx -d api.bitsacco.com

# Or use cloud provider SSL
# AWS ALB, Cloudflare, etc.
```

### Firewall

Configure firewall rules:

```bash
# UFW (Ubuntu)
ufw allow 22/tcp
ufw allow 80/tcp
ufw allow 443/tcp
ufw deny 8080/tcp  # Block direct access to app port
```

### Secrets Management

Use proper secrets management:

```bash
# Docker secrets
echo "your_secret" | docker secret create whatsapp_token -

# Kubernetes secrets
kubectl create secret generic bitsacco-secrets \
  --from-literal=whatsapp-token=your_token \
  --from-literal=bitsacco-token=your_token
```

## Troubleshooting

### Common Issues

1. **Port Already in Use**
   ```bash
   # Find process using port
   lsof -i :8080
   
   # Kill process
   kill -9 <PID>
   ```

2. **Environment Variables Not Loaded**
   ```bash
   # Check environment
   docker exec -it container_name env
   
   # Verify .env file
   cat .env
   ```

3. **Health Check Failing**
   ```bash
   # Test health endpoint
   curl http://localhost:8080/health
   
   # Check logs
   docker logs container_name
   ```

### Debug Mode

Enable debug logging:

```bash
export RUST_LOG=debug
cargo run
```

### Performance Issues

Monitor resource usage:

```bash
# Docker stats
docker stats

# System resources
htop
iostat
```

## Backup and Recovery

### Database Backups

If using persistent storage:

```bash
# Backup
docker exec container_name pg_dump -U user database > backup.sql

# Restore
docker exec -i container_name psql -U user database < backup.sql
```

### Configuration Backups

```bash
# Backup environment
cp .env .env.backup

# Backup Docker Compose
cp docker-compose.yml docker-compose.yml.backup
```

## Maintenance

### Updates

```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker-compose down
docker-compose up --build -d
```

### Monitoring

Set up monitoring alerts for:

- High CPU usage
- Memory consumption
- Error rates
- Response times
- Service availability

## Support

For deployment support:

- **Email**: devops@bitsacco.com
- **Documentation**: [https://docs.bitsacco.com](https://docs.bitsacco.com)
- **Status Page**: [https://status.bitsacco.com](https://status.bitsacco.com)
