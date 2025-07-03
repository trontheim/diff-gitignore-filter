#!/bin/bash
set -e

CURRENT_VERSION=$1
NEW_VERSION=$2
RELEASE_LEVEL=$3

echo "Running pre-release checks..."

echo "1. Checking code compilation..."
cargo check --all-targets --all-features

echo "2. Checking code formatting..."
cargo fmt --all --check

echo "3. Running clippy lints..."
cargo clippy --all-targets --all-features -- -D warnings

echo "4. Running tests..."
cargo test --all-features

echo "5. Generating CHANGELOG..."
git cliff --output CHANGELOG.md

echo "Pre-release checks completed successfully!"
