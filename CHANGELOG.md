# Changelog

All notable changes to this project will be documented in this file.

This project adheres to [Keep a Changelog](https://keepachangelog.com/en/1.0.0/)
and [Semantic Versioning](https://semver.org/).

## [Unreleased]

## [1.4.0] - 2025-12-25

### Added (Phase 3 - Authentication)
- WebAuthn authentication endpoints (`/webauthn/auth/start`, `/webauthn/auth/finish`)
- Session token generation with UUID-based tokens stored in Redis
- Session management module with 7-day TTL (604,800 seconds)
- Counter validation to prevent replay attacks (increments on each authentication)
- Integration tests for authentication flow, counter validation, and session management

### Security (Phase 3)
- Generic "Authentication failed" error messages prevent username enumeration
- Counter replay attack detection (rejects counter <= stored value)
- Atomic GETDEL for challenge retrieval (single-use challenges)
- Challenge expiry enforced via Redis TTL (5 minutes)

### Added (Phase 2 - Registration)
- **Phase 2: WebAuthn Application Integration (Issue #25)**
  - WebAuthn registration flows
  - Redis-backed challenge storage with 5-minute expiry
  - Atomic challenge consumption via Redis GETDEL
  - PostgreSQL-backed credential persistence
  - Multi-credential (multi-device) support
  - Integrated Axum handlers for WebAuthn flows

### Changed
- Normalize Redis configuration to `REDIS_URL`
- Improve metrics test realism (remove forced serialization)
- Harden test environment setup with default-only env initialization

### Testing
- CI validates full WebAuthn registration flow
- ⚠️ Two WebAuthn verification tests are ignored due to upstream limitations (see Issue #33)

### Dependencies
- Switched WebAuthn state serialization to `serde_json`

## [1.3.3] - 2025-12-23

### Added
- **Phase 1: WebAuthn Database Infrastructure (Issue #24)**
  - PostgreSQL container and docker-compose configuration
  - SQLx migrations for `users` and `credentials` tables
  - Repository trait abstraction in domain layer  
  - PostgresRepository implementation with full CRUD operations
  - Comprehensive integration test suite (9 tests, 0.17s execution)
  - Shared tokio Runtime for database tests (enables concurrent execution)
  - Connection pool configuration (min=2, max=15 connections)
  - Design documentation: `docs/webauthn-architecture.md`
  - CI: GitHub Actions services for PostgreSQL and Redis
  - CI: Rust toolchain pinned to 1.88 (sqlx-cli compatibility)

### Technical Details
- Database schema supports multi-device passkeys (multiple credentials per user)
- Replay attack prevention via credential counter
- CASCADE delete from users to credentials
- Test suite executes in 0.17s with concurrent support

### Dependencies
- Added: `sqlx` (PostgreSQL feature), `uuid`, `async-trait`

Part of WebAuthn implementation (Startup Package 2).

## [1.3.2] — 2025-01-22

### Changed
- Remove OpenSSL system dependency by disabling reqwest default features
- Reduces build dependencies and eliminates libssl-dev requirement
- Integration tests continue to use HTTP for localhost connections

## [1.3.1] - 2025-06-14

### Fixed
- Resolved Prometheus metrics recorder initialization conflicts in tests
- Fixed Clippy lint warnings
- Updated CI workflow to use test scripts for consistency with dev mode
- Improved test reliability with proper singleton pattern for metrics initialization

### Changed
- Enhanced Prometheus recorder initialization using `OnceLock::get_or_init()` for thread-safe single initialization
- Updated GitHub Actions workflow to use `dtolnay/rust-toolchain` (replacing deprecated `actions-rs/toolchain`)
- Improved CI caching strategy with more specific cache paths and restore keys

## [1.3.0] - 2025-06-14

### Added
- Prometheus metrics collection and export
- `/metrics` endpoint for Prometheus scraping
- HTTP request duration and status code metrics
- Business metrics (movie creation events)
- Environment-based metrics configuration (`AXUM_METRICS_TYPE`)
- Full health check mode with Redis connectivity testing
- Domain/Infrastructure architecture separation following EMBP
- Comprehensive metrics integration tests

### Changed
- Enhanced `/health` endpoint with optional `mode=full` parameter
- Updated project structure with clean architecture principles
- Improved documentation with configuration table and architecture overview

### Dependencies
- Added `metrics`, `metrics-exporter-prometheus`, `prometheus`

## [1.2.2] – 2025-05-03
### Added
- Integration test for graceful shutdown using `.with_graceful_shutdown()`
- Integration test for root `/` endpoint

## [1.1.2] — 2025-04-28

### Added
- Moved Python-based `api-test.py` to `scripts/` folder and marked it as deprecated.
- Added full Rust-based integration tests under `tests/` using `reqwest` and `cargo test`.
- Implemented `spawn_app!` macro to simplify test setup and reduce duplication.
- Updated `README.md` to reflect new integration test workflow (`cargo test`) and removed
  outdated Full API Usage section.

### Fixed
- Corrected handling of validation error codes: missing fields now return
  `422 Unprocessable Entity`, and invalid business logic inputs (stars, year) return
  `400 Bad Request`.

### Changed
- Deprecated old external manual testing via `api-test.py` in favor of Rust-native
  integration tests.

## [1.1.1] — 2025-04-28

### Added
- Implemented title and year normalization in `Movie::sanitize()`.
- Added SHA1-based deterministic key generation from normalized movie title and year.
- Added unit tests for movie sanitization and key generation logic.
- Added conflict detection (`409 Conflict`) for duplicate movies (same title and year).
- Added structured debug logging for normalized key strings before hashing.
- Added dependencies `sha1`, `hex`, `regex`, and `chrono` for movie key generation
  and validation.
- Added `AXUM_LOG_LEVEL` environment variable support for setting tracing log level
  dynamically.

### Fixed
- Fetching a non-existent movie no longer causes internal server error; returns correct 404 status.
- Redis `SET` command now uses correct single-string storage format, avoiding syntax errors.

### Changed
- `save_movie` now serializes the `Movie` struct to JSON before storing in Redis 
   (fixes server syntax errors).
- `get_movie` now retrieves an `Option<String>` from Redis, properly handling missing keys 
   with 404 responses.
- ⚠️ Redis storage format changed: `Movie` structs are now JSON-serialized. Older incorrectly 
  stored keys may not deserialize correctly; manual data migration may be needed.
- Improved Redis error handling for `save_movie` and `get_movie` operations.
- `POST /movies/add` no longer accepts client-supplied IDs. The server generates the ID 
  automatically.
- `POST /movies/add` now returns `201 Created` with a JSON body containing 
  `{ "id": "<generated_id>" }`.
- Updated **api-test.py** to capture server-generated IDs dynamically.
- Updated **README.md** examples to show server-generated IDs and 409 Conflict behavior.
- Simplified README examples and pointed users to `api-test.py` for full API usage.

## [1.1.0] — 2025-04-27

### Added
- Add `GET /health` endpoint for server and Redis liveness checks
- Support optional `mode=light` (default) or `mode=full` (Redis ping) via query parameter
- Extend `api-test.py` to validate health checks (no params, mode=full, mode=light)
- Added graceful shutdown support for `Ctrl+C` and `SIGTERM` signals.

### Changed
- Modularize handlers: split movies, health, and shared_types into separate modules
- Improve server startup logging: include version and bind address at launch

## [1.0.1] — 2025-04-27

### Changed
- Update README.md to reflect `/movies` API namespace restructure
- Document all CRUD operations (POST, GET, PUT, DELETE) under `/movies`
- Update usage instructions to refer to `api-test.py` instead of `api-demo.sh`

## [1.0.0] — 2025-04-27

### Added
- Migrated backend storage from in-memory `HashMap` to persistent Redis database
- Introduced `/movies` namespace for all movie-related routes:
```
     POST    /movies/add         — Add a movie
     GET     /movies/get/{id}    — Fetch a movie by ID
     PUT     /movies/update/{id} — Update a movie by ID
     DELETE  /movies/delete/{id} — Delete a movie by ID
```
- Implemented `DELETE` endpoint for removing movie entries
- Added automatic version logging using `env!("CARGO_PKG_VERSION")`
- Extended integration test coverage via `api-test.py`:
  - Test add, fetch, update, delete, and 404-not-found behavior
- Added detailed handler tracing spans with optional event control via environment variable

### Changed
- Refactored application to use `AppState` for Redis connection management
- Updated integration testing from manual shell script (`api-demo.sh`) to Python script (`api-test.py`)
- Improved error handling for Redis operations (graceful mapping to appropriate HTTP status codes)
- Standardized API route structure for consistency and future expansion

### Removed
- Deprecated old `Arc<Mutex<HashMap<String, Movie>>>` memory database
- Removed `api-demo.sh` shell script in favor of Python-based testing

---

## [0.1.0] — 2025-04-26

### Added
- Initial Axum-based movie API server
- Basic REST endpoints:
  - `POST /movie` — Add a movie
  - `GET /movie/{id}` — Fetch a movie by ID
- Shared in-memory database using `Arc<Mutex<HashMap<String, Movie>>>`
- Environment-configurable server bind address (`API_BIND_ADDR`)
- Friendly root (`/`) handler providing an API overview and welcome message
- `api-demo.sh` script to test API functionality with curl
- README.md with examples, endpoint descriptions, and usage notes

### Improved
- Introduced structured logging with `tracing` and `tracing-subscriber`
- Added `#[instrument]` macros to `add_movie` and `get_movie` handlers
- Cleaned up startup logging and endpoint behavior documentation
- Suppressed large db state in traces to avoid span pollution
- Improved error handling for add_movie (responds with `201 Created` on success)

### Changed
- Refined locking behavior comments for `tokio::sync::Mutex`
- Renamed `addr` to `endpoint` for server clarity
- Reduced `tokio` dependency features to only necessary components
