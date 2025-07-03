#!/bin/bash
# Post-release hook
# Handles development version setup and final cleanup after release

set -e

CURRENT_VERSION=$1
NEW_VERSION=$2
RELEASE_LEVEL=$3

# Source required libraries
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
LIB_DIR="$SCRIPT_DIR/../lib"

# shellcheck source=../lib/common.sh
source "$LIB_DIR/common.sh"
# shellcheck source=../lib/version-utils.sh
source "$LIB_DIR/version-utils.sh"

echo "🤖 Setting intelligent next development version with built-in version bump..."

# Parameter normalisieren (v-Prefix entfernen falls vorhanden)
CURRENT_VERSION=$(echo "$CURRENT_VERSION" | sed 's/^v//')
NEW_VERSION=$(echo "$NEW_VERSION" | sed 's/^v//')

echo "Previous version: ${CURRENT_VERSION}"
echo "Current released version: ${NEW_VERSION}"
echo "Release level: ${RELEASE_LEVEL}"

# Pre-Release Detection - override RELEASE_LEVEL if pre-release detected
if [[ "$NEW_VERSION" =~ -[a-zA-Z] ]]; then
    RELEASE_LEVEL="prerelease"
fi

# Built-in version bump für intelligente nächste Dev-Version verwenden
case "$RELEASE_LEVEL" in
    "patch")
        # Patch Release: nächste Patch-Version mit -dev
        echo "Bumping to next patch development version..."
        bump_version patch dev
        ;;
    "minor")
        # Minor Release: nächste Minor-Version mit -dev
        echo "Bumping to next minor development version..."
        bump_version minor dev
        ;;
    "major")
        # Major Release: nächste Minor-Version mit -dev (nicht Major)
        echo "Bumping to next patch development version after major release..."
        bump_version patch dev
        ;;
    "prerelease")
        # Pre-Release: bleibe bei aktueller Version mit -dev
        echo "Bumping to current development version after pre-release..."
        bump_version "" dev
        ;;
    *)
        echo "❌ Unknown release level: ${RELEASE_LEVEL}"
        echo "Expected: patch, minor, major, or prerelease"
        exit 1
        ;;
esac

# Validierung der neuen Version
echo "Validating updated Cargo.toml..."
cargo check --quiet

# Neue Version aus Cargo.toml lesen und anzeigen
NEW_DEV_VERSION=$(grep '^version = ' Cargo.toml | sed 's/version = "\(.*\)"/\1/')

git add .
git commit -m "chore(release): bump to development version v${NEW_DEV_VERSION}"

echo "✅ Successfully bumped to development version: ${NEW_DEV_VERSION}"
echo "📋 Summary:"
echo "   Release level: ${RELEASE_LEVEL}"
echo "   Previous: ${CURRENT_VERSION}"
echo "   Released: ${NEW_VERSION}"
echo "   Next dev: ${NEW_DEV_VERSION}"
echo ""
echo "ℹ️  Note: Git operations (commit/push) are handled by cargo-release"
