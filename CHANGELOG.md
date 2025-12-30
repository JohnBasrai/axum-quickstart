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

## [1.0.0] - 2025-12-27

### Added

**Phase 4: Protected Routes & Documentation**
- JWT authentication middleware for protected endpoints
- Demo protected route (`GET /protected`) with Bearer token validation
- Automated tests for protected route (Tests 8-10):
  - Valid token acceptance (200)
  - Missing token rejection (401)
  - Revoked token rejection (401)
- Comprehensive README for oauth2-client with OAuth2 flow documentation
- Comprehensive README for oauth2-server with database schema and API reference
- Enhanced main README with "Testing & Quality Assurance" section
- Documentation highlighting 10-test automated suite

**Phase 3: JWT Service**
- Complete JWT token service implementation (port 8083)
- Token generation endpoint with HS256 signing (`POST /auth/token`)
- Token validation endpoint with signature verification (`POST /auth/validate`)
- Refresh token flow with rotation security (`POST /auth/refresh`)
- Token revocation with Redis blacklist (`POST /auth/revoke`)
- 7 automated integration tests for JWT flows
- JWT primer documentation (theory and concepts)
- Development setup guide
- CONTRIBUTING.md with code quality standards

**Phase 2: OAuth2 Server**
- Complete OAuth2 authorization server implementation (port 8082)
- Authorization endpoint with consent page (`GET/POST /oauth/authorize`)
- Token exchange endpoint (`POST /oauth/token`)
- User info endpoint (`GET /oauth/userinfo`)
- PostgreSQL database schema (clients, users, authorization codes, access tokens)
- Database migrations with seed data (demo user and client)
- Argon2 password hashing for demo users
- Manual request body deserialization for better debugging

**Phase 1: OAuth2 Client**
- OAuth2 client implementation with authorization code flow (port 8081)
- HTTP handlers: home, login, callback, profile
- Integration with oauth2-server for end-to-end flow

**Infrastructure & Quality**
- Docker Compose setup (PostgreSQL + Redis)
- Development scripts: `startup.sh`, `shutdown.sh`, `test-all.sh`, `test-jwt-service.sh`
- GitHub Actions CI/CD pipeline
- EMBP (Explicit Module Boundary Pattern) architecture
- Request tracing with configurable ANSI output
- Environment-based configuration (`.env` support)
- Comprehensive rustdoc with RFC references (RFC 6749, RFC 7519)

### Changed
- OAuth2 client uses RequestBody auth type for token exchange
- Enhanced logging for debugging OAuth2 token exchange
- Main README reorganized with prominent testing section
- Claims struct includes Clone derive for Axum 0.8 compatibility

### Security
- Token revocation with Redis-backed blacklist
- Refresh token rotation (prevents replay attacks)
- JWT signature validation (HS256)
- Token expiration checking (15 min for access, 7 days for refresh)
- Protected route middleware validates signature, expiration, and revocation
- Argon2id password hashing for demo users
- CSRF protection via state parameter in OAuth2 flow

## [0.1.0] - 2025-12-26

### Added
- Initial project setup
