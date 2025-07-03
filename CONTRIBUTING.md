# Contributing to diff-gitignore-filter

Thank you for your interest in contributing to this project! This guide will help you contribute effectively to the project.

## Code of Conduct

This project adheres to the Rust Code of Conduct. Please read [the full text](https://www.rust-lang.org/policies/code-of-conduct) so that you can understand what actions will and will not be tolerated.

## Issue Guidelines

### Bug Reports

When reporting bugs, please include:

- **Environment**: OS, Rust version, diff-gitignore-filter version
- **Reproduction Steps**: Clear steps to reproduce the issue
- **Expected Behavior**: What you expected to happen
- **Actual Behavior**: What actually happened
- **Sample Data**: If possible, include a minimal diff or .gitignore that reproduces the issue

### Feature Requests

For feature requests, please provide:

- **Use Case**: Describe the problem you're trying to solve
- **Proposed Solution**: Your idea for how to solve it
- **Alternatives**: Other solutions you've considered
- **Impact**: How this would benefit other users

### Security Issues

For security vulnerabilities, please **do not** create a public issue. Instead, email the maintainers privately to report security concerns.

## Development Setup

### Prerequisites

Make sure the following tools are installed:

- **Rust** (1.82.0 or higher - see MSRV section below)
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  rustup update
  ```

- **cargo-llvm-cov** for coverage analysis
  ```bash
  cargo install cargo-llvm-cov
  ```

- **cargo-criterion** for performance benchmarks
  ```bash
  cargo install cargo-criterion
  ```

- **git-cliff** for changelog generation
  ```bash
  cargo install git-cliff
  ```

- **rustfmt** and **clippy** for code quality
  ```bash
  rustup component add rustfmt clippy
  ```

### Installation

1. Clone the repository:
   ```bash
   git clone https://github.com/trontheim/diff-gitignore-filter.git
   cd diff-gitignore-filter
   ```

2. Install dependencies:
   ```bash
   cargo build
   ```

3. Run tests to verify setup:
   ```bash
   cargo test
   ```

### MSRV (Minimum Supported Rust Version)

This project maintains a **Minimum Supported Rust Version (MSRV) of 1.82.0**. The MSRV is the oldest version of Rust that the project is guaranteed to compile with.

**Why MSRV matters:**
- Ensures compatibility with older Rust installations
- Prevents accidental use of newer language features
- Provides stability guarantees for downstream users

**Testing MSRV:**
```bash
# Install the MSRV version
rustup install 1.82.0

# Test with MSRV
cargo +1.82.0 check
cargo +1.82.0 test
```

**MSRV Policy:**
- MSRV bumps are considered breaking changes
- MSRV will only be increased when necessary for important features or security
- MSRV changes will be clearly documented in CHANGELOG.md

## Architecture Overview

### Core Design Philosophy

This project follows a **functional, layered architecture** with clear module separation and a **stream-processing pipeline** as the core concept. The design emphasizes:

- **Stream Processing**: Efficient handling of Git diff streams without loading entire files into memory
- **Configuration Hierarchy**: Git-integrated configuration system with priority-based settings
- **Modular Design**: Clear separation of concerns across functional modules
- **Error Handling**: Comprehensive error types with meaningful messages

### Core Components

- **Filter**: Main stream processing logic with .gitignore pattern matching
- **RootFinder**: Intelligent Git repository root detection with worktree support
- **Config**: Hierarchical configuration system with Git integration
- **Error**: Comprehensive error handling with structured error types

### Key Modules

- **`src/filter.rs`**: Core filtering logic with stream processing pipeline
- **`src/root_finder.rs`**: Git repository root detection with worktree support
- **`src/config/`**: Configuration management and Git config integration
- **`src/error.rs`**: Comprehensive error handling with meaningful messages

### Stream Processing Pipeline

The core filtering operates as a stream processing pipeline:

1. **Input Stream**: Git diff data received via stdin or file
2. **Pattern Matching**: .gitignore patterns applied to file paths
3. **VCS Filtering**: Optional VCS-specific pattern filtering
4. **Output Stream**: Filtered diff data sent to stdout or downstream command

## Development Workflow with cocogitto + lefthook

### Setup for New Developers
```bash
# 1. Install tools
cargo install git-cliff
brew install lefthook  # or appropriate for your OS

# 2. Activate hooks
lefthook install

# 3. Optional: Git template for reference
git config commit.template .gitmessage
```

### Modern Cargo Aliases

This project includes modern Cargo aliases for efficient development. Use these commands from the README:

#### Quick Development
```bash
# Fast syntax and type checking
cargo check-fast

# Fast unit and binary tests only
cargo test-fast
```

#### Code Quality
```bash
# Run clippy with strict warnings
cargo lint

# Check code formatting
cargo fmt-check
```

#### Coverage Analysis
```bash
# Generate HTML coverage report
cargo cov

# Generate and open HTML coverage report
cargo cov-html

# Show coverage summary only
cargo cov-summary

# Generate LCOV format for CI/CD
cargo cov-lcov

# Test-specific coverage
cargo test-cov                      # All tests with HTML report
cargo unit-cov                      # Unit tests coverage only
cargo integration-cov               # Integration tests coverage only
cargo property-cov                  # Property-based tests coverage only

# Coverage quality gates
cargo cov-check                     # Fail if coverage below 85%
cargo cov-strict                    # Fail if coverage below 90%
```

#### Benchmarking
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark suite
cargo bench --bench performance

# Benchmark with coverage
cargo bench-cov
```

### Traditional Commands

Before committing changes, perform the following steps:

1. **Format code:**
   ```bash
   cargo fmt
   ```

2. **Run linting:**
   ```bash
   cargo clippy -- -D warnings
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Run property tests:**
   ```bash
   cargo test --test property_tests
   ```

### Coverage Analysis

#### Quick Coverage Check
For a quick overview of test coverage:
```bash
cargo llvm-cov
# Or use the modern alias:
cargo cov-summary
```

#### Detailed HTML Report
For a detailed coverage report:
```bash
cargo llvm-cov --html
# Or use the modern alias:
cargo cov-html
```
The report will be generated in `target/llvm-cov/html/index.html`.

#### Coverage for Specific Modules
```bash
# Only for lib.rs
cargo llvm-cov --lib

# Only for integration tests
cargo llvm-cov --test integration_tests

# Only for unit tests
cargo llvm-cov --test unit_tests
```

#### Quality Gates
Check minimum coverage requirements:
```bash
# Check overall coverage (minimum 85%)
cargo llvm-cov --fail-under-lines 85
# Or use the modern alias:
cargo cov-check

# Check critical modules (minimum 90%)
cargo llvm-cov --lib --fail-under-lines 90
# Or use the modern alias:
cargo cov-strict
```

### Performance Testing

#### Running Benchmarks
```bash
# All benchmarks
cargo bench

# Specific benchmark group
cargo bench --bench performance

# With detailed output
cargo bench -- --verbose
```

#### Profiling
For performance analysis with profiling:
```bash
# Debug build for profiling
cargo build --profile dev

# With flamegraph (if installed)
cargo flamegraph --bench performance
```

## Dependency Management

### Security Auditing

```bash
# Install cargo-audit
cargo install cargo-audit

# Run security audit
cargo audit

# Fix security issues automatically (when possible)
cargo audit fix
```

### Dependency Updates

```bash
# Install cargo-outdated
cargo install cargo-outdated

# Check for outdated dependencies
cargo outdated

# Update dependencies (be careful with breaking changes)
cargo update
```

### Policy for Breaking Changes

- **Major version updates**: Require explicit approval and testing
- **Security updates**: Should be applied promptly, even if they introduce breaking changes
- **MSRV impact**: Dependency updates that increase MSRV require discussion

### Current Dependencies

#### Production Dependencies

| Crate | Version | Purpose | Features |
|-------|---------|---------|----------|
| [`clap`](https://crates.io/crates/clap) | 4.5 | CLI parsing | derive |
| [`ignore`](https://crates.io/crates/ignore) | 0.4 | .gitignore pattern matching | Core functionality |
| [`memchr`](https://crates.io/crates/memchr) | 2.7 | SIMD-accelerated string search | Performance |
| [`anyhow`](https://crates.io/crates/anyhow) | 1.0 | Error handling | Flexible error types |
| [`thiserror`](https://crates.io/crates/thiserror) | 2.0 | Error handling | Derive macros |
| [`tempfile`](https://crates.io/crates/tempfile) | 3.20 | Temporary file management | Testing support |
| [`relative-path`](https://crates.io/crates/relative-path) | 2.0 | Platform-agnostic path handling | Path operations |
| [`gix`](https://crates.io/crates/gix) | 0.72 | Git repository operations | Git integration |

#### Development Dependencies

| Crate | Version | Purpose | Features |
|-------|---------|---------|----------|
| [`assert_cmd`](https://crates.io/crates/assert_cmd) | 2.0 | CLI testing | Integration tests |
| [`predicates`](https://crates.io/crates/predicates) | 3.1 | Test assertions | Test utilities |
| [`criterion`](https://crates.io/crates/criterion) | 0.6 | Benchmarking | Performance testing |
| [`proptest`](https://crates.io/crates/proptest) | 1.7 | Property-based testing | Fuzzing |
| [`quickcheck`](https://crates.io/crates/quickcheck) | 1.0 | Property-based testing | Random testing |
| [`quickcheck_macros`](https://crates.io/crates/quickcheck_macros) | 1.1 | QuickCheck derive macros | Test generation |
| [`cargo-llvm-cov`](https://crates.io/crates/cargo-llvm-cov) | Latest | Code coverage analysis | Coverage reporting |
| [`git-cliff`](https://crates.io/crates/git-cliff) | Latest | Changelog generation | Release automation |

## Documentation Standards

### Rust Doc Comments

Follow these best practices for documentation:

#### Classes and Interfaces
```rust
/// Represents a Git diff filter that respects .gitignore patterns.
///
/// This struct provides the core functionality for filtering Git diffs
/// based on .gitignore rules with stream processing optimization.
pub struct Filter {
    // ...
}
```

#### Methods and Functions
```rust
/// Filters a Git diff stream, excluding files that match .gitignore patterns.
///
/// # Arguments
///
/// * `reader` - The input stream containing the Git diff
/// * `writer` - The output stream for the filtered diff
///
/// # Returns
///
/// Returns `Ok(())` on success, or an error if filtering fails.
///
/// # Examples
///
/// ```rust
/// use diff_gitignore_filter::Filter;
/// use std::io::Cursor;
///
/// let filter = Filter::new(".")?;
/// let input = "diff --git a/ignored.log b/ignored.log\n";
/// let mut output = Vec::new();
///
/// filter.process_diff(Cursor::new(input), &mut output)?;
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
pub fn process_diff<R: BufRead, W: Write>(&self, reader: R, writer: W) -> Result<()> {
    // ...
}
```

#### Error Handling
```rust
/// Represents errors that can occur during diff filtering.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Failed to find Git repository root
    #[error("Could not locate Git repository root: {0}")]
    RepositoryNotFound(String),

    /// I/O error during processing
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}
```

#### Complex Logic
```rust
// Use SIMD-optimized memchr for fast pattern recognition
// This provides significant performance improvements over naive string searching
let pattern_start = memchr::memchr(b'/', path_bytes)
    .unwrap_or(path_bytes.len());
