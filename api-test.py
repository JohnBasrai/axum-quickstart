#!/usr/bin/env python3

"""
api-test.py - Integration tests for Movie API

This script will:

| Step                      | HTTP Verb | Endpoint                     |
|---------------------------|-----------|------------------------------|
| Add movie                 | POST      | /movies/add                  |
| Fetch movie               | GET       | /movies/get/{id}             |
| Update movie              | PUT       | /movies/update/{id}          |
| Fetch updated movie       | GET       | /movies/get/{id}             |
| Delete movie              | DELETE    | /movies/delete/{id}          |
| Fetch deleted movie       | GET       | /movies/get/{id} (expect 404)|
| Fetch nonexistent movie   | GET       | /movies/get/nonexistent123 (expect 404) |
| Health check (no params)  | GET       | /health                      |
| Health check (mode=full)  | GET       | /health?mode=full            |
| Health check (mode=light) | GET       | /health?mode=light           |
 
Notes:
- Verifies successful adds, updates, deletions.
- Confirms 404 behavior for missing or deleted entries.
- Exits early with ❌ if any test fails.
- Pretty prints fetched movie data.
"""

import requests
import sys
import json
import uuid

BASE_URL = "http://localhost:8080"

def pretty_print(response):
    print(json.dumps(response.json(), indent=2))

def assert_status(response, expected_status, description=""):
    if response.status_code != expected_status:
        print(f"❌ FAIL: {description}: Expected {expected_status}, got {response.status_code}")
        print(f"Response body: {response.text}")
        pretty_print(response)
        sys.exit(1)
    else:
        print(f"✅ PASS: {description}")

verbose = False

def test_health_checks():
    # 1. Light check (no params)
    print("\n -- Checking health endpoint (light, no params)...")
    response = requests.get(f"{BASE_URL}/health")
    assert_status(response, 200, "Health check (no params)")
    if verbose: pretty_print(response)

    # 2. Full check (ping Redis)
    print("\n -- Checking health endpoint (mode=full)...")
    response = requests.get(f"{BASE_URL}/health?mode=full")
    assert_status(response, 200, "Health check (mode=full)")
    if verbose: pretty_print(response)

    # 3. Explicit light check
    print("\n -- Checking health endpoint (mode=light)...")
    response = requests.get(f"{BASE_URL}/health?mode=light")
    assert_status(response, 200, "Health check (mode=light)")
    if verbose: pretty_print(response)


def main():
    movie_id = f"t{uuid.uuid4().hex[:8]}"  # generate a random ID

    movie_payload = {
        "id": movie_id,
        "title": "The Shawshank Redemption",
        "year": 1994,
        "stars": 4.5
    }

    # 1. Add movie
    print(f"\n -- Adding movie ID {movie_id}...")
    response = requests.post(f"{BASE_URL}/movies/add", json=movie_payload)
    assert_status(response, 201, "Add movie (POST /movies/add)")

    # 2. Fetch movie
    print("\n -- Fetching movie...")
    response = requests.get(f"{BASE_URL}/movies/get/" + movie_id)
    assert_status(response, 200, "Fetch movie (GET /movies/get/{id})")

    # 3. Update movie
    updated_payload = {
        "id": movie_id,
        "title": "The Shawshank Redemption (Director's Cut)",
        "year": 1994,
        "stars": 4.8
    }
    print("\n -- Updating movie...")
    response = requests.put(f"{BASE_URL}/movies/update/" + movie_id, json=updated_payload)
    assert_status(response, 200, "Update movie (PUT /movies/update/{id})")

    # 4. Fetch updated movie
    print("\n -- Fetching updated movie...")
    response = requests.get(f"{BASE_URL}/movies/get/" + movie_id)
    assert_status(response, 200, "Fetch updated movie (GET /movies/get/{id})")

    # 5. Delete movie
    print("\n -- Deleting movie...")
    response = requests.delete(f"{BASE_URL}/movies/delete/" + movie_id)
    assert_status(response, 204, "Delete movie (DELETE /movies/delete/{id})")

    # 6. Fetch deleted movie (should be 404)
    print("\n -- Fetching deleted movie (should be 404)...")
    response = requests.get(f"{BASE_URL}/movies/get/" + movie_id)
    assert_status(response, 404, "Fetch deleted movie (GET /movies/get/{id})")

    # 7. Fetch a truly non-existent movie
    fake_id = "nonexistent123"
    print("\n -- Fetching non-existent movie...")
    response = requests.get(f"{BASE_URL}/movies/get/" + fake_id)
    assert_status(response, 404, "Fetch non-existent movie (GET /movies/get/{id})")

if __name__ == "__main__":
    test_health_checks()
    main()
    print("\n🎉 All tests passed successfully!")
