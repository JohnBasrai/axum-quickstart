#!/bin/bash

# test-all.sh
# Complete test suite for development
# Runs the same tests as CI in the same order

set -e  # Exit on any error

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

echo "🎯 Running complete test suite (same as CI)..."

# Run unit tests first
echo ""
echo "=" * 50
echo "PHASE 1: Unit Tests & Linting"
echo "=" * 50
"$SCRIPT_DIR/run-unit-tests.sh"

# Run integration tests
echo ""
echo "=" * 50
echo "PHASE 2: Integration Tests"
echo "=" * 50
"$SCRIPT_DIR/run-integration-tests.sh"

echo ""
echo "🎉 All tests passed! Your code is ready for CI."
