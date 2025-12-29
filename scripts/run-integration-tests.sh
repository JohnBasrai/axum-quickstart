#!/bin/bash

# run-integration-tests.sh
# Script to run integration tests with proper Docker setup
# Used by both developers and CI

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Starting integration test suite..."
echo "Project root: $PROJECT_ROOT"

exit_status=1

# Function to cleanup on exit
cleanup() {
    echo "🧹 Cleaning up... $exit_status"
    set -x
    cd "$PROJECT_ROOT"
    docker compose --ansi never down -v --remove-orphans
    exit $exit_status
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
docker compose --ansi never up -d redis postgres

# Wait for Redis to be ready
echo "⏳ Waiting for Redis to be ready..."
timeout=30
counter=0

while ! docker compose --ansi never exec redis redis-cli ping > /dev/null 2>&1; do
    if [ $counter -ge $timeout ]; then
        echo "❌ Redis failed to start within $timeout seconds"
        docker compose --ansi never logs redis
        exit 1
    fi
    echo "Waiting for Redis... ($counter/$timeout)"
    sleep 1
    counter=$((counter + 1))
done

echo "✅ Redis is ready!"

# Wait for PostgreSQL to be ready
echo "⏳ Waiting for PostgreSQL to be ready..."
counter=0

while ! docker compose --ansi never exec postgres pg_isready -U postgres > /dev/null 2>&1; do
    if [ $counter -ge $timeout ]; then
        echo "❌ PostgreSQL failed to start within $timeout seconds"
        docker compose --ansi never logs postgres
        exit 1
    fi
    echo "Waiting for PostgreSQL... ($counter/$timeout)"
    sleep 1
    counter=$((counter + 1))
done

echo "✅ PostgreSQL is ready!"

# Set database URL for migrations
export DATABASE_URL="postgresql://postgres:postgres@localhost:5432/axum_db"

# Run database migrations
echo "📦 Running database migrations..."
if command -v sqlx &> /dev/null; then
    sqlx migrate run
else
    echo "⚠️  sqlx-cli not installed. Install with: cargo install sqlx-cli --no-default-features --features postgres"
    echo "Attempting to continue without migrations (tests will fail if migrations are required)..."
fi

# Build the project
echo "🔨 Building project..."
cargo build --quiet

# Run integration tests
echo "🧪 Running integration tests..."

QUIET="--quiet"
QUIET=""
export RUST_LOG=info
export NO_COLOR=true

docker compose --ansi never ps
docker compose --ansi never exec postgres psql -U postgres -c "ALTER SYSTEM SET log_statement = 'all';"
docker compose --ansi never exec postgres psql -U postgres -c "ALTER SYSTEM SET log_connections = 'on';"
docker compose --ansi never exec postgres psql -U postgres -c "ALTER SYSTEM SET log_disconnections = 'on';"
docker compose --ansi never exec postgres psql -U postgres -c "SELECT pg_reload_conf();"

(docker compose --ansi never logs postgres --tail 50 --follow >& postgres.log&)

echo "------------------------------------------------"
echo "---------------- integration tests -------------"
echo "------------------------------------------------"
cargo test ${QUIET} --test integration -- --nocapture

echo "-----------------------------------------------------"
echo "---------------- webauthn_registration tests --------"
echo "-----------------------------------------------------"
cargo test ${QUIET} --test webauthn_registration -- --nocapture

echo "------------------------------------------------"
echo "---------------- metrics_endpoint tests --------"
echo "------------------------------------------------"
cargo test ${QUIET} --test metrics_endpoint -- --nocapture

echo "------------------------------------------------"
echo "---------------- webauthn_authentication tests --------"
echo "------------------------------------------------"
cargo test ${QUIET} --test webauthn_authentication -- --nocapture

echo "------------------------------------------------------"
echo "---------------- database::postgres_repository tests -"
echo "------------------------------------------------------"
cargo test --lib ${QUIET} -- infrastructure::database::postgres_repository --nocapture

echo "------------------------------------------------"
echo "---------------- database::tests ---------------"
echo "------------------------------------------------"
cargo test --lib ${QUIET} -- infrastructure::database::tests --nocapture

echo "------------------------------------------------"
echo "---------------- webauthn_credentials tests ----"
echo "------------------------------------------------"
cargo test ${QUIET} --test webauthn_credentials -- --nocapture

echo "✅ Integration tests completed successfully!"
exit_status=0
