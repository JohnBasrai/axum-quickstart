#!/usr/bin/env python3

import requests
import sys
import json
import uuid

BASE_URL = "http://localhost:8080"

def assert_status(response, expected_status, description=""):
    if response.status_code != expected_status:
        print(f"❌ FAIL: {description}: Expected {expected_status}, got {response.status_code}")
        print(f"Response body: {response.text}")
        sys.exit(1)
    else:
        print(f"✅ PASS: {description}")

def pretty_print(response):
    print(json.dumps(response.json(), indent=2))

def main():
    movie_id = f"t{uuid.uuid4().hex[:8]}"  # generate a random ID

    movie_payload = {
        "id": movie_id,
        "title": "The Shawshank Redemption",
        "year": 1994,
        "stars": 4.5
    }

    # 1. Add movie
    print(f"🔵 Adding movie ID {movie_id}...")
    response = requests.post(f"{BASE_URL}/add", json=movie_payload)
    assert_status(response, 201, "Add movie (POST /add)")

    # 2. Fetch movie
    print("🔵 Fetching movie...")
    response = requests.get(f"{BASE_URL}/get/{movie_id}")
    assert_status(response, 200, "Fetch movie (GET /get/{id})")
    pretty_print(response)

    # 3. Update movie
    updated_payload = {
        "id": movie_id,
        "title": "The Shawshank Redemption (Director's Cut)",
        "year": 1994,
        "stars": 4.8
    }
    print("🔵 Updating movie...")
    response = requests.put(f"{BASE_URL}/update/{movie_id}", json=updated_payload)
    assert_status(response, 200, "Update movie (PUT /update/{id})")

    # 4. Fetch updated movie
    print("🔵 Fetching updated movie...")
    response = requests.get(f"{BASE_URL}/get/{movie_id}")
    assert_status(response, 200, "Fetch updated movie (GET /get/{id})")
    pretty_print(response)

    # 5. Fetch a non-existent movie
    fake_id = "nonexistent123"
    print("🔵 Fetching non-existent movie...")
    response = requests.get(f"{BASE_URL}/get/{fake_id}")
    assert_status(response, 404, "Fetch non-existent movie (GET /get/{id})")

    print("\n🎉 All tests passed successfully!")

if __name__ == "__main__":
    main()
