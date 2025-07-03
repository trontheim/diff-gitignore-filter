#!/bin/bash
# Main release script
# Orchestrates the complete release workflow with hook system and rollback support

# Source all required libraries
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"

# Note: PROJECT_ROOT will be determined by common.sh
# All file operations use absolute paths via PROJECT_ROOT

# shellcheck source=./lib/common.sh
source "$SCRIPT_DIR/lib/common.sh"
# shellcheck source=./lib/commit-validator.sh
source "$SCRIPT_DIR/lib/commit-validator.sh"
# shellcheck source=./lib/version-utils.sh
source "$SCRIPT_DIR/lib/version-utils.sh"
# shellcheck source=./lib/rollback.sh
source "$SCRIPT_DIR/lib/rollback.sh"

# Initialize common environment
init_common

# Global variables for release state
CURRENT_PHASE=""
RELEASE_LEVEL=""
CURRENT_VERSION=""
NEW_VERSION=""
RELEASE_START_TIME=""

# Display usage information
show_usage() {
    echo "Usage: $0 [RELEASE_LEVEL] [OPTIONS]"
    echo ""
    echo "RELEASE_LEVEL:"
    echo "  patch    Increment patch version (default)"
    echo "  minor    Increment minor version"
    echo "  major    Increment major version"
    echo ""
    echo "OPTIONS:"
    echo "  -h, --help           Show this help message"
    echo "  -v, --verbose        Enable verbose output"
    echo "  -d, --debug          Enable debug output"
    echo "  -n, --dry-run        Perform dry run without making changes"
    echo "  --pre-release SUFFIX Create a pre-release version (e.g., alpha, beta, rc)"
    echo "  --no-push            Skip automatic push to remote"
    echo "  --no-hooks           Skip hook execution"
    echo "  --no-rollback        Disable automatic rollback on errors"
    echo ""
    echo "EXAMPLES:"
    echo "  $0                           # Patch release"
    echo "  $0 minor                     # Minor release"
    echo "  $0 major --verbose           # Major release with verbose output"
    echo "  $0 major --debug             # Major release with debug output"
    echo "  $0 patch --dry-run           # Test patch release without changes"
    echo "  $0 minor --pre-release alpha # Minor pre-release (e.g., 0.2.0-alpha)"
    echo "  $0 patch --pre-release beta  # Patch pre-release (e.g., 0.1.1-beta)"
    echo ""
    echo "ENVIRONMENT VARIABLES:"
    echo "  MAIN_BRANCH          Main branch name (default: main)"
    echo "  REMOTE_NAME          Remote name (default: origin)"
    echo "  LOG_LEVEL            Logging level (DEBUG, INFO, WARN, ERROR)"
    echo ""
}

# Parse command line arguments
parse_arguments() {
    RELEASE_LEVEL="patch"  # Default
    PRE_RELEASE=""
    DRY_RUN=false
    VERBOSE_OUTPUT=false
    SKIP_HOOKS=false
    SKIP_PUSH=false

    while [[ $# -gt 0 ]]; do
        case $1 in
            patch|minor|major)
                RELEASE_LEVEL="$1"
                shift
                ;;
            -h|--help)
                show_usage
                exit 0
                ;;
            -v|--verbose)
                VERBOSE_OUTPUT=true
                shift
                ;;
            -d|--debug)
                LOG_LEVEL="DEBUG"
                shift
                ;;
            -n|--dry-run)
                DRY_RUN=true
                info "Dry run mode enabled - no changes will be made"
                shift
                ;;
            --pre-release)
                if [[ -z "$2" || "$2" == -* ]]; then
                    error "--pre-release requires a suffix (e.g., alpha, beta, rc)"
                    show_usage
                    exit 1
                fi
                PRE_RELEASE="$2"
                shift 2
                ;;
            --no-push)
                SKIP_PUSH=true
                shift
                ;;
            --no-hooks)
                SKIP_HOOKS=true
                warning "Hook execution disabled"
                shift
                ;;
            --no-rollback)
                ENABLE_ROLLBACK=false
                warning "Automatic rollback disabled"
                shift
                ;;
            *)
                error "Unknown argument: $1"
                show_usage
                exit 1
                ;;
        esac
    done

    debug "Parsed arguments:"
    debug "  Release level: $RELEASE_LEVEL"
    debug "  Pre-release suffix: $PRE_RELEASE"
    debug "  Dry run: $DRY_RUN"
    debug "  Skip hooks: $SKIP_HOOKS"
    debug "  Skip push: $SKIP_PUSH"
    debug "  Enable rollback: $ENABLE_ROLLBACK"
}