```

### Documentation Generation

```bash
# Generate documentation
cargo doc --no-deps --open

# Test documentation examples
cargo test --doc

# Check for broken links
cargo doc --no-deps 2>&1 | grep -i warning
```

## Coverage Standards

### Overall Requirements

- **Minimum Coverage:** 85% for the entire project
- **Critical Modules:** 90% coverage required for:
  - `src/lib.rs`
  - `src/filter.rs`
  - `src/config/`
- **New Features:** Must have 100% coverage
- **Bug Fixes:** Must include tests for the fixed case

### Test Categories

The project includes **12 comprehensive test categories** with a custom test framework:

#### 1. Unit Tests (`tests/unit_tests.rs`)
- Test individual functions and methods in isolation
- Focus on edge cases and error handling
- Fast execution (< 1ms per test)

#### 2. Integration Tests (`tests/integration_tests.rs`)
- Test interaction between modules
- End-to-end scenarios with realistic datasets

#### 3. Advanced Integration Tests (`tests/advanced_integration_tests.rs`)
- Complex integration scenarios
- Multi-component interaction testing

#### 4. CLI Integration Tests (`tests/cli_integration_tests.rs`)
- Command-line interface testing
- Argument parsing and output validation

#### 5. Property Tests (`tests/property_tests.rs`)
- Generative tests with QuickCheck/proptest
- Test properties across many inputs
- Especially important for parser and filter logic

#### 6. VCS Filter Tests (`tests/vcs_filter_tests.rs`)
- Version control system specific filtering
- Git integration testing

#### 7. Worktree Tests (`tests/worktree_tests.rs`)
- Git worktree support validation
- Multi-worktree scenarios

#### 8. Downstream VCS Integration Tests (`tests/downstream_vcs_integration_test.rs`)
- Integration with downstream VCS tools
- Pipeline processing validation

#### 9. Advanced Utilities Tests (`tests/advanced_utilities_tests.rs`)
- Utility function testing
- Helper module validation

#### 10. Framework Tests (`tests/framework_tests.rs`)
- Custom test framework validation
- Test infrastructure testing

#### 11. Benchmark Tests (`tests/benchmark_tests.rs`)
- Performance regression tests
- Memory usage monitoring
- Throughput measurements

#### 12. Framework Performance Tests (`tests/benchmarks/framework_performance.rs`)
- Test framework performance validation
- Benchmark infrastructure testing

### Custom Test Framework

The project includes a **custom test framework with scenario builder** located in `tests/common/framework.rs` that provides:

- **Scenario Builder Pattern**: Fluent API for test case construction
- **Test Utilities**: Common testing helpers and fixtures
- **Performance Testing**: Integrated benchmarking capabilities
- **Fixture Management**: Reusable test data and configurations

### Coverage Commands Reference

```bash
# Basic coverage report
cargo llvm-cov

