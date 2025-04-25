# Axum Quick-Start: Movie API

A simple, self-contained web API using the [Axum](https://crates.io/crates/axum) framework in Rust.

This project demonstrates:

- Defining data structures with `serde` for (de)serialization
- Routing with path parameters and JSON bodies using Axum
- Shared, thread-safe in-memory state with `Arc<Mutex<HashMap<...>>>`
- Clean async error handling via the `anyhow` crate
- Read server bind address from `API_BIND_ADDR` environment variable
- Has friendly root (/) handler with API overview
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

### `GET /get/{id}`

- Gets the movie with the given `id`.
- Returns the movie data as JSON if found, or an empty object with a 404 status if not.

**Example:**
```bash
curl http://localhost:8080/get/t1994Qw22
```

### `POST /add`

Add a movie to the in-memory database.

**Quick test with curl:**
```bash
curl -X POST http://localhost:8080/add   -H "Content-Type: application/json"   -d '{
        "id": "t1994Qw22",
        "title": "The Shawshank Redemption",
        "year": 1994,
        "stars": 3.5
      }'
```

## For a more complete usage demo

Also check out the [`api-demo.sh`](./api-demo.sh) script for a more detailed usage example.

This script:

- Adds a sample movie with a unique ID
- Fetches the same movie using its ID
- Attempts to fetch a non-existent movie (to demonstrate 404 handling)

Run it with:

```bash
./api-demo.sh
./api-demo.sh --verbose
```

## License

MIT
