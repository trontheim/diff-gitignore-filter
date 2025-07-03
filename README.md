# diff-gitignore-filter

[![CI](https://github.com/trontheim/diff-gitignore-filter/actions/workflows/automated-tests.yml/badge.svg)](https://github.com/trontheim/diff-gitignore-filter/actions/workflows/automated-tests.yml)
[![codecov](https://codecov.io/gh/trontheim/diff-gitignore-filter/branch/main/graph/badge.svg)](https://codecov.io/gh/trontheim/diff-gitignore-filter)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

A pure stream-filter for Git diffs that respects `.gitignore` patterns. Designed to be configured as Git's external diff tool for seamless integration with memory-efficient stream processing.

**Version:** 0.1.0-dev
**Author:** Valgard Trontheim <valgard@trontheim.com>

## Table of Contents

- [Features](#features)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage](#usage)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Performance](#performance)
- [Development](#development)
- [Contributing](#contributing)
- [License](#license)

## Features

- **🌊 Pure Stream Processing**: Memory-efficient line-by-line diff processing
- **💾 Memory Efficient**: Stream-based processing with constant memory usage for text diffs
- **🌍 Complete .gitignore Support**: All standard patterns including negations and complex rules
- **🔧 Git Worktree Support**: Full compatibility with Git worktrees and submodules
- **🌐 Unicode Path Handling**: Robust support for international filenames and Git escape sequences
- **🔗 Downstream Filter Integration**: Seamless chaining with tools like Delta, Bat, and Less
- **⚙️ Git Config Integration**: Automatic configuration via Git's config system
- **🗂️ VCS Metadata Filtering**: Configurable filtering of version control system metadata files
- **🛡️ Robust Error Handling**: Comprehensive error handling with meaningful messages
- **📊 Binary Content Preservation**: Intelligent handling of binary diffs and content
- **🔄 Functional Architecture**: Clean, composable design with functional programming principles

## Installation

### Homebrew (Recommended)

```bash
brew tap trontheim/diff-gitignore-filter
brew install diff-gitignore-filter
```

### Pre-compiled Binaries

Download pre-compiled binaries for your platform from the [GitHub Releases](https://github.com/trontheim/diff-gitignore-filter/releases) page.

| Platform | Architecture | Download Link |
|----------|--------------|---------------|
| **Linux** | x86_64 (AMD64) | [diff-gitignore-filter-x86_64-unknown-linux-gnu](https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-x86_64-unknown-linux-gnu) |
| **Linux** | ARM64 | [diff-gitignore-filter-aarch64-unknown-linux-gnu](https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-aarch64-unknown-linux-gnu) |
| **macOS** | x86_64 (Intel) | [diff-gitignore-filter-x86_64-apple-darwin](https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-x86_64-apple-darwin) |
| **macOS** | ARM64 (Apple Silicon) | [diff-gitignore-filter-aarch64-apple-darwin](https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-aarch64-apple-darwin) |

#### Installation Examples

```bash
# Linux x86_64
curl -L -o diff-gitignore-filter https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-x86_64-unknown-linux-gnu
chmod +x diff-gitignore-filter
sudo mv diff-gitignore-filter /usr/local/bin/

# macOS ARM64 (Apple Silicon)
curl -L -o diff-gitignore-filter https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-aarch64-apple-darwin
chmod +x diff-gitignore-filter
sudo mv diff-gitignore-filter /usr/local/bin/
```

#### Checksum Verification

Each release includes SHA256 checksums for verification:

```bash
# Download and verify checksum (Linux/macOS)
curl -L https://github.com/trontheim/diff-gitignore-filter/releases/latest/download/diff-gitignore-filter-x86_64-unknown-linux-gnu.sha256
sha256sum -c diff-gitignore-filter-x86_64-unknown-linux-gnu.sha256
```

### From Source

```bash
git clone https://github.com/trontheim/diff-gitignore-filter.git
cd diff-gitignore-filter
cargo build --release
sudo cp target/release/diff-gitignore-filter /usr/local/bin/
```

## Quick Start

```bash
# Set up as Git pager filter (recommended)
git config --global core.pager "diff-gitignore-filter"
git config --global color.diff false

# Configure downstream filter for enhanced viewing
git config --global gitignore-diff.downstream-filter "delta --side-by-side"

# Then use Git normally - diffs will be automatically filtered
git diff
git show
git log -p
```

## Usage

### Git Pager Integration

The recommended way to use `diff-gitignore-filter` is as a Git pager:

```bash
# Global setup (applies to all Git commands)
git config --global core.pager "diff-gitignore-filter"
git config --global color.diff false

# Configure downstream filter for enhanced viewing
git config --global gitignore-diff.downstream-filter "delta --side-by-side"
# Or alternatively:
git config --global gitignore-diff.downstream-filter "bat --language diff"

# Now all Git diff commands automatically filter .gitignore files
git diff                     # Filtered diff of working directory
git show HEAD                # Filtered diff of last commit
git log -p                   # Filtered diff in log
git diff --cached            # Filtered diff of staged changes
```

### Repository-specific Setup

```bash
# Set up for current repository only
git config core.pager "diff-gitignore-filter"
git config color.diff false
git config gitignore-diff.downstream-filter "delta --side-by-side"
```

### Git Aliases

For more control or temporary usage:

```bash
# Set up aliases for specific use cases
git config --global alias.idiff '!git diff --no-pager | diff-gitignore-filter --downstream "delta --side-by-side"'
git config --global alias.bdiff '!git diff --no-pager | diff-gitignore-filter --downstream "bat --language diff"'

# VCS-specific aliases
git config --global alias.vdiff '!git diff --no-pager | diff-gitignore-filter --vcs --downstream "delta"'
git config --global alias.nvdiff '!git diff --no-pager | diff-gitignore-filter --no-vcs --downstream "delta"'

# Usage
git idiff                    # Delta-enhanced filtered diff
git vdiff                    # Force VCS filtering with delta
git nvdiff                   # Disable VCS filtering with delta
```

### Manual Pipeline Usage

For manual use or integration with other tools:

```bash
# Basic pipeline usage
git diff --no-pager | diff-gitignore-filter

# With downstream filter
git diff --no-pager | diff-gitignore-filter --downstream "delta --side-by-side"

# With VCS filtering control
git diff --no-pager | diff-gitignore-filter --vcs                              # Enable VCS filtering
git diff --no-pager | diff-gitignore-filter --no-vcs                           # Disable VCS filtering

# With custom VCS patterns
git diff --no-pager | diff-gitignore-filter --vcs-pattern ".git/,.svn/"        # Git and SVN patterns
```

### CLI Options

```bash
diff-gitignore-filter --help                           # Show help
diff-gitignore-filter --version                        # Show version
diff-gitignore-filter -d "command args"                # Short form: downstream filter
diff-gitignore-filter --downstream "command args"      # Override downstream filter
diff-gitignore-filter --vcs                            # Enable VCS metadata filtering
diff-gitignore-filter --no-vcs                         # Disable VCS metadata filtering
diff-gitignore-filter --vcs-pattern "patterns"         # Specify custom VCS patterns
```

**Available Options:**
- `-d, --downstream <COMMAND>` - Pipe filtered output to downstream command
- `--vcs` - Enable VCS ignore filtering (overrides git config)
- `--no-vcs` - Disable VCS ignore filtering (overrides git config)
- `--vcs-pattern <PATTERNS>` - Custom VCS patterns (comma-separated, e.g., '.git/,.svn/')

## Configuration

### Configuration Priority

Configuration values are resolved with the following priority (highest to lowest):

1. **CLI Arguments** (highest priority)
2. **Git Configuration Values**
3. **Built-in Defaults** (lowest priority)

### Git Config Options

```bash
# Configure downstream filter
git config gitignore-diff.downstream-filter "delta --side-by-side"

# VCS filtering configuration
git config diff-gitignore-filter.vcs-ignore.enabled true
git config diff-gitignore-filter.vcs-ignore.patterns ".git/,.svn/,.hg/"

# Show configuration
git config --get gitignore-diff.downstream-filter
git config --get diff-gitignore-filter.vcs-ignore.enabled

# Remove configuration
git config --unset gitignore-diff.downstream-filter
git config --unset diff-gitignore-filter.vcs-ignore.enabled
```

### VCS Filter Configuration

The VCS filter automatically removes version control system metadata files from diffs. This feature is enabled by default and can be configured to work with any VCS system through custom patterns. The default configuration includes common VCS patterns (`.git/`, `.svn/`, `_svn/`, `.hg/`, `CVS/`, `CVSROOT/`, `.bzr/`).

#### Configuration Methods

**CLI Arguments (Highest Priority)**
```bash
# Enable/disable VCS filtering for a single command
git diff --no-pager | diff-gitignore-filter --vcs
git diff --no-pager | diff-gitignore-filter --no-vcs

# Specify custom VCS patterns
git diff --no-pager | diff-gitignore-filter --vcs-pattern ".git/,.svn/"
```

**Git Configuration (Medium Priority)**
```bash
# Global VCS filtering control
git config --global diff-gitignore-filter.vcs-ignore.enabled true
git config --global diff-gitignore-filter.vcs-ignore.patterns ".git/,.svn/,.hg/"
```

**Default Behavior (Lowest Priority)**
```bash
# VCS filtering is enabled by default - no configuration needed
# Default VCS patterns: .git/, .svn/, _svn/, .hg/, CVS/, CVSROOT/, .bzr/
```

### Environment Variables

```bash
# Debug mode
RUST_LOG=debug git diff | diff-gitignore-filter

# Trace level for detailed information
RUST_LOG=trace git diff | diff-gitignore-filter
```

## Architecture

### Core Components

The application follows a functional, layered architecture with clear separation of concerns:

#### Stream Processing Layer
- **[`filter.rs`](src/filter.rs)** - Main diff filtering with stream processing
- **[`root_finder.rs`](src/root_finder.rs)** - Git repository root detection

#### Configuration Layer
- **[`config/app_config.rs`](src/config/app_config.rs)** - High-level application configuration
- **[`config/git_config.rs`](src/config/git_config.rs)** - Git-specific configuration operations
- **[`config/git_reader.rs`](src/config/git_reader.rs)** - Low-level Git command abstraction

#### Error Handling
- **[`error.rs`](src/error.rs)** - Centralized error types and handling

### Design Principles

- **Stream Processing**: Memory-efficient line-by-line processing
- **Functional Composition**: Immutable data structures and functional pipelines
- **Error Propagation**: Comprehensive error handling with meaningful messages
- **Configuration Layering**: Clear priority system for configuration sources
- **Binary Content Preservation**: Intelligent handling of binary diffs

## Performance

### Performance Characteristics

- **Memory**: Stream-based processing with constant memory usage for text diffs
- **CPU**: Optimized pattern matching using the `ignore` crate
- **Throughput**: Efficient processing of large diffs through streaming
- **Latency**: Minimal startup time with immediate stream processing
- **Binary Handling**: Intelligent detection and preservation of binary content

### Stream Processing Benefits

- **Constant Memory**: O(1) memory usage for text processing
- **Immediate Output**: Results start streaming immediately
- **Large File Support**: Handles arbitrarily large diffs efficiently
- **Broken Pipe Handling**: Graceful handling of downstream process termination

## Development

### Requirements

- **Rust**: 1.82.0 or later (MSRV)
- **Cargo**: Latest stable version

### Setup

```bash
git clone https://github.com/trontheim/diff-gitignore-filter.git
cd diff-gitignore-filter
cargo build
```

### Available Cargo Commands

```bash
# Build and test
cargo build                                    # Debug build
cargo build --release                         # Release build
cargo test                                    # Run all tests
cargo test --lib                              # Fast library tests only
cargo bench                                   # Run benchmarks

# Code quality
cargo clippy                                   # Linting
cargo clippy -- -D warnings                   # Strict linting
cargo fmt                                     # Code formatting
cargo fmt --check                             # Check formatting

# Specific test suites
cargo test unit_tests                         # Unit tests
cargo test integration_tests                  # Integration tests
cargo test property_tests                     # Property-based tests
cargo test cli_integration_tests              # CLI integration tests

# Combined quality check
cargo test && cargo clippy -- -D warnings && cargo fmt --check
```

### Test Framework and Coverage

The project uses a comprehensive testing approach with 12 different test categories:

#### Test Categories
- **Unit Tests** ([`unit_tests.rs`](tests/unit_tests.rs)) - Individual component testing
- **Integration Tests** ([`integration_tests.rs`](tests/integration_tests.rs)) - Cross-component functionality
- **CLI Integration Tests** ([`cli_integration_tests.rs`](tests/cli_integration_tests.rs)) - End-to-end command-line testing
- **Property-Based Tests** ([`property_tests.rs`](tests/property_tests.rs)) - Edge case testing with [`proptest`](https://crates.io/crates/proptest)
- **VCS Filter Tests** ([`vcs_filter_tests.rs`](tests/vcs_filter_tests.rs)) - VCS pattern filtering
- **Worktree Tests** ([`worktree_tests.rs`](tests/worktree_tests.rs)) - Git worktree compatibility
- **Framework Tests** ([`framework_tests.rs`](tests/framework_tests.rs)) - Custom test framework validation
- **Advanced Integration Tests** ([`advanced_integration_tests.rs`](tests/advanced_integration_tests.rs)) - Complex scenarios
- **Advanced Utilities Tests** ([`advanced_utilities_tests.rs`](tests/advanced_utilities_tests.rs)) - Utility function testing
- **Downstream VCS Integration** ([`downstream_vcs_integration_test.rs`](tests/downstream_vcs_integration_test.rs)) - Downstream filter integration
- **Benchmark Tests** ([`benchmark_tests.rs`](tests/benchmark_tests.rs)) - Performance testing
- **Custom Test Framework** ([`tests/common/framework.rs`](tests/common/framework.rs)) - Specialized testing utilities

#### Coverage Analysis

```bash
# Install coverage tools
cargo install cargo-llvm-cov

# Generate HTML coverage report
cargo llvm-cov --html --open

# Coverage quality gates
cargo llvm-cov --fail-under-lines 80          # Require 80% line coverage
cargo llvm-cov --fail-under-functions 75      # Require 75% function coverage
```

### Dependencies

#### Runtime Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| [`clap`](https://crates.io/crates/clap) | 4.5 | Command-line argument parsing with derive features |
| [`ignore`](https://crates.io/crates/ignore) | 0.4 | .gitignore pattern matching |
| [`memchr`](https://crates.io/crates/memchr) | 2.7 | Fast string searching |
| [`anyhow`](https://crates.io/crates/anyhow) | 1.0 | Error handling |
| [`thiserror`](https://crates.io/crates/thiserror) | 2.0 | Error derive macros |
| [`tempfile`](https://crates.io/crates/tempfile) | 3.20 | Temporary file handling |
| [`relative-path`](https://crates.io/crates/relative-path) | 2.0 | Path manipulation |
| [`gix`](https://crates.io/crates/gix) | 0.72 | Git repository operations |

#### Development Dependencies

| Dependency | Version | Purpose |
|------------|---------|---------|
| [`assert_cmd`](https://crates.io/crates/assert_cmd) | 2.0 | CLI testing |
| [`predicates`](https://crates.io/crates/predicates) | 3.1 | Test assertions |
| [`criterion`](https://crates.io/crates/criterion) | 0.6 | Benchmarking |
| [`proptest`](https://crates.io/crates/proptest) | 1.7 | Property-based testing |
| [`quickcheck`](https://crates.io/crates/quickcheck) | 1.0 | Property-based testing |
| [`quickcheck_macros`](https://crates.io/crates/quickcheck_macros) | 1.1 | QuickCheck derive macros |
| [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) | 0.6.16 | Code coverage analysis |
| [`git-cliff`](https://crates.io/crates/git-cliff) | Latest | Changelog generation from conventional commits |

## Development Tools

This project uses:
- **git-cliff**: Changelog generation from conventional commits
- **lefthook**: Git hooks for code quality checks

### Quick Start for Contributors
```bash
cargo install git-cliff
brew install lefthook
lefthook install
```

### Release Management

This project uses a comprehensive script-based release system with intelligent version handling and rollback mechanisms.

**Standard Release Commands:**
```bash
# Patch release (bug fixes) - intelligent handling for pre-release versions
./scripts/release.sh patch

# Minor release (new features)
./scripts/release.sh minor

# Major release (breaking changes)
./scripts/release.sh major

# Dry-run to preview changes
./scripts/release.sh patch --dry-run
```

**Pre-Release Commands:**
```bash
# Create pre-release versions
./scripts/release.sh --pre-release alpha          # 0.1.0-dev → 0.1.0-alpha
./scripts/release.sh --pre-release beta           # 0.1.0-dev → 0.1.0-beta
./scripts/release.sh minor --pre-release alpha    # 0.1.0-dev → 0.2.0-alpha
./scripts/release.sh patch --pre-release rc.1     # 0.1.0-dev → 0.1.1-rc.1

# Test pre-releases
./scripts/release.sh --pre-release beta --dry-run
```

**Intelligent Version Handling:**
- **From pre-release without level**: `0.1.0-dev` + `--pre-release beta` → `0.1.0-beta` (suffix replacement)
- **From pre-release without suffix**: `0.1.0-dev` + `patch` → `0.1.0` (suffix removal)
- **Explicit level with pre-release**: `0.1.0-dev` + `minor --pre-release alpha` → `0.2.0-alpha` (increment + suffix)

**Available Options:**
- `--dry-run` - Preview changes without making them
- `--verbose` - Enable detailed logging
- `--no-push` - Skip automatic push to remote
- `--no-hooks` - Skip hook execution
- `--no-rollback` - Disable automatic rollback on errors

## Contributing

We welcome contributions! Please see [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

**Important:** This project follows [Conventional Commits](https://www.conventionalcommits.org/) for commit messages. Please ensure your commit messages follow the conventional format described in the specification.

For the complete Conventional Commits specification, visit: https://www.conventionalcommits.org/

### Quick Start for Contributors

1. Fork the repository
2. Create a feature branch: `git checkout -b feature-name`
3. Make your changes
4. Run tests: `cargo test`
5. Run linting: `cargo clippy`
6. Format code: `cargo fmt`
7. Submit a pull request

### Development Workflow

```bash
# Before committing
cargo test                          # Ensure all tests pass
cargo clippy -- -D warnings         # Check for linting issues
cargo fmt                           # Format code
# Write commit message following Conventional Commits format (https://www.conventionalcommits.org/)
```

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
