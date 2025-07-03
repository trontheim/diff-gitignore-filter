#!/bin/bash
# Commit message validation library
# Implements Conventional Commits validation

# Configuration for allowed types and scopes
ALLOWED_TYPES=("feat" "fix" "docs" "style" "refactor" "perf" "test" "chore" "ci" "build" "revert" "security" "breaking")
ALLOWED_SCOPES=("filter" "root-finder" "config" "cli" "error" "bench" "deps" "integration" "vcs" "unicode" "streaming" "simd")

# Conventional Commits regex pattern
# Format: type(scope): description
readonly CONVENTIONAL_PATTERN='^(feat|fix|docs|style|refactor|perf|test|chore|ci|build|revert|security|breaking)(\([a-z0-9-]+\))?: .{1,50}$'

# Breaking change pattern
readonly BREAKING_PATTERN='(BREAKING CHANGE:|!)'

# Configuration
MAX_SUBJECT_LENGTH=50
ALLOW_UPPERCASE_SUBJECT=false
ALLOW_PERIOD_END=false
REQUIRE_SCOPE=false

# Load commit validation configuration if available
load_commit_config() {
    local config_file="$CONFIG_DIR/commit-validation.conf"

    if [[ -f "$config_file" ]]; then
        debug "Loading commit validation config from: $config_file"
        # shellcheck source=/dev/null
        source "$config_file"

        # Convert space-separated strings to arrays if needed
        if [[ -n "$COMMIT_TYPES" ]]; then
            IFS=' ' read -ra ALLOWED_TYPES <<< "$COMMIT_TYPES"
        fi

        if [[ -n "$COMMIT_SCOPES" ]]; then
            IFS=' ' read -ra ALLOWED_SCOPES <<< "$COMMIT_SCOPES"
        fi

        debug "Commit validation config loaded"
    else
        debug "No commit validation config found, using defaults"
    fi
}

# Main validation function
validate_commit_message() {
    local commit_msg="$1"
    local errors=()

    debug "Validating commit message: $commit_msg"

    # Empty check
    if [[ -z "$commit_msg" ]]; then
        errors+=("Commit message cannot be empty")
        print_validation_result "$commit_msg" "${errors[@]}"
        return 1
    fi

    # Basic format check
    if ! [[ $commit_msg =~ $CONVENTIONAL_PATTERN ]]; then
        errors+=("Does not match conventional commit format: type(scope): description")
    fi

    # Extract and validate type
    local type
    type=$(echo "$commit_msg" | sed -n 's/^\([a-z]*\).*/\1/p')
    if [[ -n "$type" ]] && ! array_contains "$type" "${ALLOWED_TYPES[@]}"; then
        errors+=("Invalid type '$type'. Allowed: ${ALLOWED_TYPES[*]}")
    fi

    # Extract and validate scope (if present) - Fixed regex
    local scope_pattern='\(([^)]+)\)'
    if [[ $commit_msg =~ $scope_pattern ]]; then
        local scope="${BASH_REMATCH[1]}"
        if ! array_contains "$scope" "${ALLOWED_SCOPES[@]}"; then
            errors+=("Invalid scope '$scope'. Allowed: ${ALLOWED_SCOPES[*]}")
        fi
    elif [[ "$REQUIRE_SCOPE" == "true" ]]; then
        errors+=("Scope is required but missing")
    fi

    # Extract and validate subject
    local subject
    subject=$(echo "$commit_msg" | sed -n 's/^[^:]*: \(.*\)/\1/p')
    if [[ -n "$subject" ]]; then
        # Length check
        if [[ ${#subject} -gt $MAX_SUBJECT_LENGTH ]]; then
            errors+=("Subject too long (${#subject} chars). Max $MAX_SUBJECT_LENGTH characters.")
        fi

        # Uppercase check
        if [[ "$ALLOW_UPPERCASE_SUBJECT" == "false" ]] && [[ $subject =~ ^[A-Z] ]]; then
            errors+=("Subject should not start with uppercase letter")
        fi

        # Period check
        if [[ "$ALLOW_PERIOD_END" == "false" ]] && [[ $subject =~ \.$ ]]; then
            errors+=("Subject should not end with period")
        fi

        # Empty subject check
        if [[ -z "$subject" ]]; then
            errors+=("Subject cannot be empty")
        fi
    fi

    # Breaking change detection
    if [[ $commit_msg =~ $BREAKING_PATTERN ]]; then
        warning "Breaking change detected in commit"
    fi

    # Return result
    if [[ ${#errors[@]} -eq 0 ]]; then
        print_validation_result "$commit_msg" "VALID"
        return 0
    else
        print_validation_result "$commit_msg" "${errors[@]}"
        return 1
    fi
}

# Print validation result
print_validation_result() {
    local commit_msg="$1"
    shift
    local result=("$@")

    info "Commit: $commit_msg"

    if [[ "$1" == "VALID" ]]; then
        success "Valid conventional commit"
    else
        error "Invalid commit message:"
        for error in "${result[@]}"; do
            echo "   - $error"
        done
        echo ""
        info "💡 Conventional Commit Format:"
        echo "   type(scope): description"
        echo ""
        info "📋 Examples:"
        echo "   feat(filter): add new pattern matching"
        echo "   fix(cli): resolve argument parsing issue"
        echo "   docs(readme): update installation instructions"
        echo ""
        info "🔗 Types: ${ALLOWED_TYPES[*]}"
        info "🔗 Scopes: ${ALLOWED_SCOPES[*]}"
    fi
}

# Validate multiple recent commits
validate_recent_commits() {
    local count=${1:-10}
    local failed=0

    info "Validating last $count commit messages..."

    # Get commits
    local commits
    commits=$(git log --oneline -n "$count" --pretty=format:"%s")

    while IFS= read -r commit; do
        if [[ -n "$commit" ]]; then
            if ! validate_commit_message "$commit"; then
                ((failed++))
            fi
            echo "---"
        fi
    done <<< "$commits"

    if [[ $failed -eq 0 ]]; then
        success "All $count commits are valid"
        return 0
    else
        error "$failed out of $count commits are invalid"
        return 1
    fi
}

# Validate commit message from file (for Git hooks)
validate_commit_file() {
    local commit_file="$1"

    if [[ ! -f "$commit_file" ]]; then
        error "Commit message file not found: $commit_file"
        return 1
    fi

    local commit_msg
    commit_msg=$(cat "$commit_file")
    validate_commit_message "$commit_msg"
}

# Validate the last commit
validate_last_commit() {
    local last_commit
    last_commit=$(git log -1 --pretty=format:"%s")

    if [[ -z "$last_commit" ]]; then
        error "No commits found"
        return 1
    fi

    info "Validating last commit..."
    validate_commit_message "$last_commit"
}


