#!/bin/bash
# Pre-bump hook
# Additional validations before version bump

set -e

CURRENT_VERSION=$1
NEW_VERSION=$2
RELEASE_LEVEL=$3

