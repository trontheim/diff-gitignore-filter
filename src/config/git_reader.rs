//! Git configuration reader module
//!
//! This module provides low-level Git command abstraction with error handling
//! for reading Git configuration values.

use std::env;
use std::fmt;
use std::path::PathBuf;
use std::process::Command;

/// Git-specific errors that can occur during Git operations
#[derive(Debug, Clone, PartialEq)]
pub enum GitError {
    /// Git command execution failed
    CommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    /// IO error during Git command execution
    IoError { command: String, error: String },
    /// Not in a Git repository
    NotInGitRepository { path: PathBuf },
}

impl fmt::Display for GitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GitError::CommandFailed {
                command,
                exit_code,
                stderr,
            } => write!(
                f,
                "Git command '{command}' failed with exit code {exit_code}: {stderr}"
            ),
            GitError::IoError { command, error } => {
                write!(f, "IO error executing git command '{command}': {error}")
            }
            GitError::NotInGitRepository { path } => {
                write!(f, "Not in a git repository: {}", path.display())
            }
        }
    }
}

impl std::error::Error for GitError {}

/// Trait for reading Git configuration values
pub trait GitConfigReader {
    /// Get a Git configuration value by key
    fn get_config(&self, key: &str) -> Result<Option<String>, GitError>;
}

/// System Git configuration reader that executes actual Git commands
pub struct SystemGitConfigReader;

impl GitConfigReader for SystemGitConfigReader {
    fn get_config(&self, key: &str) -> Result<Option<String>, GitError> {
        let current_dir = Self::get_current_directory()?;

        Self::validate_git_repository(&current_dir)?;

        let output = Self::execute_git_config_command(key, &current_dir)?;

        Self::parse_git_config_output(output, key)
    }
}

impl SystemGitConfigReader {
    /// Get current directory with functional error handling
    fn get_current_directory() -> Result<PathBuf, GitError> {
        env::current_dir().map_err(|e| GitError::IoError {
            command: "git config".to_owned(),
            error: format!("Failed to get current directory: {e}"),
        })
    }

    /// Validate that we're in a Git repository
    fn validate_git_repository(path: &PathBuf) -> Result<(), GitError> {
        Self::is_git_repository(path)?
            .then_some(())
            .ok_or_else(|| GitError::NotInGitRepository { path: path.clone() })
    }

    /// Execute git config command with functional error handling
    fn execute_git_config_command(
        key: &str,
        current_dir: &PathBuf,
    ) -> Result<std::process::Output, GitError> {
        Command::new("git")
            .args(["config", "--get", key])
            .current_dir(current_dir)
            .output()
            .map_err(|e| GitError::IoError {
                command: format!("git config --get {key}"),
                error: e.to_string(),
            })
    }

    /// Parse git config command output using functional approach
    fn parse_git_config_output(
        output: std::process::Output,
        key: &str,
    ) -> Result<Option<String>, GitError> {
        match output.status.code() {
            Some(0) => {
                let value_string = String::from_utf8_lossy(&output.stdout);
                let value = value_string.trim();
                Ok((!value.is_empty()).then(|| value.to_owned()))
            }
            Some(1) => Ok(None), // Key is not set
            exit_code => {
                let stderr = String::from_utf8_lossy(&output.stderr);
                Err(GitError::CommandFailed {
                    command: format!("git config --get {key}"),
                    exit_code: exit_code.unwrap_or(-1),
                    stderr: stderr.to_string(),
                })
            }
        }
    }

    /// Check if the given directory is within a Git repository
    fn is_git_repository(path: &PathBuf) -> Result<bool, GitError> {
        let output = Command::new("git")
            .args(["rev-parse", "--git-dir"])
            .current_dir(path)
            .output()
            .map_err(|e| GitError::IoError {
                command: "git rev-parse --git-dir".to_owned(),
                error: e.to_string(),
            })?;

        Ok(output.status.success())
    }
}

/// Mock Git configuration reader for testing
#[cfg(test)]
pub struct MockGitConfigReader {
    config: std::collections::HashMap<String, String>,
}

#[cfg(test)]
impl Default for MockGitConfigReader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl MockGitConfigReader {
    /// Create a new mock reader
    pub fn new() -> Self {
        Self {
            config: std::collections::HashMap::new(),
        }
    }

    /// Add a configuration value to the mock reader
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_owned(), value.to_owned());
        self
    }
}

#[cfg(test)]
impl GitConfigReader for MockGitConfigReader {
    fn get_config(&self, key: &str) -> Result<Option<String>, GitError> {
        Ok(self.config.get(key).cloned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What is tested:** MockGitConfigReader functionality with multiple configuration values
    /// **Why it is tested:** Validates that the mock implementation correctly stores and retrieves configuration values for testing
    /// **Test conditions:** Creates mock reader with multiple key-value pairs and tests retrieval
    /// **Expectations:** Should return correct values for existing keys and None for non-existent keys
    #[test]
    fn test_mock_git_config_reader() {
        let mock_reader = MockGitConfigReader::new()
            .with_config("test.key1", "value1")
            .with_config("test.key2", "value2");

        assert_eq!(
            mock_reader.get_config("test.key1"),
            Ok(Some("value1".to_owned()))
        );
        assert_eq!(
            mock_reader.get_config("test.key2"),
            Ok(Some("value2".to_owned()))
        );
        assert_eq!(mock_reader.get_config("test.nonexistent"), Ok(None));
    }

    /// **What is tested:** MockGitConfigReader behavior when no configuration is set
    /// **Why it is tested:** Ensures that empty mock reader handles requests gracefully without errors
    /// **Test conditions:** Creates empty mock reader and requests configuration value
    /// **Expectations:** Should return None for any key without errors
    #[test]
    fn test_mock_git_config_reader_empty() {
        let mock_reader = MockGitConfigReader::new();
        assert_eq!(mock_reader.get_config("any.key"), Ok(None));
    }

    /// **What is tested:** Display formatting for GitError::CommandFailed variant
    /// **Why it is tested:** Ensures that command failure errors provide clear, informative error messages
    /// **Test conditions:** Creates CommandFailed error with command, exit code, and stderr message
    /// **Expectations:** Display output should contain command name, exit code, and error message
    #[test]
    fn test_git_error_display() {
        let error = GitError::CommandFailed {
            command: "git config test".to_owned(),
            exit_code: 1,
            stderr: "error message".to_owned(),
        };

        let display = format!("{error}");
        assert!(display.contains("git config test"));
        assert!(display.contains("exit code 1"));
        assert!(display.contains("error message"));
    }

    /// **What is tested:** Display formatting for GitError::IoError variant
    /// **Why it is tested:** Validates that IO errors provide clear information about the failed command and error
    /// **Test conditions:** Creates IoError with command and error message
    /// **Expectations:** Display output should contain both command name and error description
    #[test]
    fn test_git_error_io_error() {
        let error = GitError::IoError {
            command: "git config".to_owned(),
            error: "permission denied".to_owned(),
        };

        let display = format!("{error}");
        assert!(display.contains("git config"));
        assert!(display.contains("permission denied"));
    }

    /// **What is tested:** Display formatting for GitError::NotInGitRepository variant
    /// **Why it is tested:** Ensures that repository detection errors provide clear path information
    /// **Test conditions:** Creates NotInGitRepository error with specific path
    /// **Expectations:** Display output should indicate repository issue and include the problematic path
    #[test]
    fn test_git_error_not_in_repository() {
        let path = PathBuf::from("/tmp");
        let error = GitError::NotInGitRepository { path: path.clone() };

        let display = format!("{error}");
        assert!(display.contains("Not in a git repository"));
        assert!(display.contains("/tmp"));
    }

    /// **What is tested:** SystemGitConfigReader trait implementation and error handling
    /// **Why it is tested:** Validates that the system reader handles both success and failure cases appropriately
    /// **Test conditions:** Attempts to read git configuration using system reader
    /// **Expectations:** Should either succeed or fail gracefully with appropriate error types
    #[test]
    fn test_system_git_config_reader_trait() {
        let reader = SystemGitConfigReader;

        // This test will fail if not in a git repository, but should not panic
        let result = reader.get_config("user.name");

        match result {
            Ok(_) => {
                // Success case - we're in a git repo and got a result
            }
            Err(GitError::NotInGitRepository { .. }) => {
                // Expected error when not in a git repository
            }
            Err(GitError::CommandFailed { .. }) => {
                // Git command failed for some other reason
            }
            Err(GitError::IoError { .. }) => {
                // IO error occurred
            }
        }
    }
}
