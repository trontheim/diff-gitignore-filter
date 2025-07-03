#!/bin/bash
# Post-bump hook
# Handles changelog generation and git operations after version bump

set -e

CURRENT_VERSION=$1
NEW_VERSION=$2
RELEASE_LEVEL=$3

echo "🤖 Generating changelog..."
git cliff --topo-order --output CHANGELOG.md --tag "$NEW_VERSION"
