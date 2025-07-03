#!/bin/bash
# Common functions for release scripts
# Provides logging, configuration loading, and utility functions

# Color support detection and definitions
if [[ -z "${RED:-}" ]]; then
    if [[ -t 1 ]] && command -v tput >/dev/null 2>&1 && tput colors >/dev/null 2>&1; then
        RED=$(tput setaf 1)
        GREEN=$(tput setaf 2)
        YELLOW=$(tput setaf 3)
        BLUE=$(tput setaf 4)
        PURPLE=$(tput setaf 5)
        CYAN=$(tput setaf 6)
        WHITE=$(tput setaf 7)
        NC=$(tput sgr0)
    else
        RED=""
        GREEN=""
        YELLOW=""
        BLUE=""
        PURPLE=""
        CYAN=""
        WHITE=""
        NC=""
    fi

    # Export for use in other scripts
    export RED GREEN YELLOW BLUE PURPLE CYAN WHITE NC
fi

# Global configuration variables
LIB_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# Determine PROJECT_ROOT based on script location
# From .cargo/bin/lib, go up three levels to reach project root
if command -v realpath >/dev/null 2>&1; then
    PROJECT_ROOT="$(realpath "$LIB_DIR/../..")"
else
    PROJECT_ROOT="$(cd "$LIB_DIR/../.." && pwd)"
fi
export PROJECT_ROOT
CONFIG_DIR="$LIB_DIR/../config"
HOOKS_DIR="$LIB_DIR/../hooks"

# Default configuration values
MAIN_BRANCH="${MAIN_BRANCH:-main}"
REMOTE_NAME="${REMOTE_NAME:-origin}"
ENABLE_ROLLBACK="${ENABLE_ROLLBACK:-true}"
VERBOSE_OUTPUT="${VERBOSE_OUTPUT:-true}"
COLOR_OUTPUT="${COLOR_OUTPUT:-true}"
LOG_LEVEL="${LOG_LEVEL:-INFO}"

# Logging function for debug output
log_debug() {
    [[ "$LOG_LEVEL" == "DEBUG" ]] && echo -e "${CYAN}[DEBUG]${NC} $*" >&2
}

# Enhanced output functions with color support
info() {
    if [[ "$COLOR_OUTPUT" == "true" ]]; then
        echo -e "${BLUE}ℹ️  $*${NC}"
    else
        echo "ℹ️  $*"
    fi
}

success() {
    if [[ "$COLOR_OUTPUT" == "true" ]]; then
        echo -e "${GREEN}✅ $*${NC}"
    else
        echo "✅ $*"
    fi
}

warning() {
    if [[ "$COLOR_OUTPUT" == "true" ]]; then
        echo -e "${YELLOW}⚠️  $*${NC}"
    else
        echo "⚠️  $*"
    fi
}

error() {
    if [[ "$COLOR_OUTPUT" == "true" ]]; then
        echo -e "${RED}❌ $*${NC}" >&2
    else
        echo "❌ $*" >&2
    fi
}

verbose() {
    if [[ "$VERBOSE_OUTPUT" == "true" ]]; then
        if [[ "$COLOR_OUTPUT" == "true" ]]; then
            echo -e "${CYAN}ℹ️  $*${NC}"
        else
            echo "ℹ️  $*"
        fi
    fi
}

debug() {
    [[ "$LOG_LEVEL" == "DEBUG" ]] && log_debug "$@"
}

# Configuration loading
load_config() {
    local config_file="$CONFIG_DIR/release.conf"

    if [[ -f "$config_file" ]]; then
        debug "Loading configuration from: $config_file"
        # shellcheck source=/dev/null
        source "$config_file"
        debug "Configuration loaded successfully"
    else
        debug "No configuration file found at: $config_file, using defaults"
    fi
}

# Hook execution with phase tracking
execute_hook() {
    local hook_name="$1"
    local phase="$2"
    shift 2
    local hook_args=("$@")

    local hook_script="$HOOKS_DIR/$hook_name.sh"

    if [[ ! -f "$hook_script" ]]; then
        warning "Hook script not found: $hook_script"
        return 0
    fi

    if [[ ! -x "$hook_script" ]]; then
        warning "Hook script not executable: $hook_script"
        return 0
    fi

    info "Executing $hook_name hook..."
    debug "Hook script: $hook_script"
    debug "Hook args: ${hook_args[*]}"

    # Set current phase for rollback purposes
    export CURRENT_PHASE="$phase"

    if [[ "$DRY_RUN" == "true" ]]; then
        success "$hook_name hook completed successfully"
        return 0
    fi

    if bash "$hook_script" "${hook_args[@]}"; then
        success "$hook_name hook completed successfully"
        return 0
    else
        error "$hook_name hook failed"
        return 1
    fi
}


# Safe execution with automatic rollback
safe_execute() {
    local description="$1"
    shift
    local command=("$@")

    info "Executing: $description"
    debug "Command: ${command[*]}"

    if "${command[@]}"; then
        success "$description completed"
        return 0
    else
        error "$description failed"
        if [[ "$ENABLE_ROLLBACK" == "true" ]]; then
            warning "Automatic rollback will be triggered"
        fi
        return 1
    fi
}

# Utility functions
is_git_repo() {
    git rev-parse --git-dir >/dev/null 2>&1
}

get_current_branch() {
    git branch --show-current
}

is_working_dir_clean() {
    [[ -z "$(git status --porcelain)" ]]
}


# Array utility functions
array_contains() {
    local element="$1"
    shift
    local array=("$@")

    for item in "${array[@]}"; do
        if [[ "$item" == "$element" ]]; then
            return 0
        fi
    done
    return 1
}


# Initialize common environment
init_common() {
    # Note: Working directory is managed by the main script
    # All file operations use absolute paths via PROJECT_ROOT

    # Load configuration
    load_config

    # Validate git repository
    if ! is_git_repo; then
        error "Not in a git repository"
        exit 1
    fi

    debug "Common environment initialized"
    debug "Project root: $PROJECT_ROOT"
    debug "Script directory: $SCRIPT_DIR"
    debug "Current directory: $(pwd)"
}

