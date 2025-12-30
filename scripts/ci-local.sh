#!/usr/bin/env bash
set -euo pipefail

# Disable GitHub Actions caches for local CI runs.
# To avoid masked state while debugging CI failures.
export CI_DISABLE_CACHE=1

act --rm
