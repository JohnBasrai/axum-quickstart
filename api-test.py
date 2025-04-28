#!/usr/bin/env python3

"""
api-test.py - Integration tests for Movie API

This script will:

| Step                      | HTTP Verb | Endpoint                        |
|---------------------------|-----------|---------------------------------|
| Add movie                 | POST      | /movies/add                     |
| Fetch movie               | GET       | /movies/get/{id}                |
| Update movie              | PUT       | /movies/update/{id}             |
| Fetch updated movie       | GET       | /movies/get/{id}                |
| Delete movie              | DELETE    | /movies/delete/{id}             |
| Fetch deleted movie       | GET       | /movies/get/{id} (expect 404)   |
| Fetch nonexistent movie   | GET       | /movies/get/nonexistent123 (404)|
| Health check (no params)  | GET       | /health                         |
| Health check (mode=full)  | GET       | /health?mode=full               |
| Health check (mode=light) | GET       | /health?mode=light              |

Notes:
- The server generates the Movie ID. It is captured from the POST /movies/add response.
- Verifies successful adds, updates, deletions.
- Confirms 404 behavior for missing or deleted entries.
- Exits early with ❌ if any test fails.
- Pretty prints fetched movie data.
"""

import json
import requests
import subprocess
import sys
import time

BASE_URL = "http://localhost:8080"


def restart_redis_container():
    print("\n -- Recreating Redis container (redis-test)...")
    subprocess.run(["docker", "rm", "-f", "redis-test"],
                   stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    result = subprocess.run([
        "docker", "run", "--name", "redis-test", "-p", "6379:6379", "-d", "redis:7"
    ], capture_output=True, text=True)
    if result.returncode != 0:
        print(f"❌ Failed to restart Redis container:\n{result.stderr}")
        sys.exit(1)
    else:
        print(f"✅ Redis container recreated: {result.stdout.strip()}")
    time.sleep(2)
    subprocess.run(["docker", "ps"], stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)

def pretty_print(response):
    try:
        print(json.dumps(response.json(), indent=2))
    except Exception:
        print(response.text)

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
    print("\n -- Checking health endpoint (light, no params)...")
    response = requests.get(f"{BASE_URL}/health")
    assert_status(response, 200, "Health check (no params)")
    if verbose: pretty_print(response)

    print("\n -- Checking health endpoint (mode=full)...")
    response = requests.get(f"{BASE_URL}/health?mode=full")
    assert_status(response, 200, "Health check (mode=full)")
    if verbose: pretty_print(response)

    print("\n -- Checking health endpoint (mode=light)...")
    response = requests.get(f"{BASE_URL}/health?mode=light")
    assert_status(response, 200, "Health check (mode=light)")
    if verbose: pretty_print(response)

def main():
    # 1. Create a movie (no id in payload)
    movie_payload = {
        "title": "The Shawshank Redemption",
        "year": 1994,
        "stars": 4.5
    }

    print("\n -- Adding new movie...")
    response = requests.post(f"{BASE_URL}/movies/add", json=movie_payload)
    assert_status(response, 201, "Add movie (POST /movies/add)")

    # Extract the returned movie ID
    response_data = response.json()
    movie_id = response_data["id"]
    print(f"Movie created with ID: {movie_id}")

    # 1b. Try adding the same movie again (should fail with 409 Conflict)
    print("\n -- Adding duplicate movie (should be 409 Conflict)...")
    response = requests.post(f"{BASE_URL}/movies/add", json=movie_payload)
    assert_status(response, 409, "Add duplicate movie (POST /movies/add)")

    # Continue using the original movie_id for fetch, update, delete

    # 2. Fetch movie
    print("\n -- Fetching movie...")
    response = requests.get(f"{BASE_URL}/movies/get/" + movie_id)
    assert_status(response, 200, "Fetch movie (GET /movies/get/{id})")

    # 3. Update movie
    updated_payload = {
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

    # 6. Fetch deleted movie (should 404)
    print("\n -- Fetching deleted movie (should be 404)...")
    response = requests.get(f"{BASE_URL}/movies/get/" + movie_id)
    assert_status(response, 404, "Fetch deleted movie (GET /movies/get/{id})")

    # 7. Fetch a truly nonexistent movie
    fake_id = "nonexistent123"
    print("\n -- Fetching non-existent movie...")
    response = requests.get(f"{BASE_URL}/movies/get/" + fake_id)
    assert_status(response, 404, "Fetch non-existent movie (GET /movies/get/{id})")

if __name__ == "__main__":
    restart_redis_container()
    test_health_checks()
    main()
    print("\n🎉 All tests passed successfully!")