# Initialize release environment
init_release() {
    RELEASE_START_TIME=$(date -Iseconds)

    info "🚀 Starting release process..."
    verbose "Release level: $RELEASE_LEVEL"
    verbose "Start time: $RELEASE_START_TIME"

    # Initialize rollback system if enabled and not dry run
    if [[ "$ENABLE_ROLLBACK" == "true" && "$DRY_RUN" != "true" ]]; then
        init_rollback
        setup_rollback_traps
    elif [[ "$DRY_RUN" == "true" ]]; then
        debug "Rollback disabled for dry run"
    fi

    # Get current version
    CURRENT_VERSION=$(get_version)
    verbose "Current version: $CURRENT_VERSION"

    # Calculate new version
    NEW_VERSION=$(get_next_version "$CURRENT_VERSION" "$RELEASE_LEVEL" "$PRE_RELEASE")
    info "Target version: $NEW_VERSION"

    debug "Release environment initialized"
}

# Validate release prerequisites
validate_prerequisites() {
    CURRENT_PHASE="validation"
    info "🔍 Validating release prerequisites..."

    # Check if we're in a git repository
    if ! is_git_repo; then
        error "Not in a git repository"
        exit 1
    fi

    # Check current branch
    local current_branch
    current_branch=$(get_current_branch)
    if [[ "$current_branch" != "$MAIN_BRANCH" ]]; then
        error "Must be on $MAIN_BRANCH branch to release"
        error "Current branch: $current_branch"
        exit 1
    fi
    success "On correct branch: $current_branch"

    # Check working directory cleanliness
    if ! is_working_dir_clean; then
        if [[ "$DRY_RUN" == "true" ]]; then
            warning "Working directory is not clean (continuing with dry run)"
            warning "In a real release, please commit or stash changes first"
        else
            error "Working directory is not clean"
            error "Please commit or stash changes before releasing"
            exit 1
        fi
    else
        success "Working directory is clean"
    fi

    # Check remote connectivity
    if ! git ls-remote --exit-code "$REMOTE_NAME" >/dev/null 2>&1; then
        error "Cannot connect to remote: $REMOTE_NAME"
        exit 1
    fi
    verbose "Remote connectivity verified"

    # Check if local branch is up to date (skip for dry run)
    if [[ "$DRY_RUN" != "true" ]]; then
        git fetch "$REMOTE_NAME" "$MAIN_BRANCH" --quiet
        local local_commit remote_commit
        local_commit=$(git rev-parse HEAD)
        remote_commit=$(git rev-parse "$REMOTE_NAME/$MAIN_BRANCH")

        if [[ "$local_commit" != "$remote_commit" ]]; then
            error "Local branch is not up to date with remote"
            error "Local:  $local_commit"
            error "Remote: $remote_commit"
            error "Run: git pull $REMOTE_NAME $MAIN_BRANCH"
            exit 1
        fi
        verbose "Local branch is up to date"
    fi

    # Validate version format
    if ! validate_version "$NEW_VERSION"; then
        error "Invalid target version: $NEW_VERSION"
        exit 1
    fi

    # Check if target version tag already exists
    if tag_exists "v$NEW_VERSION"; then
        error "Tag already exists for version: v$NEW_VERSION"
        exit 1
    fi
    verbose "Target version available: v$NEW_VERSION"

    # Validate required tools
    local required_tools=("cargo" "git")

    # Note: cargo-bump no longer required - using built-in version bump


    for tool in "${required_tools[@]}"; do
        if ! command -v "$tool" >/dev/null 2>&1; then
            error "Required tool not found: $tool"
            exit 1
        fi
    done
    verbose "All required tools available"

    # Validate commit messages if enabled
    if [[ "$USE_COMMIT_VALIDATOR" == "true" ]]; then
        if ! validate_recent_commits "$VERIFY_LAST_N_COMMITS"; then
            error "Commit message validation failed"
            exit 1
        fi
        success "Commit message validation passed"
    fi

    success "All prerequisites validated"
}

