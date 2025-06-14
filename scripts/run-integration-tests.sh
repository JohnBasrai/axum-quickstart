#!/bin/bash

# run-integration-tests.sh
# Script to run integration tests with proper Docker setup
# Used by both developers and CI

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Starting integration test suite..."
echo "Project root: $PROJECT_ROOT"

# Function to cleanup on exit
cleanup() {
    echo "🧹 Cleaning up..."
    cd "$PROJECT_ROOT"
    docker compose down -v --remove-orphans || true
}

# Set trap to cleanup on script exit
trap cleanup EXIT

# Change to project root
cd "$PROJECT_ROOT"

# Check if docker compose is available
if ! command -v docker compose &> /dev/null; then
    echo "❌ docker compose is not installed. Please install Docker Compose."
    exit 1
fi

# Start services
echo "🐳 Starting Docker services..."
docker compose up -d redis

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
timeout=30
counter=0

while ! docker compose exec redis redis-cli ping > /dev/null 2>&1; do
    if [ $counter -ge $timeout ]; then
        echo "❌ Redis failed to start within $timeout seconds"
        docker compose logs redis
        exit 1
    fi
    echo "Waiting for Redis... ($counter/$timeout)"
    sleep 1
    counter=$((counter + 1))
done

echo "✅ Redis is ready!"

# Build the project
echo "🔨 Building project..."
cargo build

# Run integration tests
echo "🧪 Running integration tests..."
RUST_LOG=debug cargo test -- --test-threads 1 --test integration      -- --nocapture
RUST_LOG=debug cargo test -- --test-threads 1 --test metrics_endpoint -- --nocapture

echo "✅ Integration tests completed successfully!"
