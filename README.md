# Axum Quickstart Project

[![Rust CI](https://github.com/JohnBasrai/axum-quickstart/actions/workflows/rust.yml/badge.svg)](https://github.com/JohnBasrai/axum-quickstart/actions)
[![Latest Release](https://img.shields.io/github/v/release/JohnBasrai/axum-quickstart?style=flat-square)](https://github.com/JohnBasrai/axum-quickstart/releases)

## Overview

This project is a quickstart template for building a **production-ready RESTful API** in Rust using Axum and Tokio.
It includes:

 - Full CRUD endpoints for managing movies
 - HTML root (`/`) page with dynamic version display
 - Redis-backed storage for persistence
 - Comprehensive HTTP integration tests using reqwest
 - Clean project architecture with modular separation of concerns

---

## Running the Application

Make sure you have a Redis server running locally.

You can quickly start Redis using Docker:

    docker run --name test-redis -p 6379:6379 -d redis

Then start the server:

    cargo run

The application will be available at http://localhost:8080.
Opening the root URL (`/`) in a browser will display a styled HTML status page with 
the current application version (auto-extracted from `Cargo.toml` at build time).

---

## Running Tests

Integration tests are written in Rust and use real HTTP requests via reqwest.

To run the tests:

1. **Start a Redis server locally**

   If not already running:

       docker run --name test-redis -p 6379:6379 -d redis

2. **Run all tests**

       cargo test

This will run:

- Unit tests inside `src/`
- Full HTTP-based integration tests inside `tests/`

Each integration test spins up its own isolated Axum server instance and binds to a random port, allowing tests to run concurrently without conflict.

> Note: The old Python-based scripts/api-test.py is deprecated and no longer maintained. All testing is now handled natively in Rust.

---

## Project Structure

    src/
      lib.rs         # Application setup (create_app)
      main.rs        # Server startup entry point
      handlers/      # Route handlers (movies, health, etc.)
    tests/
      integration.rs # Full end-to-end integration tests
---

## License

This project is licensed under the MIT License.
