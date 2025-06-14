#!/bin/bash

# dev-setup.sh
# Development environment setup script

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🛠️  Setting up development environment..."

# Change to project root
cd "$PROJECT_ROOT"

# Make all scripts executable
echo "📝 Making scripts executable..."
chmod +x scripts/*.sh

# Check prerequisites
echo "🔍 Checking prerequisites..."

# Check Rust
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust from https://rustup.rs/"
    exit 1
fi

# Check Docker
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker."
    exit 1
fi

# Check Docker Compose
if ! command -v docker compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose."
    exit 1
fi

echo "✅ All prerequisites are installed!"

# Install Rust components
echo "🦀 Installing Rust components..."
rustup component add rustfmt clippy

# Start Docker services for development
echo "🐳 Starting development services..."
docker compose up -d redis

echo "✅ Development environment is ready!"
echo ""
echo "Available commands:"
echo "  ./scripts/run-unit-tests.sh      - Run unit tests and linting"
echo "  ./scripts/run-integration-tests.sh - Run integration tests"
echo "  ./scripts/test-all.sh            - Run complete test suite (like CI)"
echo "  docker compose up -d redis       - Start Redis"
echo "  docker compose down              - Stop all services"
echo "  docker compose logs redis        - View Redis logs"
echo ""
echo "To run with Redis Commander (GUI): docker compose --profile debug up -d"
