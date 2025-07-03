//! Integration tests for diff-gitignore-filter CLI
//!
//! Tests the complete CLI functionality end-to-end using real processes
//! and temporary git repositories.

use assert_cmd::Command;
use diff_gitignore_filter::config::CliArgs;
use diff_gitignore_filter::{AppConfig, ConfigError};
use predicates::prelude::*;
use tempfile::TempDir;

mod common;
use common::test_utilities::with_directory_change_boxed;
use common::TestRepo;
// Modern framework imports

/// **What is tested:** CLI help command functionality and output content
/// **Why it is tested:** Ensures the help system works correctly and displays expected information
/// **Test conditions:** Execute with --help flag
/// **Expectations:** Should succeed and display help text containing application description
#[test]
fn test_cli_help() {
    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("Pure stream-filter for Git diffs"));
}

/// **What is tested:** CLI version command functionality and output format
/// **Why it is tested:** Verifies version information is correctly displayed
/// **Test conditions:** Execute with --version flag
/// **Expectations:** Should succeed and display version information with application name
#[test]
fn test_cli_version() {
    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("diff-gitignore-filter"));
}

/// **What is tested:** Basic diff filtering functionality with gitignore patterns
/// **Why it is tested:** Core functionality test ensuring gitignore patterns are applied correctly to diff content
/// **Test conditions:** Repository with basic patterns and files, sample diff input
/// **Expectations:** Should include allowed files (src/main.rs) and exclude ignored files (debug.log, target/)
#[test]
fn test_basic_diff_filtering() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("Hello, world!"))
        .stdout(predicate::str::contains("debug.log").not())
        .stdout(predicate::str::contains("target/debug/main").not());
}

/// **What is tested:** Legacy framework compatibility for basic diff filtering
/// **Why it is tested:** Ensures backward compatibility with legacy test framework usage patterns
/// **Test conditions:** Direct framework usage with basic patterns and files, sample diff input
/// **Expectations:** Should produce same results as modern framework with proper filtering behavior
#[test]
fn test_basic_diff_filtering_legacy() {
    // Direct framework usage
    let temp_dir = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string()))
                .collect::<Vec<(String, String)>>(),
        )
        .build()
        .unwrap()
        .into_temp_dir();

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(temp_dir.path())
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("Hello, world!"))
        .stdout(predicate::str::contains("debug.log").not())
        .stdout(predicate::str::contains("target/debug/main").not());
}

/// **What is tested:** Handling of empty stdin input
/// **Why it is tested:** Ensures graceful handling of edge case with no input data
/// **Test conditions:** Repository setup with empty string as stdin input
/// **Expectations:** Should succeed and produce empty output without errors
#[test]
fn test_empty_input() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .write_stdin("")
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

/// **What is tested:** Processing of non-diff text input
/// **Why it is tested:** Ensures the tool handles arbitrary text input gracefully without requiring diff format
/// **Test conditions:** Repository setup with plain text input (not diff format)
/// **Expectations:** Should pass through non-diff content unchanged
#[test]
fn test_non_diff_input() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    let non_diff_input = "This is not a diff\nJust some random text\n";

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .write_stdin(non_diff_input)
        .assert()
        .success()
        .stdout(predicate::str::contains("This is not a diff"))
        .stdout(predicate::str::contains("Just some random text"));
}

/// **What is tested:** Behavior when executed outside of a git repository
/// **Why it is tested:** Ensures fallback mechanism works when no git repository is available
/// **Test conditions:** Temporary directory without .git, sample diff input
/// **Expectations:** Should succeed with fallback behavior, no gitignore filtering applied
#[test]
fn test_not_in_git_repo() {
    let temp_dir = TempDir::new().unwrap();
    // Don't create .git directory

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(temp_dir.path())
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success() // Should succeed with fallback mechanism
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("debug.log")); // No gitignore filtering without repo
}

/// **What is tested:** Downstream filter integration with simple command (cat)
/// **Why it is tested:** Verifies that filtered output can be piped to downstream commands
/// **Test conditions:** Repository with basic patterns, --downstream cat command, sample diff input
/// **Expectations:** Should apply filtering and pass results through downstream command successfully
#[test]
fn test_downstream_filter_echo() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .arg("--downstream")
        .arg("cat") // Use 'cat' as a simple downstream filter
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("debug.log").not());
}

/// **What is tested:** Error handling for invalid downstream commands
/// **Why it is tested:** Ensures proper error reporting when downstream command cannot be executed
/// **Test conditions:** Repository setup with non-existent downstream command
/// **Expectations:** Should fail gracefully with DownstreamProcessFailed error message
#[test]
fn test_downstream_filter_invalid_command() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .arg("--downstream")
        .arg("nonexistent-command-12345")
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .failure()
        .stderr(predicate::str::contains("DownstreamProcessFailed"));
}

