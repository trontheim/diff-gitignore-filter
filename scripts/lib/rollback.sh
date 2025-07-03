#!/bin/bash
# Rollback mechanisms for release scripts
# Provides comprehensive rollback functionality with phase-specific handling

# Global variables for rollback state
INITIAL_HEAD=""
INITIAL_VERSION=""
INITIAL_BRANCH=""
ROLLBACK_LOG_FILE=""
MODIFIED_FILES=()

# Initialize rollback system
init_rollback() {
    debug "Initializing rollback system..."

    # Save initial state
    INITIAL_HEAD=$(git rev-parse HEAD)
    INITIAL_VERSION=$(get_version)
    INITIAL_BRANCH=$(get_current_branch)
    ROLLBACK_LOG_FILE="/tmp/release-rollback-$(date +%s).log"

    # Create rollback log
    {
        echo "# Release Rollback Log"
        echo "# Generated: $(date)"
        echo "INITIAL_HEAD=$INITIAL_HEAD"
        echo "INITIAL_VERSION=$INITIAL_VERSION"
        echo "INITIAL_BRANCH=$INITIAL_BRANCH"
        echo "PROJECT_ROOT=$PROJECT_ROOT"
        echo ""
    } > "$ROLLBACK_LOG_FILE"

    success "Rollback system initialized"
    debug "Initial HEAD: $INITIAL_HEAD"
    debug "Initial version: $INITIAL_VERSION"
    debug "Initial branch: $INITIAL_BRANCH"
    debug "Rollback log: $ROLLBACK_LOG_FILE"

    export INITIAL_HEAD INITIAL_VERSION INITIAL_BRANCH ROLLBACK_LOG_FILE
}

# Log rollback action
log_rollback_action() {
    local action="$1"
    local details="$2"
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    echo "[$timestamp] $action: $details" >> "$ROLLBACK_LOG_FILE"
    debug "Rollback action logged: $action - $details"
}


# Track modified file for potential restoration
track_modified_file() {
    local file="$1"
    MODIFIED_FILES+=("$file")
    log_rollback_action "FILE_MODIFIED" "$file"
    debug "Tracking modified file: $file"
}

# Save current state before making changes
save_current_state() {
    local phase="$1"
    local state_file="/tmp/release-state-$phase-$(date +%s).json"

    {
        echo "{"
        echo "  \"phase\": \"$phase\","
        echo "  \"timestamp\": \"$(date -Iseconds)\","
        echo "  \"head\": \"$(git rev-parse HEAD)\","
        echo "  \"version\": \"$(get_version)\","
        echo "  \"branch\": \"$(get_current_branch)\","
        echo "  \"working_dir_clean\": $(is_working_dir_clean && echo "true" || echo "false"),"
        echo "  \"tags\": [],"
        echo "  \"modified_files\": ["
        first=true
        for file in "${MODIFIED_FILES[@]}"; do
            [[ "$first" == "true" ]] && first=false || echo ","
            echo "    \"$file\""
        done
        echo "  ]"
        echo "}"
    } > "$state_file"

    log_rollback_action "STATE_SAVED" "$state_file"
    debug "State saved for phase $phase: $state_file"
}

# Clean up tags created during release process
cleanup_created_tags() {
    debug "Tag cleanup disabled (no tags tracked)"
    return 0
}

# Clean up tags created after a specific commit
cleanup_tags_after_commit() {
    local commit="$1"

    if [[ -z "$commit" ]]; then
        error "No commit specified for tag cleanup"
        return 1
    fi

    debug "Cleaning up tags created after commit: $commit"

    # Get tags that are reachable from HEAD but not from the specified commit
    local tags_to_delete
    tags_to_delete=$(git tag --merged HEAD --no-merged "$commit" 2>/dev/null || true)

    if [[ -n "$tags_to_delete" ]]; then
        info "Cleaning up tags created during release process..."
        echo "$tags_to_delete" | while IFS= read -r tag; do
            if [[ -n "$tag" ]]; then
                if delete_tag "$tag"; then
                    log_rollback_action "TAG_CLEANUP" "$tag"
                else
                    warning "Failed to clean up tag: $tag"
                fi
            fi
        done
    else
        debug "No tags to clean up after commit: $commit"
    fi
}