# Generate HTML report
cargo llvm-cov --html

# Coverage with fail threshold
cargo llvm-cov --fail-under-lines 85

# Coverage for specific tests
cargo llvm-cov --test unit_tests
cargo llvm-cov --test integration_tests
cargo llvm-cov --test property_tests

# Coverage without tests (only documented examples)
cargo llvm-cov --doctests

# Coverage with JSON output for CI
cargo llvm-cov --json --output-path coverage.json

# Coverage with LCOV format for external tools
cargo llvm-cov --lcov --output-path coverage.lcov

# Detailed coverage per file
cargo llvm-cov --summary-only

# Coverage with branch coverage
cargo llvm-cov --branch
```

## Commit Message Guidelines

### Conventional Commits Format

This project follows the [Conventional Commits](https://www.conventionalcommits.org/) specification. All commits must use this format:

```
<type>[optional scope]: <description>

[optional body]

[optional footer(s)]
```

### Setup Instructions for New Developers

Configure the provided commit message template to get helpful guidelines when committing:

```bash
# Set up the commit message template (one-time setup)
git config commit.template .gitmessage

# Now when you commit, the template will be pre-filled with guidelines
git commit
```

### Available Commit Types

Based on the project's `.gitmessage` template, the following types are available:

#### Standard Types
- **feat**: A new feature
- **fix**: A bug fix
- **docs**: Documentation only changes
- **style**: Changes that do not affect the meaning of the code (white-space, formatting, etc.)
- **refactor**: A code change that neither fixes a bug nor adds a feature
- **perf**: A code change that improves performance
- **test**: Adding missing tests or correcting existing tests
- **chore**: Changes to the build process or auxiliary tools
- **ci**: Changes to CI configuration files and scripts
- **build**: Changes that affect the build system or external dependencies
- **revert**: Reverts a previous commit

#### Additional Types
- **security**: Security-related changes
- **breaking**: Explicit breaking changes

### Project Scopes

The following scopes are defined for this project:

#### Core Scopes
- **filter**: Core filtering logic
- **root-finder**: Git repository root detection
- **config**: Configuration management
- **cli**: Command-line interface
- **error**: Error handling
- **bench**: Benchmarking code
- **deps**: Dependency updates

#### Additional Scopes
- **integration**: Integration features
- **vcs**: Version control system features
- **unicode**: Unicode handling
- **streaming**: Stream processing
- **simd**: SIMD optimizations

### Breaking Changes

Breaking changes can be indicated using these methods:

#### Method 1: Exclamation Mark Notation
```bash
feat(cli)!: change default output format to JSON
```

#### Method 2: BREAKING CHANGE Footer
```bash
feat(config): add new configuration options

