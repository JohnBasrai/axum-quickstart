# Axum Quickstart — Production-Oriented API Foundation

Axum-based REST API demonstrating incremental integration of WebAuthn/Passkeys authentication into an existing service, with emphasis on correctness, observability, and testing.

## Context

This project demonstrates how to build and evolve a real-world Rust API service with:

* **Stateless service design** - horizontally scalable, externalized state
* **Observability** - metrics, health checks, structured logging
* **Comprehensive integration testing** - real PostgreSQL and Redis, not mocks
* **CI parity with local development** - same workflow, same results

**Current work:** Incrementally adding WebAuthn/Passkeys authentication to the existing base, demonstrating how modern authentication is added to real systems—not greenfield demos.

## Overview

This project demonstrates:

- **WebAuthn/Passkeys** - Passwordless authentication (Touch ID, Face ID, YubiKey, Windows Hello)
- **Clean Architecture** - Dependency inversion with Repository pattern
- **PostgreSQL** - Authoritative persistence with ACID guarantees for cryptographic credentials
- **Redis** - Session management, challenge storage, and caching
- **Integration Testing** - 57 automated tests validating real behavior against actual services

## Features

### WebAuthn Authentication (Passwordless)
- **Registration** - Create passkey credentials with authenticators
- **Authentication** - Login using biometrics or hardware keys
- **Credential Management** - List and delete registered passkeys
- **Replay Attack Prevention** - Cryptographic counter validation
- **Multi-device Support** - Multiple passkeys per account

### Core API
- Health checks and metrics (Prometheus)
- CRUD operations with Redis backend
- Session-based authentication

## Quick Start

### Using Scripts (Recommended)

```bash
# Start all services (PostgreSQL + Redis)
source ./scripts/startup.sh

# Run the server
cargo run

# Server running at http://localhost:3000

# Stop all services when done
./scripts/shutdown.sh
```

### Manual Setup

```bash
# Copy environment config
cp .env.example .env

# Start services (PostgreSQL + Redis)
docker compose up -d

# Run database migrations
cargo install sqlx-cli --no-default-features --features postgres
sqlx migrate run

# Start API server
cargo run

# Server running at http://localhost:3000
```

## Local Development

### Prerequisites

