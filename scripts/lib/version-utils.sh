#!/bin/bash
# Version management utilities
# Provides functions for version manipulation and validation

# Version comparison functions
version_compare() {
    local version1="$1"
    local version2="$2"

    # Remove 'v' prefix if present
    version1="${version1#v}"
    version2="${version2#v}"

    # Split versions into components
    IFS='.' read -ra v1_parts <<< "$version1"
    IFS='.' read -ra v2_parts <<< "$version2"

    # Compare major, minor, patch
    for i in {0..2}; do
        local v1_part="${v1_parts[i]:-0}"
        local v2_part="${v2_parts[i]:-0}"

        # Remove pre-release suffix for comparison
        v1_part="${v1_part%%-*}"
        v2_part="${v2_part%%-*}"

        if [[ $v1_part -gt $v2_part ]]; then
            echo "1"
            return
        elif [[ $v1_part -lt $v2_part ]]; then
            echo "-1"
            return
        fi
    done

    echo "0"
}

# Get version from Cargo.toml
get_version() {
    local cargo_file="${1:-Cargo.toml}"

    # If using default Cargo.toml and PROJECT_ROOT is set, use get_current_version
    if [[ "$cargo_file" == "Cargo.toml" && -n "$PROJECT_ROOT" ]]; then
        get_current_version
        return $?
    fi

    # Handle custom cargo file path
    if [[ -n "$PROJECT_ROOT" && "$cargo_file" == "Cargo.toml" ]]; then
        cargo_file="$PROJECT_ROOT/Cargo.toml"
    fi

    if [[ ! -f "$cargo_file" ]]; then
        error "Cargo.toml not found: $cargo_file"
        return 1
    fi

    grep '^version = ' "$cargo_file" | sed 's/version = "\(.*\)"/\1/'
}

# Set version in Cargo.toml
set_version() {
    local new_version="$1"
    local cargo_file="${2:-Cargo.toml}"

    # If relative path and PROJECT_ROOT is set, make it absolute
    if [[ -n "$PROJECT_ROOT" && "$cargo_file" == "Cargo.toml" ]]; then
        cargo_file="$PROJECT_ROOT/Cargo.toml"
    fi

    if [[ ! -f "$cargo_file" ]]; then
        error "Cargo.toml not found: $cargo_file"
        return 1
    fi

    if ! validate_version "$new_version"; then
        return 1
    fi

    debug "Setting version to $new_version in $cargo_file"

    # Update version
    sed -i.tmp "s/^version = \".*\"/version = \"$new_version\"/" "$cargo_file"
    rm -f "${cargo_file}.tmp"

    # Verify the change
    local updated_version
    updated_version=$(get_version "$cargo_file")

    if [[ "$updated_version" == "$new_version" ]]; then
        return 0
    else
        error "Failed to update version. Expected: $new_version, Got: $updated_version"
        return 1
    fi
}

# Bump version using built-in semantic versioning logic
bump_version() {
    local level="$1"
    local pre_release="${2:-}"

    debug "Bumping version: level=$level, pre_release=$pre_release"

    # Validate level (allow empty for pre-release only changes)
    if [[ -n "$level" ]] && ! array_contains "$level" "patch" "minor" "major"; then
        error "Invalid bump level: $level. Must be patch, minor, or major"
        return 1
    fi

    # Get current version
    local current_version
    current_version=$(get_version)

    if [[ -z "$current_version" ]]; then
        error "Could not determine current version from Cargo.toml"
        return 1
    fi

    debug "Current version: $current_version"

    # Calculate next version
    local next_version
    if [[ -n "$level" ]]; then
        next_version=$(get_next_version "$current_version" "$level" "$pre_release")
    else
        # Only pre-release change requested
        if [[ -n "$pre_release" ]]; then
            parse_version "$current_version" "version"
            next_version="$version_base-$pre_release"
        else
            error "No bump level or pre-release specified"
            return 1
        fi
    fi

    if [[ -z "$next_version" ]]; then
        error "Failed to calculate next version"
        return 1
    fi

    debug "Next version: $next_version"

    # Validate new version
    if ! validate_version "$next_version"; then
        error "Generated version is invalid: $next_version"
        return 1
    fi

    # Update Cargo.toml
    if set_version "$next_version"; then
        cargo check --quiet
        echo "$next_version"
        return 0
    else
        error "Failed to update version in Cargo.toml"
        return 1
    fi
}

