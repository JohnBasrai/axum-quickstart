# Axum Quick-Start: Movie API

A simple, self-contained web API using the [Axum](https://crates.io/crates/axum) framework in Rust.

This project demonstrates:

- Defining data structures with `serde` for (de)serialization
- Routing with path parameters and JSON bodies using Axum
- Persistent Redis database backend with multiplexed async connections
- Clean async error handling via the `anyhow` crate
- Structured application logging using tracing and tracing-subscriber
- Minimal response payloads (e.g., POST /movies/add returns status code only)
- Read server bind address from `API_BIND_ADDR` environment variable
- Friendly root (`/`) handler with API overview
  - Uses an async closure with a raw string for readability
  - Lists available endpoints and their purpose
  - Improves first impression by avoiding a 404 on /

## Running the Server

By default, the server runs on `127.0.0.1:8080`. You can override this
by setting the `API_BIND_ADDR` environment variable:

```bash
# Run the server on a custom address/port
API_BIND_ADDR=0.0.0.0:8080 cargo run
```

Then visit: [http://localhost:8080](http://localhost:8080)

## Endpoints

---
### `POST /movies/add`

- Adds a new movie to the database.
- Expects a JSON body with fields: `id`, `title`, `year`, `stars`.

**Example:**
```bash
curl -X POST http://localhost:8080/movies/add   -H "Content-Type: application/json"   -d '{
        "id": "t1994Qw22",
        "title": "The Shawshank Redemption",
        "year": 1994,
        "stars": 4.5
      }'
```

---
### `GET /movies/get/{id}`

- Fetches the movie with the given `id`.
- Returns the movie data as JSON if found, or a 404 if not.

**Example:**
```bash
curl http://localhost:8080/movies/get/t1994Qw22
```

---
### `PUT /movies/update/{id}`

- Updates an existing movie by `id`.
- Expects a JSON body with new values for `title`, `year`, `stars`.

**Example:**
```bash
curl -X PUT http://localhost:8080/movies/update/t1994Qw22   -H "Content-Type: application/json"   -d '{
        "id": "t1994Qw22",
        "title": "The Shawshank Redemption (Director's Cut)",
        "year": 1994,
        "stars": 4.8
      }'
```

---
### `DELETE /movies/delete/{id}`

- Deletes a movie from the database by `id`.

**Example:**
```bash
curl -X DELETE http://localhost:8080/movies/delete/t1994Qw22
```

---
### `GET /health`

- Checks if the server (and optionally the Redis backend) are healthy.
- Optional query parameter `mode`:
  - `mode=light` (default) — basic server liveness check (server responding)
  - `mode=full` — ping Redis backend to ensure database connectivity

**Examples:**
```bash
# Basic liveness check
curl http://localhost:8080/health

# Full health check (including Redis)
curl http://localhost:8080/health?mode=full
```

---
## Testing the API

See [`api-test.py`](./api-test.py) for a detailed usage example.

This script:

- Adds a sample movie with a random ID
- Fetches the added movie
- Updates the movie
- Fetches it again to verify the update
- Deletes the movie
- Attempts to fetch it again (should return 404)

Run it with:

```bash
python3 api-test.py
```

## License

MIT
