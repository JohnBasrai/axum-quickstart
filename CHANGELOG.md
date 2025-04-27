# Changelog

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