# Execute hook with error handling
execute_release_hook() {
    local hook_name="$1"
    local phase="$2"
    shift 2
    local hook_args=("$@")

    if [[ "$SKIP_HOOKS" == "true" ]]; then
        info "Skipping $hook_name hook (--no-hooks specified)"
        return 0
    fi

    CURRENT_PHASE="$phase"

    if execute_hook "$hook_name" "$phase" "${hook_args[@]}"; then
        return 0
    else
        error "$hook_name hook failed"
        if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
            rollback_by_phase "$phase" "$NEW_VERSION" "$hook_name hook failure"
        fi
        exit 1
    fi
}

# Perform version bump
perform_version_bump() {
    CURRENT_PHASE="version-bump"
    info "🔧 Performing version bump..."

    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would bump version from $CURRENT_VERSION to $NEW_VERSION"
        return 0
    fi

    # Track Cargo.toml for rollback
    track_modified_file "Cargo.toml"

    # Perform the version bump
    local bumped_version
    if bumped_version=$(bump_version "$RELEASE_LEVEL" "$PRE_RELEASE"); then
        # Update NEW_VERSION with the actual bumped version
        NEW_VERSION="$bumped_version"
        debug "Version bump successful: $NEW_VERSION"
    else
        error "Version bump failed"
        if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
            rollback_by_phase "$CURRENT_PHASE" "$NEW_VERSION" "Version bump failure"
        fi
        exit 1
    fi

    # Verify the version bump
    local updated_version
    updated_version=$(get_version)
    if [[ "$updated_version" != "$NEW_VERSION" ]]; then
        error "Version bump verification failed"
        error "Expected: $NEW_VERSION, Got: $updated_version"
        if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
            rollback_by_phase "$CURRENT_PHASE" "$NEW_VERSION" "version bump verification failure"
        fi
        exit 1
    fi

    success "Version updated to $NEW_VERSION"
}

perform_release_commit() {
    CURRENT_PHASE="release-commit"
    info "📦 Committing release changes..."

    if [[ "$DRY_RUN" == "true" ]]; then
        info "DRY RUN: Would commit changes for version $NEW_VERSION"
        return 0
    fi

    # Commit changes
    git add .
    git commit -m "chore(release): v$NEW_VERSION"

    # Tag the release
    git tag -a "v$NEW_VERSION" -m "v$NEW_VERSION"

    success "Changes committed and tagged as v$NEW_VERSION"
}

# Main release workflow
main_release_workflow() {
    info "📋 Executing main release workflow..."

    # Phase 1: Pre-release checks
    execute_release_hook "pre-release" "pre-release" "$CURRENT_VERSION" "$NEW_VERSION" "$RELEASE_LEVEL"

    # Phase 2: Pre-bump validations
    execute_release_hook "pre-bump" "pre-bump" "$CURRENT_VERSION" "$NEW_VERSION" "$RELEASE_LEVEL"

    # Phase 3: Version bump
    perform_version_bump

    # Phase 4: Post-bump operations
    execute_release_hook "post-bump" "post-bump" "$CURRENT_VERSION" "$NEW_VERSION" "$RELEASE_LEVEL"

    perform_release_commit

    # Phase 5: Post-release operations
    execute_release_hook "post-release" "post-release" "$CURRENT_VERSION" "$NEW_VERSION" "$RELEASE_LEVEL"

    success "Main release workflow completed"
}

