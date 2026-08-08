---
title: Deployment
description: Deploy Ravix applications with Docker, environment configuration, and production considerations.
sidebar:
  order: 6
---

# Deployment

## Docker

### Multi-stage Build

```dockerfile
# Dockerfile
FROM rust:1.79 AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/my-api /usr/local/bin/my-api
EXPOSE 3000
CMD ["my-api"]
```

### Docker Compose

```yaml
# docker-compose.yml
version: "3.8"
services:
  app:
    build: .
    ports:
      - "3000:3000"
    environment:
      - DATABASE_URL=postgres://postgres:postgres@db:5432/myapp
      - RUST_LOG=info
      - APM_SERVICE_NAME=my-api
    depends_on:
      - db
    restart: unless-stopped

  db:
    image: postgres:16
    environment:
      - POSTGRES_DB=myapp
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=postgres
    volumes:
      - postgres_data:/var/lib/postgresql/data
    ports:
      - "5432:5432"

volumes:
  postgres_data:
```

## Environment Configuration

Use `config` crate with environment-specific files:

```rust
// src/config.rs
use config::{Config, File, Environment};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Settings {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub apm: ApmConfig,
    pub logger: LoggerConfig,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Deserialize)]
pub struct ApmConfig {
    pub service_name: String,
    pub log_path: String,
}

#[derive(Deserialize)]
pub struct LoggerConfig {
    pub min_level: String,
    pub classifications: Vec<ClassificationConfig>,
}

impl Settings {
    pub fn new() -> Result<Self, config::ConfigError> {
        let env = std::env::var("APP_ENV").unwrap_or_else(|_| "development".into());

        Config::builder()
            .add_source(File::with_name("config/default"))
            .add_source(File::with_name(&format!("config/{}", env)).required(false))
            .add_source(Environment::with_prefix("APP").separator("__"))
            .build()?
            .try_deserialize()
    }
}
```

```
config/
├── default.toml
├── development.toml
├── production.toml
└── test.toml
```

```toml
# config/default.toml
[server]
host = "0.0.0.0"
port = 3000

[database]
max_connections = 10

[apm]
service_name = "my-api"
log_path = "apm.ndjson"

[logger]
min_level = "info"
classifications = [
  { name = "PUBLIC", path = "public.ndjson" },
  { name = "CONFIDENTIAL", path = "confidential.ndjson" }
]
```

```toml
# config/production.toml
[server]
host = "0.0.0.0"
port = 8080

[database]
url = "postgres://user:pass@host:5432/db"
max_connections = 50

[logger]
min_level = "warn"
```

## Production Checklist

:::caution[Before Deploying]

- [ ] Set `RUST_LOG=warn` or `error` in production
- [ ] Configure APM with persistent log path (mounted volume)
- [ ] Set up log rotation for `.ndjson` files
- [ ] Use `cargo build --release` with `strip = true` in Cargo.toml
- [ ] Enable `lto = true` in release profile for smaller binaries
- [ ] Configure health check endpoint (`/health`)
- [ ] Set up graceful shutdown (SIGTERM handling)
- [ ] Configure reverse proxy (nginx, Caddy) for TLS termination
      :::

```toml
# Cargo.toml release profile
[profile.release]
lto = true
strip = true
codegen-units = 1
panic = "abort"
```

## Health Checks

```rust
#[controller("/health")]
impl HealthController {
    #[get("/")]
    pub async fn health(&self) -> Response {
        Http::json(json!({ "status": "healthy", "version": env!("CARGO_PKG_VERSION") }))
    }

    #[get("/ready")]
    pub async fn ready(&self, State(container): State<ContainerRef>) -> Response {
        let db: Arc<dyn Database> = container.resolve();
        match db.ping().await {
            Ok(_) => Http::json(json!({ "status": "ready" })),
            Err(_) => Http::status(503),
        }
    }
}
```

## Kubernetes

```yaml
# k8s/deployment.yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: my-api
spec:
  replicas: 3
  selector:
    matchLabels:
      app: my-api
  template:
    metadata:
      labels:
        app: my-api
    spec:
      containers:
        - name: my-api
          image: my-registry/my-api:latest
          ports:
            - containerPort: 3000
          env:
            - name: DATABASE_URL
              valueFrom:
                secretKeyRef:
                  name: my-api-secrets
                  key: database-url
            - name: APP_ENV
              value: "production"
          livenessProbe:
            httpGet:
              path: /health
              port: 3000
            initialDelaySeconds: 10
            periodSeconds: 30
          readinessProbe:
            httpGet:
              path: /health/ready
              port: 3000
            initialDelaySeconds: 5
            periodSeconds: 10
          resources:
            requests:
              memory: "256Mi"
              cpu: "250m"
            limits:
              memory: "512Mi"
              cpu: "500m"
---
apiVersion: v1
kind: Service
metadata:
  name: my-api
spec:
  selector:
    app: my-api
  ports:
    - port: 80
      targetPort: 3000
  type: ClusterIP
```

## Observability in Production

- **APM**: Ship `apm.ndjson` to Elasticsearch, Datadog, or Loki
- **Logs**: Ship `public.ndjson` and `confidential.ndjson` separately
- **Metrics**: Add `prometheus` feature to `ravix-apm` for `/metrics` endpoint
- **Tracing**: Correlation IDs propagate through middleware automatically

## Graceful Shutdown

```rust
use tokio::signal;

#[tokio::main]
async fn main() {
    let container = build_container();
    let app = App::new().container(container);
    let router = app.into_router();

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();

    axum::serve(listener, router)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .unwrap();
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c().await.expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }

    println!("Shutdown signal received, draining...");
}
```