BREAKING CHANGE: Configuration file format changed
```

#### Method 3: Explicit Breaking Type
```bash
breaking(api): remove deprecated filter methods
```

### Examples from .gitmessage Template

The following examples are provided in the project's commit message template:

```bash
# Feature and fixes
feat(filter): add SIMD-optimized pattern matching
fix(root-finder): handle Git worktrees correctly
docs(readme): update installation instructions

# Security and breaking changes
security(config): validate user input to prevent path traversal
breaking(cli): remove deprecated --legacy-mode flag

# Breaking change examples
breaking(api): remove deprecated filter methods
feat(cli)!: change default output format to JSON
```

For more information, see the [Conventional Commits specification](https://www.conventionalcommits.org/).

## Pull Request Process

### Code Quality Checks

Before creating a pull request:

1. **Check formatting:**
   ```bash
   cargo fmt --check
   # Or use the modern alias:
   cargo fmt-check
   ```

2. **Linting without warnings:**
   ```bash
   cargo clippy -- -D warnings
   # Or use the modern alias:
   cargo lint
   ```

3. **All tests pass:**
   ```bash
   cargo test --all-features
   # Or use the fast alias for development:
   cargo test-fast
   ```

4. **Documentation tests:**
   ```bash
   cargo test --doc
   ```

### Coverage Requirements

1. **Maintain minimum coverage:**
   ```bash
   cargo llvm-cov --fail-under-lines 85
   # Or use the modern alias:
   cargo cov-check
   ```

2. **Critical modules coverage:**
   ```bash
   cargo llvm-cov --lib --fail-under-lines 90
   # Or use the modern alias:
   cargo cov-strict
   ```

3. **New code coverage:**
   - New code must have 100% coverage
   - Use `cargo cov-html` to identify untested areas

### Performance Validation

1. **Benchmark regression check:**
   ```bash
   cargo bench
   ```

2. **Don't degrade performance:**
   - Compare benchmark results with main branch
   - Document performance changes in the PR

### Documentation Updates

1. **API documentation:**
   ```bash
   cargo doc --no-deps --open
   ```

2. **README updates:** If public API was changed

3. **CHANGELOG.md:** Entry for breaking changes or new features

4. **Coverage documentation:** Update `docs/coverage.md` for coverage changes

## CI/CD Pipeline

### GitHub Actions Workflow

Our CI/CD pipeline performs the following checks:

#### Code Quality Stage
```yaml
- name: Format Check
  run: cargo fmt --check

