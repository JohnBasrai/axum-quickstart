#!/bin/bash

# test-all.sh
# Complete test suite for development
# Runs the same tests as CI in the same order

set -xe
set -o pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

: "🎯 Running complete test suite (same as CI)..."

# Run unit tests first
: "--------------------------------------------------------------------------------"
: "PHASE 1: Unit Tests & Linting"
: "--------------------------------------------------------------------------------"
"${SCRIPT_DIR}"/run-unit-tests.sh
: "run-unit-tests.sh status: OK"

# Run integration tests
: "--------------------------------------------------------------------------------"
: "PHASE 2: Integration Tests"
: "--------------------------------------------------------------------------------"
"${SCRIPT_DIR}"/run-integration-tests.sh
: "run-integration-tests.sh OK"
: "  🎉 All tests passed! Your code is ready for CI."
