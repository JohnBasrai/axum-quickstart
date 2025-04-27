# Changelog

## [1.1.0] - 2025-04-27

### Added
- Add `GET /health` endpoint for server and Redis liveness checks
- Support optional `mode=light` (default) or `mode=full` (Redis ping) via query parameter
- Extend `api-test.py` to validate health checks (no params, mode=full, mode=light)
- Added graceful shutdown support for `Ctrl+C` and `SIGTERM` signals.

### Changed
- Modularize handlers: split movies, health, and shared_types into separate modules
- Improve server startup logging: include version and bind address at launch

## [1.0.1] - 2025-04-27

### Changed
- Update README.md to reflect `/movies` API namespace restructure
- Document all CRUD operations (POST, GET, PUT, DELETE) under `/movies`
- Update usage instructions to refer to `api-test.py` instead of `api-demo.sh`

## [1.0.0] - 2025-04-27

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

## [0.1.0] - 2025-04-26

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
