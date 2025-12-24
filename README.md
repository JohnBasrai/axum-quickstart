# Axum Quickstart — Production-Oriented API Foundation

A **production-grade Axum starter** focused on **correctness, observability, scalability, and security-ready persistence** — not a toy CRUD demo.

This repository builds on an existing, mature Axum foundation and incrementally adds **WebAuthn / Passkeys** support in clearly defined phases.
The README is updated **only at the end of each WebAuthn phase** to reflect *completed, validated capabilities*.

> **Baseline:** axum-quickstart (existing, production-ready foundation)
> **WebAuthn Status:** Phase 1 complete — Persistence & Integrity
> **Next milestone:** Phase 2 — WebAuthn flows integrated into the application layer

---

## Context

This repository began as **axum-quickstart**, a production-oriented Axum API foundation with:

* Stateless service design
* Externalized state
* Observability (metrics, health checks, structured logging)
* Comprehensive integration testing
* CI parity with local development

The current work **incrementally adds WebAuthn / Passkeys** to this existing base.
The WebAuthn effort is intentionally broken into multiple phases, starting with **persistence and data integrity** before introducing application-level authentication flows.

This approach mirrors how modern authentication is added to real systems—not greenfield demos.

---

## Project Goals

This project exists to demonstrate how to build and evolve a real-world Rust API service with:

* **Correctness** — validated through real integration tests
* **Observability** — metrics, health checks, and structured logging
* **Scalability** — stateless services with externalized state
* **Security-ready persistence** — PostgreSQL-backed integrity for authentication data

These are architectural constraints, not aspirational statements.

---

## Technology Stack

* **Rust / Tokio** — async runtime
* **Axum** — HTTP routing and request handling
* **PostgreSQL + SQLx** — authoritative persistence with compile-time query checking
* **Redis** — caching and ephemeral state
* **Prometheus** — metrics collection
* **Tracing** — structured logging and spans
* **Docker Compose** — local parity with CI

---

## Current Capabilities (WebAuthn – End of Phase 1)

### Persistence & Data Integrity ✅

PostgreSQL is the **source of truth** for all WebAuthn-related, security-sensitive data.

The database layer enforces:

* Foreign key integrity (credentials cannot exist without users)
* Cascade deletion (removing a user removes all credentials)
* Monotonic counters to prevent replay attacks
* Explicit schema migrations managed via SQLx

All guarantees are validated via **real integration tests**, not mocks.

---

### Repository Architecture ✅

* Domain layer defines **behavior and trait contracts**
* Infrastructure layer provides **PostgreSQL-backed implementations**
* No database or transport types leak into the domain
* Repository implementations are testable in isolation

This structure deliberately precedes application-level WebAuthn flows.

---

### Integration Testing Strategy ✅

Integration tests validate **real behavior**, not mocked interactions.

Each test:

* Runs against PostgreSQL and Redis
* Applies migrations automatically
* Cleans up state after execution
* Can run concurrently and in any order

CI executes the **same workflow** as local development.

---

### Observability ✅

* Prometheus metrics endpoint
* Structured logging via `tracing`
* Health checks with Redis connectivity validation

---

## WebAuthn / Passkeys (Incremental Integration)

WebAuthn support is being added **incrementally** to the existing axum-quickstart foundation.

### Phase 1 — Persistence & Integrity ✅ (Complete)

* Users table
* Credentials table
* Counter tracking for replay prevention
* Multiple credentials per user (multi-device support)
* Referential integrity guarantees
* CI-validated integration tests

### Phase 2 — Application Integration 🚧 (Next)

Planned work:

* WebAuthn registration flow
* WebAuthn authentication flow
* Redis-backed challenge storage
* Integration into Axum handlers
* Removal of unused Phase-1 scaffolding

This README will be updated again **after Phase 2 is complete**.

---

## Configuration

### Runtime Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection string |
| `API_BIND_ADDR` | `127.0.0.1:8080` | Server bind address |
| `AXUM_METRICS_TYPE` | `noop` | Metrics backend (`prom` or `noop`) |
| `AXUM_LOG_LEVEL` | `debug` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `AXUM_SPAN_EVENTS` | `close` | Tracing span events (`full`, `enter_exit`, `close`) |
| `AXUM_DB_RETRY_COUNT` | `50` | Number of database connection retry attempts during startup |
| `AXUM_DB_ACQUIRE_TIMEOUT_SEC` | `30` | Database connection pool acquire timeout (seconds) |

PostgreSQL is **required** for WebAuthn Phase 1 and beyond.

---

## Local Development

### Prerequisites

* Rust toolchain
* Docker & Docker Compose
* `sqlx-cli` for migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

---

### Start Dependencies

```bash
docker compose up -d
```

Ensure PostgreSQL and Redis are healthy.

---

### Run Migrations

```bash
sqlx migrate run
```

---

### Run the Server

```bash
cargo run
```

### The output of the above steps should look something like this.

```
bash $ docker compose up -d
[+] Running 2/2
 ✔ Container axum-quickstart-redis-1     Running                                                                                                     0.0s 
 ✔ Container axum-quickstart-postgres-1  Running                                                                                                     0.0s 
bash $ sqlx migrate run
Applied 20250101000001/migrate create users table (11.473996ms)
Applied 20250101000002/migrate create credentials table (15.291935ms)
bash $ cargo run
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
     Running `target/debug/axum-quickstart`
2025-12-24T19:48:26.198948Z  INFO axum_quickstart::infrastructure::database::postgres_repository: src/infrastructure/database/postgres_repository.rs:61: 🚨 axum-quickstart attaching to database at: "postgres://postgres:postgres@localhost:5432/axum_quickstart_test"
2025-12-24T19:48:26.257727Z  INFO axum_quickstart: src/main.rs:51: Starting axum server 1.3.3 on endpoint:127.0.0.1:8080
^C2025-12-24T19:48:32.693051Z  INFO axum_quickstart: src/main.rs:68: Caught Control-C. Closing server gracefully...
bash $ █
```

---

## Testing

### Full Test Suite (Recommended)

```bash
./scripts/test-all.sh
```

This mirrors CI exactly.

### Integration Tests Only

```bash
./scripts/run-integration-tests.sh
```

---

## Architecture Overview

The project follows a **clean, explicit boundary model**:

* **Domain** — business logic and trait contracts
* **Infrastructure** — concrete implementations (PostgreSQL, Redis, metrics)
* **Application** — Axum handlers and routing

Dependencies flow inward; implementations never leak outward.

The service is intentionally designed to be **horizontally scalable**: all persistent and ephemeral state is externalized (PostgreSQL and Redis), and application instances remain stateless.

---

## WebAuthn Implementation Phases

The following phases describe the incremental addition of **WebAuthn / Passkeys** to the existing axum-quickstart foundation:

* **Phase 1** — Persistence & Integrity ✅
* **Phase 2** — WebAuthn Flows 🚧
* **Phase 3** — Credential Management & Hardening (planned)

The README is updated **only at phase boundaries**.

---

## License

MIT License