# Parse version components
parse_version() {
    local version="$1"
    local result_var="$2"

    # Remove 'v' prefix if present
    version="${version#v}"

    # Split version and pre-release
    local base_version="${version%%-*}"
    local pre_release=""

    if [[ "$version" == *"-"* ]]; then
        pre_release="${version#*-}"
    fi

    # Split base version into components
    IFS='.' read -ra version_parts <<< "$base_version"

    # Use eval for compatibility with older bash versions
    eval "${result_var}_major=${version_parts[0]:-0}"
    eval "${result_var}_minor=${version_parts[1]:-0}"
    eval "${result_var}_patch=${version_parts[2]:-0}"
    eval "${result_var}_pre_release='$pre_release'"
    eval "${result_var}_full='$version'"
    eval "${result_var}_base='$base_version'"
}

# Get next version based on bump level
get_next_version() {
    local current_version="$1"
    local bump_level="$2"
    local pre_release="${3:-}"

    # Parse version without associative arrays for compatibility
    parse_version "$current_version" "version"

    local major="$version_major"
    local minor="$version_minor"
    local patch="$version_patch"
    local current_pre_release="$version_pre_release"

    # Smart version handling for pre-release versions
    if [[ -n "$current_pre_release" && "$bump_level" == "patch" ]]; then
        if [[ -z "$pre_release" ]]; then
            # No new pre-release suffix specified: remove existing pre-release suffix
            debug "Current version is pre-release: $current_version"
            debug "Removing pre-release suffix instead of incrementing patch"
            echo "$major.$minor.$patch"
            return 0
        else
            # New pre-release suffix specified: replace existing pre-release suffix without incrementing
            debug "Current version is pre-release: $current_version"
            debug "Replacing pre-release suffix '$current_pre_release' with '$pre_release' without incrementing"
            echo "$major.$minor.$patch-$pre_release"
            return 0
        fi
    fi

    case "$bump_level" in
        "patch")
            ((patch++))
            ;;
        "minor")
            ((minor++))
            patch=0
            ;;
        "major")
            ((major++))
            minor=0
            patch=0
            ;;
        *)
            error "Invalid bump level: $bump_level"
            return 1
            ;;
    esac

    local next_version="$major.$minor.$patch"

    if [[ -n "$pre_release" ]]; then
        next_version="$next_version-$pre_release"
    fi

    echo "$next_version"
}


# Get latest git tag
get_latest_tag() {
    git describe --tags --abbrev=0 2>/dev/null || echo ""
}


# Check if tag exists
tag_exists() {
    local tag="$1"
    git tag -l | grep -q "^${tag}$"
}



# Validate version format (basic check without error messages)
is_valid_version() {
    local version="$1"
    local version_pattern='^[0-9]+\.[0-9]+\.[0-9]+(-[a-zA-Z0-9.-]+)?$'
    [[ $version =~ $version_pattern ]]
}

# Version validation with error messages and debug output
validate_version() {
    local version="$1"

    if [[ -z "$version" ]]; then
        error "Version cannot be empty"
        return 1
    fi

    if ! is_valid_version "$version"; then
        error "Invalid version format: $version"
        error "Expected format: MAJOR.MINOR.PATCH[-PRERELEASE]"
        return 1
    fi

    debug "Version validation passed: $version"
    return 0
}


# Get current version from Cargo.toml (basic version)
get_current_version() {
    grep '^version = ' "$PROJECT_ROOT/Cargo.toml" | sed 's/version = "\(.*\)"/\1/'
}

# Check if version is a pre-release
is_prerelease() {
    local version="$1"
    [[ "$version" == *"-"* ]]
}