- name: Clippy Linting
  run: cargo clippy -- -D warnings

- name: Security Audit
  run: cargo audit
```

#### Testing Stage
```yaml
- name: Unit Tests
  run: cargo test --test unit_tests

- name: Integration Tests
  run: cargo test --test integration_tests

- name: Property Tests
  run: cargo test --test property_tests

- name: Documentation Tests
  run: cargo test --doc
```

#### Coverage Stage
```yaml
- name: Generate Coverage
  run: cargo llvm-cov --lcov --output-path coverage.lcov

- name: Upload to Codecov
  uses: codecov/codecov-action@v3
  with:
    file: coverage.lcov

- name: Coverage Gate
  run: cargo llvm-cov --fail-under-lines 85
```

#### Performance Stage
```yaml
- name: Benchmark Tests
  run: cargo bench --bench performance

- name: Performance Regression Check
  run: |
    cargo bench --bench performance -- --save-baseline current
    # Compare with previous baseline
```

### Coverage Reporting

- **Codecov Integration:** Automatic coverage reports for every PR
- **Coverage Trends:** Tracking coverage development over time
- **Branch Coverage:** In addition to line coverage, branch coverage is measured
- **Differential Coverage:** Only new/changed lines are evaluated for PR coverage

### Quality Gates

The pipeline fails if:
- Coverage falls below 85%
- Critical modules have less than 90% coverage
- Clippy warnings are present
- Tests fail
- Performance degrades by more than 10%

### Deployment

After successful merge into main:
- Automatic creation of release candidates with **git-cliff**
- Benchmark results are saved as baseline
- Documentation is automatically updated
- Release pipeline with available scripts in `scripts/`

## Local Testing with act

You can test GitHub Actions locally with [act](https://github.com/nektos/act) to validate workflows without running them in GitHub.

### Installation

**macOS:**
```bash
brew install act
```

**Linux:**
```bash
curl https://raw.githubusercontent.com/nektos/act/master/install.sh | sudo bash
```

**Windows:**
```bash
choco install act-cli
```

### Quick Start

```bash
# Create secrets file (optional)
cp .secrets.dist .secrets
# Edit .secrets with your values

