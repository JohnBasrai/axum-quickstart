# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- None

### Changed
- None

### Fixed
- None

## [1.4.0] - 2024-12-30

### Added

**Phase 4: WebAuthn Credential Management**
- `GET /webauthn/credentials` endpoint for listing user's registered passkeys
- `DELETE /webauthn/credentials/:id` endpoint for removing passkeys
- Session-based authentication with Bearer token validation
- Ownership verification to prevent unauthorized credential deletion
- 7 integration tests for session validation and credential CRUD operations
- Session helper functions (`create_session`, `validate_session`)

**Infrastructure & Testing Improvements**
- Local CI runner (`scripts/ci-local.sh`) using `act` for debugging CI failures
- Doctests now run in unit test suite to match GitHub Actions
- Fixed database hostname for GitHub Actions (postgres → localhost)
- Serialized metrics tests with `#[serial]` to prevent Prometheus registry races
- Prevented duplicate CI workflow runs on PR branches
- Standardized database name to `axum_db` across all environments
- Bumped Rust toolchain to 1.92.0 for consistency
- GitHub Actions cache optimization (773 MB target cache, ~56% build time reduction)

**Documentation**
- Comprehensive WebAuthn architecture documentation
- SQLx offline mode how-to guide
- Updated CONTRIBUTING.md with testing guidelines

### Changed
- Database name standardized to `axum_db` (from `axum_quickstart_test`)
- GitHub Actions workflow improvements for better CI/CD parity with local testing
- Enhanced integration test scripts with better error handling

## [1.3.3] - 2024-12-28

### Added

**Phase 3: WebAuthn Authentication Flow**
- `POST /webauthn/auth/start` endpoint for authentication challenge generation
- `POST /webauthn/auth/finish` endpoint for credential verification and session creation
- Counter-based replay attack prevention with atomic database updates
- Session token creation with 7-day TTL in Redis
- Challenge storage in Redis with 5-minute expiry
- 6 integration tests for authentication flow (3 additional tests ignored pending Issue #33)
- Session management tests (creation, TTL validation)
- Counter increment and replay detection tests
- Redis challenge storage tests (atomic GETDEL, expiry validation)

**Documentation**
- WebAuthn architecture documentation with security highlights
- Phase 3 completion markers in architecture docs

### Fixed
- WebAuthn verifier injection limitations documented (Issue #33)

## [1.3.2] - 2024-12-27

### Fixed
- Removed OpenSSL dependency by disabling reqwest default features (#29)
- Resolved build failures on systems without OpenSSL development headers

### Changed
- Updated reqwest configuration to use rustls instead of native-tls

## [1.3.1] - 2024-12-27

### Added

**Phase 2: WebAuthn Registration Flow**
- `POST /webauthn/register/start` endpoint for registration challenge generation
- `POST /webauthn/register/finish` endpoint for credential storage
- webauthn-rs integration for protocol implementation
- Challenge storage in Redis with automatic expiry
- User creation during registration (username-based)
- 6 integration tests for registration flow (2 additional tests ignored pending Issue #33)
- Comprehensive error handling for registration edge cases

**Documentation**
- Phase 2 completion documentation
- Registration flow architecture details

### Changed
- Enhanced error responses for registration endpoints
- Improved test coverage for WebAuthn flows

## [1.3.0] - 2024-12-26

### Added

**Phase 1: WebAuthn Database Infrastructure**
- Domain models (`User`, `Credential`) for WebAuthn entities
- `Repository` trait defining data access contract (Clean Architecture)
- `PostgresRepository` implementation with SQLx
- Database migrations for `users` and `credentials` tables
- Foreign key constraints ensuring data integrity (credentials → users)
- 9 unit tests for repository operations
- 1 schema test for cascade deletion
- UUID-based user identification
- Binary credential ID storage (BYTEA)
- Cryptographic counter storage for replay attack prevention

**Infrastructure**
- PostgreSQL 16 integration with Docker Compose
- SQLx compile-time query verification
- Database initialization with connection pooling
- Comprehensive integration test setup

**Documentation**
- WebAuthn architecture overview
- Clean Architecture principles documentation
- Repository pattern explanation
- Database schema documentation

### Changed
- Upgraded project structure to support WebAuthn features
- Enhanced CI/CD pipeline for database testing

---

**Previous versions (pre-WebAuthn) are not documented in this changelog**
