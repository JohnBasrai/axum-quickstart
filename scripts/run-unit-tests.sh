#!/bin/bash

# run-unit-tests.sh
# Script to run unit tests, linting, and formatting checks
# Used by both developers and CI

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo "🚀 Starting unit test suite..."
echo "Project root: $PROJECT_ROOT"

# Change to project root
cd "$PROJECT_ROOT"

# Run clippy
echo "📎 Running Clippy (linter)..."
cargo clippy --all-targets --all-features --no-deps -- -D warnings

# Check formatting
echo "🎨 Checking code formatting..."
if [ -f .rustfmt-ignore ]; then
    echo "Found .rustfmt-ignore - skipping format check"
    echo "Manual formatting control enabled"
else
    cargo fmt --check
fi

# Build the project
echo "🔨 Building project..."
cargo build --quiet

# Run unit tests
echo "🧪 Running unit tests..."
cargo test --lib -- --skip infrastructure::database

echo "✅ Unit tests completed successfully!"
