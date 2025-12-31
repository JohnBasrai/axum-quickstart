# Axum Quickstart — Movie API with WebAuthn

Rust-based Movie API with production grade features: clean architecture, observability, comprehensive testing, and passwordless authentication with WebAuthn/Passkeys.

## Context

This project demonstrates real-world patterns for building and evolving backend services with Rust:

* **Clean Architecture** - dependency inversion with Repository pattern, domain-driven boundaries
* **Stateless service design** - horizontally scalable with externalized state (PostgreSQL, Redis)
* **Observability** - Prometheus metrics, health checks, structured logging
* **Comprehensive testing** - 57 integration tests with real services, not mocks
* **CI parity** - local development matches CI exactly

The codebase showcases incremental feature development, demonstrating how capabilities like passwordless authentication are added to existing systems rather than built as isolated greenfield demos.

## Overview

This project demonstrates:

- **Clean Architecture** - Dependency inversion with Repository pattern
- **PostgreSQL** - Authoritative persistence with ACID guarantees
- **Redis** - Session management, challenge storage, and caching
- **Integration Testing** - 57 automated tests validating real behavior against actual services
- **WebAuthn/Passkeys** - Passwordless authentication (Touch ID, Face ID, YubiKey, Windows Hello)

## Features

### Data Persistence & Caching
- **PostgreSQL** - ACID-compliant storage with foreign key constraints and migrations
- **Redis** - High-performance caching, session storage, ephemeral challenge data
- **Repository Pattern** - Clean abstraction layer with trait-based contracts

### Strong Authentication
- **Registration** - Create passkey credentials with authenticators (WebAuthn)
- **Authentication** - Login using biometrics or hardware keys
- **Credential Management** - List and delete registered passkeys
- **Replay Attack Prevention** - Cryptographic counter validation
- **Multi-device Support** - Multiple passkeys per account

### Observability & Operations
- **Health Checks** - Light and full modes with Redis connectivity validation
- **Prometheus Metrics** - HTTP request duration, status codes, business metrics (movie creation events)
- **Structured Logging** - Tracing instrumentation with configurable levels and span events

### CRUD Operations
- **Movies API** - Full create, read, update, delete with validation
- **Hash-based IDs** - Deterministic key generation from normalized data
- **Input Sanitization** - Whitespace normalization, range validation

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

### Core Operations
- `GET /` - HTML landing page with version and endpoint listing
- `GET /health` - Health check (light mode by default)
- `GET /health?mode=full` - Full health check including Redis connectivity
- `GET /metrics` - Prometheus metrics in text exposition format

### Movies (Redis-backed CRUD)
- `GET /movies/get/{id}` - Fetch movie by ID (200 OK or 404 Not Found)
- `POST /movies/add` - Create movie (201 Created or 409 Conflict if duplicate)
- `PUT /movies/update/{id}` - Update movie (200 OK, allows overwrite)
- `DELETE /movies/delete/{id}` - Delete movie (204 No Content or 404 Not Found)

### WebAuthn (Passwordless Authentication)
- `POST /webauthn/register/start` - Begin passkey registration with challenge generation
- `POST /webauthn/register/finish` - Complete passkey registration and store credential
- `POST /webauthn/auth/start` - Begin passkey authentication with challenge
- `POST /webauthn/auth/finish` - Complete passkey authentication and create session
- `GET /webauthn/credentials` - List user's registered passkeys (requires Bearer token)
- `DELETE /webauthn/credentials/{id}` - Delete specific passkey (requires Bearer token)

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

**Framework & Runtime:**
- Rust 1.92.0 (pinned for sqlx-cli 0.8.2 compatibility) with Axum 0.8 web framework
- Tokio async runtime
- SQLx for compile-time verified queries

**Data Layer:**
- PostgreSQL 16 for persistent storage
- Redis 7 for sessions, caching, and ephemeral data

**Authentication:**
- `webauthn-rs` - WebAuthn protocol implementation
- Public key cryptography (ECDSA, EdDSA)
- FIDO2/WebAuthn specification compliance

**Testing & CI:**
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

The project follows **clean architecture with explicit boundaries:**

* **Domain** — business logic and trait contracts (Repository, User, Credential, Movie)
* **Infrastructure** — concrete implementations (PostgreSQL, Redis, WebAuthn, Metrics)
* **Application** — Axum handlers and HTTP routing

Dependencies flow inward; implementations never leak outward. The service is designed for **horizontal scalability**: all persistent state (PostgreSQL) and ephemeral state (Redis) are externalized, keeping application instances stateless.

**Key architectural patterns:**
- Repository pattern for data access abstraction
- Dependency injection via AppState
- EMBP (Explicit Module Boundary Pattern) for module organization
- Integration testing against real services, not mocks

## Security Highlights

**Authentication:**
- **Phishing-resistant authentication** - WebAuthn's cryptographic challenge-response prevents phishing
- **Replay attack prevention** - Signature counters validated on every authentication
- **Session expiry** - Redis automatically expires sessions (7 days) and challenges (5 minutes)
- **Generic error messages** - Prevent username enumeration attacks

**Data Integrity:**
- **ACID-compliant storage** - PostgreSQL ensures data integrity with foreign key constraints
- **Foreign key constraints** - Credentials cannot exist without users; cascade deletion enforced
- **Atomic operations** - Redis GETDEL ensures single-use challenges
- **Input validation** - All user inputs sanitized and validated before processing

## WebAuthn Feature Development

The following phases demonstrate **incremental addition of WebAuthn/Passkeys** to the existing REST API foundation. This showcases how modern authentication capabilities are integrated into real systems rather than built in isolation.

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

**WebAuthn:**
- Browser-based E2E testing with Playwright for full signature verification
- Production deployment guide with HTTPS requirements

**General Improvements:**
- Enhanced documentation with architecture flow diagrams
- Additional API feature demonstrations
- Performance benchmarking suite

## References

- [WebAuthn Architecture](docs/webauthn-architecture.md) - Implementation details
- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/) - W3C standard
- [SQLx Offline Mode](docs/sqlx-offline-mode-howto.md) - CI/CD guide

## License

MIT
