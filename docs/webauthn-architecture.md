# WebAuthn/Passkey Authentication

## Overview

This implementation demonstrates modern passwordless authentication integrated into an existing REST API, showcasing security expertise, clean architecture principles, and Rust proficiency.

**What it demonstrates:**
- Modern security patterns (WebAuthn/Passkeys)
- Clean Architecture with Dependency Inversion
- Repository pattern with trait-based abstraction
- PostgreSQL integration for cryptographic data
- Multi-phase project planning and execution

## Why WebAuthn?

WebAuthn enables passwordless authentication using biometrics (Touch ID, Face ID), hardware keys (YubiKey), or platform authenticators (Windows Hello). It's phishing-resistant, uses public/private key cryptography, and eliminates password-related security risks.

## Architecture

### Clean Architecture with Dependency Inversion Principle

```
┌─────────────────────────────────────────┐
│         Application Layer               │
│      (Handlers, Routes, AppState)       │
└────────────┬────────────────────────────┘
             │ depends on
             ↓
┌─────────────────────────────────────────┐
│    Domain Layer (Abstraction/Policy)    │
│       <<interface>> Repository          │
│         User, Credential                │
└────────────△────────────────────────────┘
             │ implements
             │
┌────────────┴────────────────────────────┐
│  Infrastructure Layer (Mechanism)       │
│      PostgresRepository (concrete)      │
│            (sqlx)                       │
└─────────────────────────────────────────┘
```

**Key Principle:** Both high-level (Application) and low-level (Infrastructure) layers depend on the abstraction (Repository trait in Domain). The Domain layer has zero dependencies - it defines the contract that both other layers depend on. This is the [Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle).

**Benefits:**
- Testability: Easy to mock Repository for unit tests
- Flexibility: Can swap PostgreSQL for MySQL, SQLite, or in-memory storage
- Clean boundaries: Domain logic has no database dependencies
- Consistency: Mirrors existing Metrics trait pattern in the codebase

### EMBP (Explicit Module Boundary Pattern)

Module gateways (`mod.rs`) control public API. Domain defines interfaces (Repository trait), infrastructure implements them (PostgresRepository). Clear dependency direction: Infrastructure → Domain (never reverse).

## Security Highlights

### Replay Attack Prevention

WebAuthn uses signature counters to prevent replay attacks:
1. Counter starts at 0 on registration
2. Authenticator increments counter with each authentication
3. Server verifies counter > stored value
4. If counter ≤ stored value, authentication is rejected (replay attack detected)

This cryptographic counter is stored in PostgreSQL and updated atomically during authentication.

### Why PostgreSQL (Not Redis)?

**PostgreSQL provides:**
- Permanent storage (can't lose user credentials on restart)
- Foreign key constraints (credentials must belong to valid users)
- ACID guarantees for atomic counter updates
- Efficient relational queries ("all credentials for user X")
- Binary data storage (BYTEA) for cryptographic keys

**Redis is still used for:**
- WebAuthn challenges (5-minute TTL)
- Session tokens (existing functionality)
- Caching (existing functionality)

Credentials are cryptographic assets requiring relational integrity - PostgreSQL is the right tool.

## Implementation Phases

### Phase 1: Database Infrastructure ✅ COMPLETE
Database layer for credential storage with Repository pattern.

**Delivered:** User/Credential models, Repository trait, PostgresRepository implementation, SQLx migrations, integration tests, CI/CD updates.

### Phase 2: Registration Flow ✅ COMPLETE
API endpoints for registering passkeys using webauthn-rs crate.

**Delivered:** Registration challenge generation, credential storage, webauthn-rs integration, challenge storage in Redis.

**Endpoints:** `POST /webauthn/register/start`, `POST /webauthn/register/finish`

### Phase 3: Authentication Flow ✅ COMPLETE
Login with passkeys, session creation, counter validation.

**Delivered:** Authentication challenge generation, credential verification, counter-based replay attack prevention, session token creation.

**Endpoints:** `POST /webauthn/auth/start`, `POST /webauthn/auth/finish`

### Phase 4: Credential Management ✅ COMPLETE
Users can view and delete their registered passkeys.

**Delivered:** Session validation, credential listing, credential deletion with ownership verification, protected endpoints.

**Endpoints:** `GET /webauthn/credentials`, `DELETE /webauthn/credentials/:id`

### Phase 5: Testing & Documentation (Planned)
Browser-based test page, end-to-end tests, production-ready documentation.

## Technology Stack

### Phase 1 Additions
- `sqlx 0.7` - Async PostgreSQL driver with compile-time query checking
- `uuid` - Cryptographically secure user ID generation
- `async-trait` - Trait support for async methods
- `chrono[serde]` - Timestamp serialization

### Phase 2+ (Planned)
- `webauthn-rs` - WebAuthn protocol implementation
- `base64` - Credential encoding/decoding

## Database Schema

Migrations define the schema (see `migrations/` directory):
- **users table:** UUID primary key, unique username, timestamps
- **credentials table:** Binary credential ID, foreign key to users, public key (binary), signature counter, timestamps

Foreign key constraints ensure credentials always belong to valid users. Indexes optimize username and user_id lookups.

## Code Organization

```
src/
├── domain/
│   ├── webauthn_models.rs    # User, Credential structs
│   ├── repository.rs          # Repository trait + RepositoryPtr
│   └── mod.rs                 # Gateway exports
├── infrastructure/
│   ├── database/
│   │   ├── postgres_repository.rs  # PostgreSQL implementation
│   │   └── mod.rs                  # Database gateway
│   └── mod.rs                      # Infrastructure gateway
└── migrations/
    ├── 20250101000001_create_users_table.sql
    └── 20250101000002_create_credentials_table.sql
```

Follows existing codebase patterns (Metrics trait) and EMBP module organization.

## Why This Project Exists

WebAuthn/Passkeys represent the future of authentication - phishing-resistant, no password fatigue, strong cryptographic guarantees. This implementation integrates passwordless auth into an existing REST API to show how modern authentication can be retrofitted into production systems.

The architecture follows dependency inversion principles to keep domain logic database-agnostic while using PostgreSQL's ACID guarantees for credential storage integrity.

## References

- [WebAuthn Specification](https://www.w3.org/TR/webauthn-2/)
- [Dependency Inversion Principle](https://en.wikipedia.org/wiki/Dependency_inversion_principle)
- [Clean Architecture](https://blog.cleancoder.com/uncle-bob/2012/08/13/the-clean-architecture.html)
- [webauthn-rs Documentation](https://docs.rs/webauthn-rs/)

---

**Status:** Phase 4 Complete | **Next:** Testing & Documentation (Phase 5)  
**Updated:** December 2024

