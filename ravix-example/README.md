# Ravix Blog API Example

A full-featured blog application built with the Ravix framework, demonstrating:

- MongoDB persistence with async/await
- JWT authentication with Argon2 password hashing
- APM instrumentation with Elastic Kibana integration
- Integration testing with testcontainers
- Docker containerization

## Quick Start

### Prerequisites

- Rust 1.79+
- Docker and Docker Compose
- MongoDB (or use the provided Docker setup)

### Local Development

1. Copy the environment template:

```bash
cp .env.example .env
```

2. Start MongoDB:

```bash
docker compose up mongo -d
```

3. Run the application:

```bash
cargo run -p ravix-example
```

The API will be available at `http://localhost:3001`.

### Docker Compose

Start the full stack (API + MongoDB + Filebeat):

```bash
docker compose up --build
```

Start with ELK stack for APM visualization:

```bash
docker compose --profile elk up --build
```

This starts Elasticsearch and Kibana in addition to the API and MongoDB.

## API Endpoints

### Authentication

| Method | Path             | Description                              |
| ------ | ---------------- | ---------------------------------------- |
| POST   | `/auth/register` | Register a new user (returns JWT + user) |
| POST   | `/auth/login`    | Login (returns JWT + user)               |

### Posts

| Method | Path         | Auth        | Description     |
| ------ | ------------ | ----------- | --------------- |
| GET    | `/posts`     | -           | List all posts  |
| GET    | `/posts/:id` | -           | Get single post |
| POST   | `/posts`     | JWT         | Create a post   |
| PUT    | `/posts/:id` | JWT + owner | Update post     |
| DELETE | `/posts/:id` | JWT + owner | Delete post     |

### Comments

| Method | Path                                | Auth        | Description             |
| ------ | ----------------------------------- | ----------- | ----------------------- |
| GET    | `/posts/:post_id/comments`          | -           | List comments on a post |
| POST   | `/posts/:post_id/comments`          | JWT         | Add a comment           |
| PUT    | `/posts/:post_id/comments/:id`      | JWT + owner | Edit a comment          |
| DELETE | `/posts/:post_id/comments/:id`      | JWT + owner | Delete a comment        |
| POST   | `/posts/:post_id/comments/:id/like` | JWT         | Like a comment          |
| DELETE | `/posts/:post_id/comments/:id/like` | JWT         | Unlike a comment        |

## Kibana Dashboard Setup

### 1. Create Index Pattern

1. Open Kibana at `http://localhost:5601`
2. Navigate to **Stack Management** → **Index Patterns**
3. Create index pattern: `ravix-apm-*`
4. Set timestamp field: `start_time`

### 2. Create Dashboards

#### Throughput Panel

- **Visualization Type**: TSVB (Time Series Visual Builder)
- **Panel Type**: Metric
- **Query**: `type: "transaction"`
- **Split by**: `name.keyword`
- **Aggregation**: Count

#### Latency Panel

- **Visualization Type**: TSVB
- **Panel Type**: Timeseries
- **Query**: `type: "transaction"`
- **Metrics**:
  - P50: `percentiles(duration_ms, 50)`
  - P95: `percentiles(duration_ms, 95)`
  - P99: `percentiles(duration_ms, 99)`
- **Split by**: `name.keyword`

#### DB Operation Breakdown

- **Visualization Type**: Lens or Discover
- **Query**: `span_type: "db"`
- **Group by**: `metadata.collection`, `metadata.op`
- **Metrics**: Count, Average `duration_ms`

#### Error Rate Panel

- **Visualization Type**: TSVB
- **Panel Type**: Metric
- **Query**: `result: "HTTP 5*"`
- **Aggregation**: Count rate over time

#### Argon2 Cost Panel

- **Visualization Type**: TSVB
- **Query**: `name: "argon2.verify"` or `name: "argon2.hash"`
- **Metrics**: P50/P95/P99 of `duration_ms`
- **Expected range**: 50-200ms for Argon2id default parameters

### 3. Distributed Tracing

Use the `correlation_id` field to trace requests across logs:

1. Find a transaction by correlation ID in Discover
2. View the waterfall of spans under that transaction
3. Each span shows `duration_ms` and metadata for debugging

## Running Tests

```bash
# Unit and integration tests
cargo test -p ravix-example

# Benchmarks
cargo bench -p ravix-example
```

## Postman Documentation

A complete Postman collection and documentation is available in the `docs/` directory:

- **[POSTMAN.md](docs/POSTMAN.md)** - Complete API documentation with sample requests and responses
- **[Blog API.postman_collection.json](Blog%20API.postman_collection.json)** - Ready-to-import Postman collection

### Quick Import

1. Start the services: `docker compose up --build`
2. Open Postman and import `ravix-example/Blog API.postman_collection.json`
3. Follow the testing workflow in the collection

## Environment Variables

| Variable             | Description               | Default                     |
| -------------------- | ------------------------- | --------------------------- |
| `MONGO_URI`          | MongoDB connection string | `mongodb://localhost:27017` |
| `MONGO_DB`           | Database name             | `blog`                      |
| `JWT_SECRET`         | Secret for signing JWTs   | (required)                  |
| `JWT_EXPIRY_SECONDS` | Token expiry time         | `3600`                      |
| `SERVER_PORT`        | Server bind address       | `0.0.0.0:3001`              |

## Architecture

```
ravix-example/
├── src/
│   ├── config.rs        # AppConfig from environment
│   ├── db.rs            # MongoDB initialization
│   ├── errors.rs        # AppError enum
│   ├── middleware.rs    # JWT guard
│   ├── models/          # User, Post, Comment domain types
│   ├── repositories/      # MongoDB-backed repositories
│   ├── services/        # Business logic with APM spans
│   └── controllers/     # HTTP handlers
├── tests/integration/   # Integration tests with testcontainers
├── benches/             # Criterion benchmarks
└── Dockerfile           # Multi-stage build
```

## APM Instrumentation

All operations are instrumented with spans:

- **HTTP transactions**: Automatic via `apm_middleware`
- **Service layer**: `post.create`, `auth.login`, etc.
- **Repository layer**: `mongo.posts.insert_one`, `mongo.users.find_one`, etc.
- **Argon2**: `argon2.hash`, `argon2.verify` (blocking spans)

The APM log (`apm.ndjson`) is shipped to Elasticsearch via Filebeat and can be visualized in Kibana.