# Restore files to their initial state
restore_modified_files() {
    if [[ ${#MODIFIED_FILES[@]} -eq 0 ]]; then
        debug "No files to restore"
        return 0
    fi

    info "Restoring modified files..."

    for file in "${MODIFIED_FILES[@]}"; do
        if [[ -f "$file" ]]; then
            local backup_file="${file}.backup.*"
            # Find the most recent backup
            local latest_backup
            latest_backup=$(ls -t ${backup_file} 2>/dev/null | head -1)

            if [[ -n "$latest_backup" && -f "$latest_backup" ]]; then
                if cp "$latest_backup" "$file"; then
                    success "Restored file: $file"
                    log_rollback_action "FILE_RESTORED" "$file"
                else
                    error "Failed to restore file: $file"
                    log_rollback_action "FILE_RESTORE_FAILED" "$file"
                fi
            else
                warning "No backup found for file: $file"
                log_rollback_action "FILE_NO_BACKUP" "$file"
            fi
        fi
    done

    # Clear the array
    MODIFIED_FILES=()
}

# Full rollback to initial HEAD
rollback_to_initial_head() {
    local reason="${1:-Unknown error}"

    warning "Rolling back to initial HEAD due to: $reason"
    log_rollback_action "FULL_ROLLBACK_START" "$reason"

    if [[ -z "$INITIAL_HEAD" ]]; then
        error "No initial HEAD saved - cannot rollback safely"
        log_rollback_action "ROLLBACK_FAILED" "No initial HEAD"
        return 1
    fi

    # Save current state before rollback
    save_current_state "rollback"

    # Reset to initial HEAD
    info "Resetting to initial HEAD: $INITIAL_HEAD"
    if git reset --hard "$INITIAL_HEAD"; then
        success "Reset to initial HEAD completed"
        log_rollback_action "HEAD_RESET" "$INITIAL_HEAD"
    else
        error "Failed to reset to initial HEAD"
        log_rollback_action "HEAD_RESET_FAILED" "$INITIAL_HEAD"
        return 1
    fi

    # Clean up tags
    cleanup_tags_after_commit "$INITIAL_HEAD"

    # Restore version if needed
    local current_version
    current_version=$(get_version)
    if [[ "$current_version" != "$INITIAL_VERSION" ]]; then
        info "Restoring initial version: $INITIAL_VERSION"
        if set_cargo_version "$INITIAL_VERSION"; then
            log_rollback_action "VERSION_RESTORED" "$INITIAL_VERSION"
        else
            warning "Failed to restore initial version"
            log_rollback_action "VERSION_RESTORE_FAILED" "$INITIAL_VERSION"
        fi
    fi

    success "Rollback to initial HEAD completed: $INITIAL_HEAD"
    log_rollback_action "FULL_ROLLBACK_COMPLETE" "$INITIAL_HEAD"

    return 0
}

# Phase-specific rollback
rollback_by_phase() {
    local phase="$1"
    local version="${2:-unknown}"
    local reason="${3:-Error in $phase}"

    info "Performing phase-specific rollback for: $phase"
    log_rollback_action "PHASE_ROLLBACK_START" "$phase - $reason"

    case "$phase" in
        "validation")
            info "Validation phase error - no rollback needed (no changes made)"
            warning "Fix the validation issues and try again"
            # Skip rollback summary for validation errors
            SKIP_ROLLBACK_SUMMARY=true
            return 0
            ;;
        "pre-release"|"pre-bump")
            info "Early phase rollback - resetting to initial state"
            rollback_to_initial_head "$reason"
            ;;
        "post-bump"|"release")
            info "Post-bump rollback - resetting to initial state and cleaning tags"
            rollback_to_initial_head "$reason"
            ;;
        "post-release")
            warning "Post-release error - manual intervention may be required"
            warning "Initial HEAD was: $INITIAL_HEAD"
            warning "Current HEAD is: $(git rev-parse HEAD)"
            warning "To rollback manually: git reset --hard $INITIAL_HEAD"
            log_rollback_action "POST_RELEASE_ERROR" "$reason"

            # Still try to clean up what we can
            cleanup_created_tags
            ;;
        *)
            warning "Unknown phase - performing safe rollback"
            rollback_to_initial_head "$reason"
            ;;
    esac

    log_rollback_action "PHASE_ROLLBACK_COMPLETE" "$phase"
}

# Emergency rollback (for use in trap handlers)
emergency_rollback() {
    local exit_code="$1"
    local signal="${2:-}"

    error "Emergency rollback triggered (exit code: $exit_code, signal: $signal)"
    log_rollback_action "EMERGENCY_ROLLBACK" "exit_code=$exit_code signal=$signal"

    # Try to determine current phase
    local current_phase="${CURRENT_PHASE:-unknown}"

    # Perform rollback based on current phase
    rollback_by_phase "$current_phase" "unknown" "Emergency rollback (exit: $exit_code)"

    # Show rollback summary
    show_rollback_summary

    exit "$exit_code"
}

# Show rollback summary
show_rollback_summary() {
    # Skip summary if no actual rollback was performed
    if [[ "$SKIP_ROLLBACK_SUMMARY" == "true" ]]; then
        return 0
    fi

    echo ""
    info "=== Rollback Summary ==="
    info "Initial HEAD: $INITIAL_HEAD"
    info "Initial version: $INITIAL_VERSION"
    info "Initial branch: $INITIAL_BRANCH"

    if [[ -f "$ROLLBACK_LOG_FILE" ]]; then
        info "Rollback log: $ROLLBACK_LOG_FILE"
        echo ""
        info "Recent rollback actions:"
        tail -10 "$ROLLBACK_LOG_FILE" | while IFS= read -r line; do
            echo "  $line"
        done
    fi

    echo ""
    info "Current state:"
    info "  HEAD: $(git rev-parse HEAD)"
    info "  Version: $(get_version)"
    info "  Branch: $(get_current_branch)"
    info "  Working dir clean: $(is_working_dir_clean && echo "Yes" || echo "No")"
}

# Cleanup rollback system
cleanup_rollback() {
    debug "Cleaning up rollback system..."

    # Remove temporary files
    if [[ -n "$ROLLBACK_LOG_FILE" && -f "$ROLLBACK_LOG_FILE" ]]; then
        debug "Removing rollback log: $ROLLBACK_LOG_FILE"
        rm -f "$ROLLBACK_LOG_FILE"
    fi

    # Clear arrays
    MODIFIED_FILES=()

    debug "Rollback system cleanup completed"
}

# Setup trap handlers for automatic rollback
setup_rollback_traps() {
    debug "Setting up rollback trap handlers..."

    # Trap for various signals and errors
    trap 'emergency_rollback $? SIGINT' INT
    trap 'emergency_rollback $? SIGTERM' TERM
    trap 'emergency_rollback $? EXIT' EXIT

    debug "Rollback trap handlers configured"
}

# Disable rollback traps
disable_rollback_traps() {
    debug "Disabling rollback trap handlers..."

    trap - INT TERM EXIT

    debug "Rollback trap handlers disabled"
}


