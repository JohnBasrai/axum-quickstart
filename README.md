# Axum Quick-Start: Movie API

A simple, self-contained web API using the [Axum](https://crates.io/crates/axum) framework in Rust.

This project demonstrates:

- Defining data structures with `serde` for (de)serialization
- Routing with path parameters and JSON bodies using Axum
- Persistent Redis database backend with multiplexed async connections
- Clean async error handling via the `anyhow` crate
- Structured application logging using tracing and tracing-subscriber
- Minimal and structured response payloads (`POST /movies/add` returns `201 Created` with generated ID in JSON)
- Read server bind address from `API_BIND_ADDR` environment variable
- Friendly root (`/`) handler with API overview
  - Lists available endpoints and their purpose
  - Improves first impression by avoiding a 404 on /

## Running the Server

By default, the server runs on `127.0.0.1:8080`. You can override this
by setting the `API_BIND_ADDR` environment variable:

```bash
API_BIND_ADDR=0.0.0.0:8080 cargo run
```

Then visit: [http://localhost:8080](http://localhost:8080)

---

## Quick Example

**Add a Movie:**

```bash
curl -X POST http://localhost:8080/movies/add \
     -H "Content-Type: application/json" \
     -d '{ "title": "The Shawshank Redemption", "year": 1994, "stars": 4.5 }'
```

The server will generate an ID based on the title and year, and return it in the response body:
```json
{ "id": "<generated_id>" }
```

If a movie with the same title and year already exists, the server will return `409 Conflict`.

---

## Full API Usage

For detailed examples covering:

- Adding a movie
- Fetching a movie
- Updating a movie
- Deleting a movie
- Handling duplicate movies
- Health checks

see [`api-test.py`](./api-test.py).

---

## License

MIT