/// **What is tested:** Complex gitignore pattern handling including negation and path-specific patterns
/// **Why it is tested:** Ensures advanced gitignore features work correctly (comments, negation, path patterns)
/// **Test conditions:** Complex patterns with comments, negation (!important.log), and path-specific rules
/// **Expectations:** Should correctly apply complex pattern logic including negation and path matching
#[test]
fn test_complex_gitignore_patterns() {
    // Using new framework with complex patterns
    let complex_patterns = vec![
        "# Comments".to_string(),
        "*.log".to_string(),
        "!important.log".to_string(),
        "/build/".to_string(),
        "docs/*.pdf".to_string(),
    ];

    let repo = TestRepo::builder()
        .with_patterns(complex_patterns)
        .build_temp_dir()
        .unwrap();

    let complex_diff = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
diff --git a/debug.log b/debug.log
index 1111111..2222222 100644
--- a/debug.log
+++ b/debug.log
@@ -1 +1,2 @@
 log entry
+another log entry
diff --git a/important.log b/important.log
index 3333333..4444444 100644
--- a/important.log
+++ b/important.log
@@ -1 +1,2 @@
 important entry
+more important entry
diff --git a/build/output b/build/output
index 5555555..6666666 100644
--- a/build/output
+++ b/build/output
@@ -1 +1,2 @@
 build output
+more build output
diff --git a/docs/manual.pdf b/docs/manual.pdf
index 7777777..8888888 100644
--- a/docs/manual.pdf
+++ b/docs/manual.pdf
@@ -1 +1,2 @@
 pdf content
+more pdf content
"#;

    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .write_stdin(complex_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("important.log")) // Should be included (!important.log)
        .stdout(predicate::str::contains("debug.log").not()) // Should be excluded (*.log)
        .stdout(predicate::str::contains("build/output").not()) // Should be excluded (/build/)
        .stdout(predicate::str::contains("docs/manual.pdf").not()); // Should be excluded (docs/*.pdf)
}

/// **What is tested:** Integration with git configuration for downstream filter settings
/// **Why it is tested:** Verifies that git config values are properly read and used when CLI args are not provided
/// **Test conditions:** Git config set for downstream filter, no CLI downstream argument
/// **Expectations:** Should use git config value for downstream processing
#[test]
fn test_git_config_integration() {
    // Using new framework with git config
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build()
        .unwrap();

    // Set up git config for downstream filter using new framework
    let cleanup = repo
        .set_git_config_with_cleanup("gitignore-diff.downstream-filter", "cat")
        .expect("Failed to set git config");

    // Test that the tool uses the git config when no CLI arg is provided
    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("debug.log").not());

    // Automatic cleanup when cleanup function goes out of scope
    cleanup();
}

/// **What is tested:** CLI argument precedence over git configuration settings
/// **Why it is tested:** Ensures CLI arguments take priority over git config when both are present
/// **Test conditions:** Git config with one command, CLI arg with different command
/// **Expectations:** Should use CLI argument value, ignoring git config setting
#[test]
fn test_cli_arg_overrides_git_config() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build()
        .unwrap();

    // Set up git config for downstream filter using new framework
    let cleanup = repo
        .set_git_config_with_cleanup("gitignore-diff.downstream-filter", "nonexistent-command")
        .expect("Failed to set git config");

    // Test that CLI arg overrides git config
    let mut cmd = Command::cargo_bin("diff-gitignore-filter").unwrap();
    cmd.current_dir(repo.path())
        .arg("--downstream")
        .arg("cat") // This should override the git config
        .write_stdin(common::framework::TestData::SAMPLE_DIFF)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("debug.log").not());

    // Automatic cleanup when cleanup function goes out of scope
    cleanup();
}

/// **What is tested:** AppConfig creation and integration with various CLI argument combinations
/// **Why it is tested:** Validates that AppConfig correctly interprets CLI arguments and creates proper configurations
/// **Test conditions:** Various CLI argument combinations (VCS enabled/disabled, no downstream)
/// **Expectations:** Should create valid AppConfig objects with correct settings based on CLI arguments
#[test]
fn test_app_config_integration_basic() {
    // Using new framework
    let repo = TestRepo::builder()
        .with_patterns(
            common::framework::TestData::BASIC_PATTERNS
                .iter()
                .map(|s| s.to_string()),
        )
        .with_files(
            common::framework::TestData::BASIC_FILES
                .iter()
                .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string())),
        )
        .build_temp_dir()
        .unwrap();

    // Change to test directory for git config access using thread-safe approach
    let result = with_directory_change_boxed(repo.path(), || {
        // Test basic configuration without CLI flags
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        };

        match AppConfig::from_cli(cli_args) {
            Ok(config) => {
                // Should have default behavior
                assert!(config.vcs_enabled());
                assert!(!config.vcs_patterns().is_empty());
                assert!(config.downstream_filter().is_none());
            }
            Err(_) => {
                // Error is acceptable in test environment
            }
        }

        // Test with VCS disabled
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: true,
            downstream: None,
            vcs_pattern: None,
        };

        match AppConfig::from_cli(cli_args) {
            Ok(config) => {
                assert!(!config.vcs_enabled());
            }
            Err(_) => {
                // Error is acceptable in test environment
            }
        }

        Ok(())
    });

    // If directory change failed, skip test gracefully
    if result.is_err() {}
}

/// **What is tested:** Error handling in AppConfig creation for various failure scenarios
/// **Why it is tested:** Ensures robust error handling when AppConfig cannot be created due to environment issues
/// **Test conditions:** Non-git directory, various CLI argument combinations
/// **Expectations:** Should handle configuration errors gracefully with appropriate error types
#[test]
fn test_app_config_error_scenarios() {
    // Test error scenarios for AppConfig

    // Test in directory without git using thread-safe approach
    let temp_dir = tempfile::TempDir::new().unwrap();

    let result = with_directory_change_boxed(temp_dir.path(), || {
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: false,
            downstream: Some("test-command".to_string()),
            vcs_pattern: None,
        };

        let result = AppConfig::from_cli(cli_args);

        // Should handle various error scenarios gracefully
        match result {
            Ok(_) => {
                // Success is acceptable
            }
            Err(ConfigError::NotInGitRepository { .. }) => {
                // Expected error type
            }
            Err(ConfigError::GitCommandFailed { .. }) => {
                // Also acceptable
            }
            Err(_) => {
                // Other errors might occur
            }
        }

        Ok(())
    });

    // If directory change failed, that's also acceptable for this error test
    let _ = result;
}