# Show release summary
show_release_summary() {
    local end_time
    end_time=$(date -Iseconds)

    if [[ "$VERBOSE_OUTPUT" == "true" ]]; then
        echo ""
        info "=== Release Summary ==="
        info "🎯 Release Level: $RELEASE_LEVEL"
        info "📦 Version: $CURRENT_VERSION → $NEW_VERSION"
        info "🏷️  Tag: v$NEW_VERSION"
        info "🌿 Branch: $(get_current_branch)"
        info "⏰ Start Time: $RELEASE_START_TIME"
        info "⏰ End Time: $end_time"

        if [[ "$DRY_RUN" == "true" ]]; then
            info "🧪 Mode: Dry Run (no changes made)"
        else
            info "✅ Mode: Live Release"
        fi

        echo ""
        info "=== Git Status ==="
        info "HEAD: $(git rev-parse --short HEAD)"
    else
        # Minimal summary for standard output
        info "✅ Release completed: $CURRENT_VERSION → $NEW_VERSION"
    fi

    if [[ "$VERBOSE_OUTPUT" == "true" ]]; then
        if [[ "$DRY_RUN" != "true" ]]; then
            info "Recent commits:"
            git log --oneline -3

            echo ""
            info "Recent tags:"
            git tag -l | tail -3
        fi

        echo ""
        if [[ "$DRY_RUN" == "true" ]]; then
            info "=== Next Steps (Dry Run) ==="
            info "Run without --dry-run to perform the actual release:"
            info "  $0 $RELEASE_LEVEL"
        else
            info "=== Next Steps ==="
            if [[ "$SKIP_PUSH" != "true" ]]; then
                info "📤 Push changes to remote:"
                info "   git push $REMOTE_NAME $MAIN_BRANCH"
                info "   git push $REMOTE_NAME --tags"
            fi

            info "🔍 Verify the release:"
            info "   git show v$NEW_VERSION"
            info "   git log --oneline -5"
        fi

        echo ""
        info "📋 Additional tasks:"
        info "   - Update documentation if needed"
        info "   - Announce the release"
        info "   - Monitor for issues"
    fi
}

# Cleanup and exit
cleanup_and_exit() {
    local exit_code=${1:-0}

    debug "Cleaning up release environment..."

    # Disable rollback traps
    if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
        disable_rollback_traps

        if [[ $exit_code -eq 0 ]]; then
            cleanup_rollback
        fi
    fi

    debug "Release script cleanup completed"
    exit $exit_code
}

# Main function
main() {
    # Parse command line arguments
    parse_arguments "$@"

    # Initialize release environment
    init_release

    # Validate prerequisites
    validate_prerequisites

    # Execute main release workflow
    main_release_workflow

    # Show summary
    show_release_summary

    # Success!
    if [[ "$DRY_RUN" == "true" ]]; then
        success "🧪 Dry run completed successfully!"
    else
        success "🎉 Release $NEW_VERSION completed successfully!"
    fi

    # Clean exit
    cleanup_and_exit 0
}

# Error handler for unexpected errors
handle_error() {
    local exit_code=$?
    local line_number=$1

    if [[ "$DRY_RUN" == "true" ]]; then
        error "DRY RUN: Error on line $line_number (exit code: $exit_code)"
        exit $exit_code
    fi

    error "Unexpected error on line $line_number (exit code: $exit_code)"

    if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
        emergency_rollback $exit_code "unexpected_error_line_$line_number"
    fi

    cleanup_and_exit $exit_code
}

# Set up error handling (only for non-dry-run)
if [[ "${DRY_RUN:-false}" != "true" ]]; then
    trap 'handle_error $LINENO' ERR
fi

# Run main function if script is executed directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi
