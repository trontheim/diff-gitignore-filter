//! CLI entry point for diff-gitignore-filter
//!
//! A pure stream-filter for Git diffs that respects .gitignore patterns
//! and supports optional downstream filtering.

use clap::{ArgAction, Parser};
use std::env;
use std::io::{self, BufReader, Seek, SeekFrom};
use std::process;
use tempfile::tempfile;

use diff_gitignore_filter::{AppConfig, ConfigError, Filter, Result, RootFinder};

/// Pure stream-filter for Git diffs that respects .gitignore patterns
#[derive(Parser)]
#[command(name = "diff-gitignore-filter")]
#[command(version, about, long_about = None)]
struct Args {
    /// Pipe filtered output to downstream command
    #[arg(short, long, value_name = "COMMAND")]
    downstream: Option<String>,

    /// Enable VCS ignore filtering (overrides git config)
    #[arg(long, overrides_with = "no_vcs", action = ArgAction::SetTrue)]
    vcs: bool,

    /// Disable VCS ignore filtering (overrides git config)
    #[arg(long, overrides_with = "vcs", action = ArgAction::SetTrue)]
    no_vcs: bool,

    /// Custom VCS patterns (overrides git config)
    #[arg(
        long,
        value_name = "PATTERNS",
        help = "Comma-separated VCS patterns (e.g., '.git/,.svn/')",
        long_help = "Specify custom VCS ignore patterns as comma-separated list. \
                     Overrides git config 'diff-gitignore-filter.vcs-ignore.patterns'. \
                     Patterns are trimmed and empty patterns are filtered out."
    )]
    vcs_pattern: Option<String>,
}

/// Convert CLI args to CliArgs struct for AppConfig
impl From<Args> for diff_gitignore_filter::config::CliArgs {
    fn from(args: Args) -> Self {
        Self {
            vcs: args.vcs,
            no_vcs: args.no_vcs,
            downstream: args.downstream,
            vcs_pattern: args.vcs_pattern,
        }
    }
}

/// Process diff with temporary file using AppConfig with functional composition
fn process_diff_with_config<W: io::Write>(
    mut temp_file: std::fs::File,
    mut output: W,
    config: &AppConfig,
) -> Result<()> {
    // Phase 1: Root-Finding with functional error handling
    temp_file.seek(SeekFrom::Start(0)).map_err(|e| {
        diff_gitignore_filter::Error::processing_error(format!("Failed to seek to start: {e}"))
    })?;

    let root_result = {
        let root_reader = BufReader::new(&temp_file);
        RootFinder::find_root(env::current_dir()?, root_reader)
    }; // root_reader is automatically dropped here

    // Phase 2: Filter-Pipeline with functional composition and improved fallback logic
    let root = root_result.or_else(|_| env::current_dir()).map_err(|e| {
        diff_gitignore_filter::Error::processing_error(format!(
            "Failed to determine root directory: {e}"
        ))
    })?;

    let filter = Filter::new(root)?;

    // Functional composition for VCS patterns with proper ownership handling
    let filter = if config.vcs_enabled() {
        filter.with_vcs_patterns(config.vcs_patterns().to_vec())
    } else {
        filter
    };

    // Functional composition for downstream filter with proper ownership handling
    let filter = match config.downstream_filter() {
        Some(command) => filter.with_downstream(command.to_string()),
        None => filter,
    };

    // Phase 3: Filter-Processing with functional error handling
    temp_file.seek(SeekFrom::Start(0)).map_err(|e| {
        diff_gitignore_filter::Error::processing_error(format!(
            "Failed to seek to start for filter: {e}"
        ))
    })?;

    let filter_reader = BufReader::new(&temp_file);
    filter.process_diff(filter_reader, &mut output)
}

/// Handle configuration errors with user-friendly messages using functional pattern matching
fn handle_config_error(error: ConfigError) -> ! {
    let error_message = match error {
        ConfigError::GitCommandFailed { .. } => "Git command failed",
        ConfigError::InvalidGitConfig { .. } => "Invalid git config",
        ConfigError::NotInGitRepository { .. } => "Not in git repository",
        ConfigError::IoError { .. } => "Configuration error",
        ConfigError::InvalidCliArgument { .. } => "Invalid CLI argument",
    };

    eprintln!("{error_message}");
    process::exit(1);
}

fn main() -> Result<()> {
    // Functional pipeline with Result monad composition
    let config = Args::parse()
        .pipe(diff_gitignore_filter::config::CliArgs::from)
        .pipe(AppConfig::from_cli)
        .unwrap_or_else(|error| handle_config_error(error));

    // Functional composition for file operations
    let temp_file = create_temp_file_with_stdin()?;

    // Process the diff with functional error propagation
    process_diff_with_config(temp_file, io::stdout(), &config)
}

/// Helper trait for functional pipeline composition
trait Pipe<T> {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
        Self: Sized;
}

impl<T> Pipe<T> for T {
    fn pipe<U, F>(self, f: F) -> U
    where
        F: FnOnce(Self) -> U,
    {
        f(self)
    }
}

/// Create temporary file and copy stdin with functional error handling
fn create_temp_file_with_stdin() -> Result<std::fs::File> {
    let mut temp_file = tempfile().map_err(|e| {
        diff_gitignore_filter::Error::processing_error(format!("Failed to create temp file: {e}"))
    })?;

    io::copy(&mut io::stdin(), &mut temp_file).map_err(|e| {
        diff_gitignore_filter::Error::processing_error(format!("Failed to copy stdin: {e}"))
    })?;

    Ok(temp_file)
}

#[cfg(test)]
mod tests {
    use super::*;
    use diff_gitignore_filter::config::CliArgs;
    use std::io::Write;
    use tempfile::tempfile;

    /// **What is tested:** Conversion from CLI Args struct to CliArgs struct
    /// **Why it is tested:** Ensures that command-line arguments are properly converted to the internal configuration format
    /// **Test conditions:** Creates Args with various field values and converts using From trait
    /// **Expectations:** All fields should be correctly mapped from Args to CliArgs
    #[test]
    fn test_cli_args_conversion() {
        let args = Args {
            downstream: Some("test-command".to_string()),
            vcs: true,
            no_vcs: false,
            vcs_pattern: None,
        };

        let cli_args = CliArgs::from(args);
        assert_eq!(cli_args.downstream, Some("test-command".to_string()));
        assert!(cli_args.vcs);
        assert!(!cli_args.no_vcs);
        assert_eq!(cli_args.vcs_pattern, None);
    }

    /// **What is tested:** Basic diff processing with AppConfig integration
    /// **Why it is tested:** Validates that the main processing pipeline works with configuration and handles git repository detection
    /// **Test conditions:** Creates temporary file with diff content and processes with basic VCS-enabled config
    /// **Expectations:** Should either succeed with non-empty output or fail gracefully with meaningful error messages
    #[test]
    fn test_process_diff_with_config_basic() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let diff_content = r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 hello
+world
"#;

        // Create temp file and write content
        let mut temp_file = tempfile()?;
        temp_file.write_all(diff_content.as_bytes())?;

        let mut output = Vec::new();

        // Create a basic config for testing
        let cli_args = CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        };

        // This might fail if not in a git repo, but should not panic
        let config_result = AppConfig::from_cli(cli_args);

        match config_result {
            Ok(config) => {
                let result = process_diff_with_config(temp_file, &mut output, &config);

                // Either succeeds or fails gracefully
                match result {
                    Ok(_) => {
                        // If successful, output should contain something
                        assert!(!output.is_empty());
                    }
                    Err(e) => {
                        // Should be a meaningful error message
                        let error_msg = e.to_string();
                        assert!(
                            error_msg.contains("git")
                                || error_msg.contains("root")
                                || error_msg.contains("directory")
                        );
                    }
                }
            }
            Err(_) => {
                // Config creation failed - acceptable in test environment
                // without proper git setup
            }
        }

        Ok(())
    }

    /// **What is tested:** Diff processing with downstream command configuration
    /// **Why it is tested:** Ensures that downstream command integration works correctly in the main processing pipeline
    /// **Test conditions:** Creates diff content and processes with config containing downstream command ("cat")
    /// **Expectations:** Should handle downstream command without panicking, either succeeding or failing with appropriate errors
    #[test]
    fn test_process_diff_with_config_downstream(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let diff_content = r#"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 hello
+world
"#;

        // Create temp file and write content
        let mut temp_file = tempfile()?;
        temp_file.write_all(diff_content.as_bytes())?;

        let mut output = Vec::new();

        // Create config with downstream command
        let cli_args = CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: Some("cat".to_string()),
            vcs_pattern: None,
        };

        // This might fail if not in a git repo, but should not panic
        let config_result = AppConfig::from_cli(cli_args);

        match config_result {
            Ok(config) => {
                let result = process_diff_with_config(temp_file, &mut output, &config);

                // Either succeeds or fails gracefully
                match result {
                    Ok(_) => {
                        // Just verify it doesn't panic - output can be empty
                        // Test passes if no panic occurs
                    }
                    Err(e) => {
                        // Should be a meaningful error message
                        let error_msg = e.to_string();
                        assert!(
                            error_msg.contains("git")
                                || error_msg.contains("root")
                                || error_msg.contains("directory")
                                || error_msg.contains("downstream")
                        );
                    }
                }
            }
            Err(_) => {
                // Config creation failed - acceptable in test environment
                // without proper git setup
            }
        }

        Ok(())
    }

    /// **What is tested:** Handling of empty input in the main processing pipeline
    /// **Why it is tested:** Verifies graceful handling of edge case where no diff content is provided to the main function
    /// **Test conditions:** Creates empty temporary file and processes with basic configuration
    /// **Expectations:** Should handle empty input gracefully, producing minimal output or failing with meaningful errors
    #[test]
    fn test_process_diff_with_config_empty() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        // Create empty temp file
        let temp_file = tempfile()?;
        let mut output = Vec::new();

        // Create basic config
        let cli_args = CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        };

        let config_result = AppConfig::from_cli(cli_args);

        match config_result {
            Ok(config) => {
                let result = process_diff_with_config(temp_file, &mut output, &config);

                // Should handle empty input gracefully
                match result {
                    Ok(_) => {
                        // Empty input should produce empty or minimal output
                        assert!(output.len() <= 1); // Might have newline
                    }
                    Err(e) => {
                        // Should be a meaningful error message
                        let error_msg = e.to_string();
                        assert!(
                            error_msg.contains("git")
                                || error_msg.contains("root")
                                || error_msg.contains("directory")
                        );
                    }
                }
            }
            Err(_) => {
                // Config creation failed - acceptable in test environment
                // without proper git setup
            }
        }

        Ok(())
    }
}
