# Axum Quickstart Project

[![Rust CI](https://github.com/JohnBasrai/axum-quickstart/actions/workflows/rust.yml/badge.svg)](https://github.com/JohnBasrai/axum-quickstart/actions)
[![Latest Release](https://img.shields.io/github/v/release/JohnBasrai/axum-quickstart?style=flat-square)](https://github.com/JohnBasrai/axum-quickstart/releases)

## Overview

This project is a quickstart template for building a **production-ready RESTful API** in Rust using Axum and Tokio.
It includes:

 - Full CRUD endpoints for managing movies
 - HTML root (`/`) page with dynamic version display
 - Redis-backed storage for persistence
 - **Prometheus metrics collection and export**
 - **Health checks with Redis connectivity testing**
 - **Environment-based configuration**
 - Comprehensive HTTP integration tests using reqwest
 - Clean project architecture with domain/infrastructure separation

---

## Features

### API Endpoints
- `GET /` - HTML landing page with API documentation
- `GET /health` - Light health check
- `GET /health?mode=full` - Full health check including Redis connectivity
- `GET /metrics` - Prometheus metrics endpoint
- `POST /movies/add` - Add a new movie
- `GET /movies/get/{id}` - Fetch movie by ID
- `PUT /movies/update/{id}` - Update existing movie
- `DELETE /movies/delete/{id}` - Delete movie

### Observability
- **Metrics**: HTTP request duration, status codes, and business metrics (movie creation events)
- **Structured logging**: Configurable log levels and tracing
- **Health monitoring**: Redis connectivity checks

---

## Configuration

The application supports environment-based configuration:

| Variable | Default | Description |
|----------|---------|-------------|
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection string |
| `API_BIND_ADDR` | `127.0.0.1:8080` | Server bind address |
| `AXUM_METRICS_TYPE` | `noop` | Metrics backend (`prom` or `noop`) |
| `AXUM_LOG_LEVEL` | `debug` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `AXUM_SPAN_EVENTS` | `close` | Tracing span events (`full`, `enter_exit`, `close`) |

---

## Running the Application

Make sure you have a Redis server running locally.

You can quickly start Redis using Docker:

    docker run --name test-redis -p 6379:6379 -d redis

Then start the server:

    cargo run

The application will be available at http://localhost:8080.

### With Prometheus Metrics

To enable Prometheus metrics collection:

    AXUM_METRICS_TYPE=prom cargo run

Then visit:
- http://localhost:8080 - Application landing page
- http://localhost:8080/metrics - Prometheus metrics endpoint

---

## Running Tests

### Quick Start

Use the provided test scripts for convenience:

```bash
./scripts/test-all.sh  # Complete test suite (same as CI)
```

### Manual Testing

Integration tests are written in Rust and use real HTTP requests via reqwest.

To run tests manually:

1. **Start a Redis server locally**

   If not already running:

       docker run --name test-redis -p 6379:6379 -d redis

2. **Run all tests**

       cargo test

This will run:

- Unit tests inside `src/` (domain logic, metrics, app state)
- Full HTTP-based integration tests inside `tests/`
- Metrics endpoint integration tests

Each integration test spins up its own isolated Axum server instance and binds to a random port, allowing tests to run concurrently without conflict.

---

## Project Structure

    src/
      lib.rs              # Application setup and router creation
      main.rs             # Server startup entry point
      app_state.rs        # Shared application state
      domain/             # Business logic and abstractions
        metrics.rs        # Metrics trait definition
      handlers/           # Route handlers
        movies.rs         # Movie CRUD operations
        health.rs         # Health check endpoints
        metrics.rs        # Metrics endpoint
        root.rs           # Landing page
      infrastructure/     # External service implementations
        metrics/          # Metrics implementations module
          noop/           # No-op metrics implementation
          prometheus/     # Prometheus metrics implementation
    scripts/
      dev-setup.sh        # Development environment setup script
      run-integration-tests.sh # Integration tests with Docker orchestration
      run-unit-tests.sh   # Unit tests, linting, and formatting checks
      test-all.sh         # Complete test suite (same as CI)
    tests/
      integration.rs      # Basic integration tests
      metrics_endpoint.rs # Metrics-specific integration tests
---

## Architecture

The project follows **Clean Architecture** principles with clear separation between:

- **Domain Layer**: Business logic and trait definitions (`domain/`)
- **Infrastructure Layer**: External service implementations (`infrastructure/`)
- **Application Layer**: HTTP handlers and routing (`handlers/`, `lib.rs`)

This design allows for:
- Easy testing with mock implementations
- Swappable backends (metrics, storage, etc.)
- Clear dependency boundaries
- Maintainable and scalable code

---

## License

This project is licensed under the MIT License.