- Rust toolchain (install via [rustup](https://rustup.rs/))
- Docker & Docker Compose
- `sqlx-cli` for database migrations

```bash
cargo install sqlx-cli --no-default-features --features postgres
```

### Development Workflow

```bash
# Format code
cargo fmt

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Build
cargo build

# Watch mode (requires cargo-watch)
cargo watch -x run
```

**Contributing:** See [CONTRIBUTING.md](CONTRIBUTING.md)

## API Endpoints

### WebAuthn
- `POST /webauthn/register/start` - Begin passkey registration
- `POST /webauthn/register/finish` - Complete passkey registration
- `POST /webauthn/auth/start` - Begin passkey authentication
- `POST /webauthn/auth/finish` - Complete passkey authentication
- `GET /webauthn/credentials` - List user's passkeys (requires session)
- `DELETE /webauthn/credentials/:id` - Delete passkey (requires session)

### Core
- `GET /health` - Health check
- `GET /metrics` - Prometheus metrics
- `GET /api/movies` - List movies (demo CRUD)
- `POST /api/movies` - Create movie (demo CRUD)

**Architecture details:** See [docs/webauthn-architecture.md](docs/webauthn-architecture.md)

## Configuration

### Runtime Environment Variables

| Variable | Default | Description |
|:---------|:--------|:------------|
| `REDIS_URL` | `redis://127.0.0.1:6379` | Redis connection string |
| `DATABASE_URL` | `postgresql://postgres:postgres@localhost:5432/axum_db` | PostgreSQL connection string |
| `API_BIND_ADDR` | `127.0.0.1:8080` | Server bind address |
| `AXUM_METRICS_TYPE` | `noop` | Metrics backend (`prom` for Prometheus or `noop`) |
| `AXUM_LOG_LEVEL` | `debug` | Log level (`trace`, `debug`, `info`, `warn`, `error`) |
| `AXUM_SPAN_EVENTS` | `close` | Tracing span events (`full`, `enter_exit`, `close`) |
| `AXUM_DB_RETRY_COUNT` | `50` | Database connection retry attempts during startup |
| `AXUM_DB_ACQUIRE_TIMEOUT_SEC` | `30` | Database connection pool acquire timeout (seconds) |

**Note:** PostgreSQL is required for WebAuthn functionality. Copy `.env.example` to `.env` and customize as needed.

## Testing

Run the complete test suite (matches CI exactly):

```bash
./scripts/test-all.sh
```

**Individual test suites:**
- `./scripts/run-unit-tests.sh` - Unit tests, linting, format checks, doctests
- `./scripts/run-integration-tests.sh` - Integration tests with Docker services  
- `./scripts/ci-local.sh` - Local CI simulation using `act`

**Test Coverage:** 57 tests across unit, integration, and WebAuthn flows. See scripts for detailed breakdowns.

### Known Limitations

⚠️ **WebAuthn Verification Tests (Issue #33)**

Five WebAuthn tests are currently ignored due to upstream test utility limitations:
- 2 registration tests (`test_register_finish_*`)
- 3 authentication tests (`test_auth_start_*`)

These tests require injectable WebAuthn verifier instances for full end-to-end verification. The current implementation validates:
- ✅ Database operations (user/credential CRUD)
- ✅ Challenge storage and expiry in Redis
- ✅ Counter validation and replay prevention
- ✅ Session creation and validation
- ⚠️ WebAuthn signature verification (requires browser automation)

Full verification requires browser-based E2E tests (planned for Phase 5).

## Technology Stack

**Core:**
- Rust 1.92.0 with Axum 0.8 web framework
- PostgreSQL 16 for credential storage
- Redis 7 for sessions and challenges
- SQLx for compile-time verified queries

**WebAuthn:**
- `webauthn-rs` - Protocol implementation
- Public key cryptography (ECDSA, EdDSA)
- FIDO2/WebAuthn specification compliance

**Testing:**
- Docker Compose for integration test services
- GitHub Actions CI/CD with caching
- Local CI runner using `act`

## Project Structure

```
axum-quickstart/
├── src/
│   ├── domain/              # Business logic (Repository trait, models)
│   ├── infrastructure/      # Implementation (PostgreSQL, Redis, WebAuthn)
│   ├── handlers/            # HTTP handlers (WebAuthn, CRUD, health)
│   └── lib.rs               # Public API gateway (EMBP)
├── tests/                   # Integration tests
├── migrations/              # SQLx database migrations
├── scripts/                 # Development and CI scripts
├── docs/                    # Architecture and setup guides
└── docker-compose.yml       # PostgreSQL + Redis services
```

**Architecture:** Follows [EMBP (Explicit Module Boundary Pattern)](docs/embp.md) and Clean Architecture principles.

**Dependencies flow inward:** Domain defines contracts, Infrastructure provides implementations, Application orchestrates handlers. No database or transport types leak into domain logic.

## Architecture Overview

The project follows a **clean, explicit boundary model:**

* **Domain** — business logic and trait contracts (Repository, User, Credential)
* **Infrastructure** — concrete implementations (PostgreSQL, Redis, WebAuthn, Metrics)
* **Application** — Axum handlers and routing

Dependencies flow inward; implementations never leak outward. The service is intentionally designed to be **horizontally scalable**: all persistent and ephemeral state is externalized (PostgreSQL and Redis), and application instances remain stateless.

## Security Highlights

- **Phishing-resistant authentication** - WebAuthn's cryptographic challenge-response prevents phishing
- **Replay attack prevention** - Signature counters validated on every authentication
- **ACID-compliant credential storage** - PostgreSQL ensures data integrity with foreign key constraints
- **Session expiry and challenge TTLs** - Redis automatically expires challenges (5min) and sessions (7 days)
- **Foreign key constraints** - Credentials cannot exist without users; cascade deletion enforced
- **Generic error messages** - Prevent username enumeration attacks
- **Atomic operations** - Redis GETDEL ensures single-use challenges

## WebAuthn Implementation Phases

The following phases describe the incremental addition of **WebAuthn / Passkeys** to the existing axum-quickstart foundation:

### Phase 1: Database Infrastructure ✅ Complete
- Users and Credentials tables with foreign key constraints
- Repository pattern with Clean Architecture
- Counter tracking for replay prevention
- Multiple credentials per user (multi-device support)
- 9 unit tests + 1 schema test

### Phase 2: Registration Flow ✅ Complete  
- Registration endpoints (`/webauthn/register/start`, `/webauthn/register/finish`)
- Redis-backed challenge storage with automatic expiry
- webauthn-rs integration for protocol implementation
- 6 integration tests (2 ignored - Issue #33)

### Phase 3: Authentication Flow ✅ Complete
- Authentication endpoints (`/webauthn/auth/start`, `/webauthn/auth/finish`)
- Session token generation with 7-day TTL
- Counter validation to prevent replay attacks
- Generic error messages to prevent username enumeration
- 6 integration tests (3 ignored - Issue #33)

### Phase 4: Credential Management ✅ Complete
- Credential listing endpoint (`GET /webauthn/credentials`)
- Credential deletion endpoint (`DELETE /webauthn/credentials/:id`)
- Session-based authentication with Bearer tokens
- Ownership verification for secure deletion
- 7 integration tests

### Phase 5: Browser Testing & Documentation (Planned)
- Browser-based E2E tests with Playwright
- Full WebAuthn signature verification
- Production deployment guide
- Enhanced flow diagrams

## Next Steps

- **Phase 5:** Browser-based E2E testing with Playwright
- Enhanced documentation with flow diagrams
- Production deployment guide

## References

- [WebAuthn Architecture](docs/webauthn-architecture.md) - Implementation details
- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/) - W3C standard
- [SQLx Offline Mode](docs/sqlx-offline-mode-howto.md) - CI/CD guide

## License

MIT