# Show all jobs
act --list

# Run push event (default)
act

# Run specific job
act -j lefthook-checks

# Full test suite
act -j test

# With secrets file
act --secret-file .secrets
```

### Common Commands

#### Testing Individual Jobs

```bash
# Lefthook checks (fast)
act -j lefthook-checks

# Test matrix (all Rust versions)
act -j test

# MSRV test
act -j msrv

# Code coverage
act -j coverage

# Security audit
act -j security

# Cross-platform tests
act -j cross-platform-test

# Documentation
act -j docs

# Benchmarks
act -j benchmarks
```

#### Testing Workflows

```bash
# Automated Tests Workflow (default push event)
act -W .github/workflows/automated-tests.yml

# Release Workflow (with workflow_dispatch)
act workflow_dispatch -W .github/workflows/release.yml

# Simulate pull request
act pull_request
```

#### Debug Options

```bash
# Verbose mode
act -v

# Dry run (shows commands without execution)
act -n

# Specific platform
act -P ubuntu-latest=catthehacker/ubuntu:full-22.04
```

### Configuration

#### .actrc
The `.actrc` file is already configured with:
- Ubuntu containers for all platforms
- Apple Silicon compatibility (`linux/amd64`)

#### Secrets
```bash
# Create secrets file
cp .secrets.dist .secrets

# Minimal configuration
RUST_LOG=info
CARGO_TERM_COLOR=always
RUST_BACKTRACE=1

# Optional for GitHub integration
GITHUB_TOKEN=your_token_here
CODECOV_TOKEN=your_codecov_token_here
```

### Troubleshooting

#### Apple Silicon (M1/M2)
```bash
# If containers won't start
act --container-architecture linux/amd64
```

#### Cache Issues
```bash
# Clear Docker cache if needed
docker system prune
```

#### Memory Issues
```bash
# Adjust container limits in .actrc
--container-options="--memory=2g --cpus=1"
```

### Tips

1. **Fast Tests**: Start with `lefthook-checks` for quick validation
2. **Selective Testing**: Test only relevant jobs for fast iteration
3. **Docker Cache**: Docker reuses containers for better performance
4. **Secrets Security**: Never commit `.secrets`

### Example Workflow

```bash
# Before a commit
act -j lefthook-checks

# Before a pull request
act pull_request -j test
act -j coverage

# Release preparation
act -j cross-platform-test
act workflow_dispatch -W .github/workflows/release.yml
```

## Help and Support

- **Issues:** For bugs and feature requests
- **Discussions:** For questions and general discussions
- **Documentation:** See `docs/` directory for detailed documentation
- **Coverage Reports:** Current coverage reports under `target/llvm-cov/html/`

## License

By contributing, you agree that your contributions will be licensed under the same license as the project.
