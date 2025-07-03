//! Git configuration module
//!
//! This module provides Git-specific configuration operations with validation
//! and error handling for diff-gitignore-filter settings.

use super::git_reader::{GitConfigReader, GitError, SystemGitConfigReader};
use std::fmt;
use std::path::PathBuf;

/// Configuration errors that can occur during Git config operations
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigError {
    /// Git command execution failed
    GitCommandFailed {
        command: String,
        exit_code: i32,
        stderr: String,
    },
    /// Invalid Git configuration value
    InvalidGitConfig {
        key: String,
        value: String,
        expected: String,
    },
    /// Not in a Git repository
    NotInGitRepository { path: PathBuf },
    /// IO error during configuration
    IoError { source: String },
    /// Invalid CLI argument value
    InvalidCliArgument {
        argument: String,
        value: String,
        expected: String,
    },
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::GitCommandFailed {
                command,
                exit_code,
                stderr,
            } => write!(
                f,
                "Git command '{command}' failed with exit code {exit_code}: {stderr}"
            ),
            ConfigError::InvalidGitConfig {
                key,
                value,
                expected,
            } => write!(
                f,
                "Invalid git config value: {key}='{value}' (expected: {expected})"
            ),
            ConfigError::NotInGitRepository { path } => {
                write!(f, "Not in a git repository: {}", path.display())
            }
            ConfigError::IoError { source } => {
                write!(f, "IO error during configuration: {source}")
            }
            ConfigError::InvalidCliArgument {
                argument,
                value,
                expected,
            } => write!(
                f,
                "Invalid CLI argument: {argument}='{value}' (expected: {expected})"
            ),
        }
    }
}

impl std::error::Error for ConfigError {}

impl From<GitError> for ConfigError {
    fn from(error: GitError) -> Self {
        match error {
            GitError::CommandFailed {
                command,
                exit_code,
                stderr,
            } => ConfigError::GitCommandFailed {
                command,
                exit_code,
                stderr,
            },
            GitError::IoError { command, error } => ConfigError::IoError {
                source: format!("Git command '{command}' failed: {error}"),
            },
            GitError::NotInGitRepository { path } => ConfigError::NotInGitRepository { path },
        }
    }
}

/// Git configuration operations
pub struct GitConfig;

impl GitConfig {
    /// Get VCS ignore enabled setting from Git config
    pub fn get_vcs_ignore_enabled() -> Result<Option<bool>, ConfigError> {
        let reader = SystemGitConfigReader;
        Self::get_vcs_ignore_enabled_with_reader(&reader)
    }

    /// Get VCS ignore enabled setting with custom reader (for testing)
    pub fn get_vcs_ignore_enabled_with_reader<R: GitConfigReader>(
        reader: &R,
    ) -> Result<Option<bool>, ConfigError> {
        let key = "diff-gitignore-filter.vcs-ignore.enabled";

        reader
            .get_config(key)?
            .map(|value| Self::parse_boolean_value(&value, key))
            .transpose()
    }

    /// Parse boolean value from Git configuration using functional approach
    fn parse_boolean_value(value: &str, key: &str) -> Result<bool, ConfigError> {
        let normalized = value.to_lowercase();

        ["true", "1", "yes", "on"]
            .iter()
            .any(|&v| v == normalized)
            .then_some(true)
            .or_else(|| {
                ["false", "0", "no", "off"]
                    .iter()
                    .any(|&v| v == normalized)
                    .then_some(false)
            })
            .ok_or_else(|| ConfigError::InvalidGitConfig {
                key: key.to_owned(),
                value: value.to_owned(),
                expected: "true, false, 1, 0, yes, no, on, or off".to_owned(),
            })
    }

    /// Get VCS ignore patterns from Git config
    pub fn get_vcs_ignore_patterns() -> Result<Option<Vec<String>>, ConfigError> {
        let reader = SystemGitConfigReader;
        Self::get_vcs_ignore_patterns_with_reader(&reader)
    }

    /// Get VCS ignore patterns with custom reader (for testing)
    pub fn get_vcs_ignore_patterns_with_reader<R: GitConfigReader>(
        reader: &R,
    ) -> Result<Option<Vec<String>>, ConfigError> {
        let key = "diff-gitignore-filter.vcs-ignore.patterns";

        reader
            .get_config(key)?
            .map(|value| Self::parse_comma_separated_patterns(&value, key))
            .transpose()
    }

    /// Parse comma-separated patterns using functional approach
    fn parse_comma_separated_patterns(value: &str, key: &str) -> Result<Vec<String>, ConfigError> {
        let patterns: Vec<String> = value
            .split(',')
            .map(str::trim)
            .filter(|pattern| !pattern.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        (!patterns.is_empty())
            .then_some(patterns)
            .ok_or_else(|| ConfigError::InvalidGitConfig {
                key: key.to_owned(),
                value: value.to_owned(),
                expected: "comma-separated list of non-empty patterns".to_owned(),
            })
    }

    /// Get downstream filter command from Git config
    pub fn get_downstream_filter() -> Result<Option<String>, ConfigError> {
        let reader = SystemGitConfigReader;
        Self::get_downstream_filter_with_reader(&reader)
    }

    /// Get downstream filter command with custom reader (for testing)
    pub fn get_downstream_filter_with_reader<R: GitConfigReader>(
        reader: &R,
    ) -> Result<Option<String>, ConfigError> {
        let key = "gitignore-diff.downstream-filter";

        reader
            .get_config(key)
            .map(|opt| opt.and_then(Self::parse_downstream_filter))
            .map_err(ConfigError::from)
    }

    /// Parse downstream filter value using functional approach
    fn parse_downstream_filter(value: String) -> Option<String> {
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::git_reader::MockGitConfigReader;

    /// **What is tested:** VCS ignore enabled configuration parsing for true values
    /// **Why it is tested:** Ensures that "true" values in git config are correctly parsed as boolean true
    /// **Test conditions:** Mock git config with "diff-gitignore-filter.vcs-ignore.enabled" set to "true"
    /// **Expectations:** Should return Ok(Some(true)) for true configuration value
    #[test]
    fn test_get_vcs_ignore_enabled_true() {
        let mock_reader = MockGitConfigReader::new()
            .with_config("diff-gitignore-filter.vcs-ignore.enabled", "true");

        let result = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
        assert_eq!(result, Ok(Some(true)));
    }

    /// **What is tested:** VCS ignore enabled configuration parsing for false values
    /// **Why it is tested:** Validates that "false" values in git config are correctly parsed as boolean false
    /// **Test conditions:** Mock git config with "diff-gitignore-filter.vcs-ignore.enabled" set to "false"
    /// **Expectations:** Should return Ok(Some(false)) for false configuration value
    #[test]
    fn test_get_vcs_ignore_enabled_false() {
        let mock_reader = MockGitConfigReader::new()
            .with_config("diff-gitignore-filter.vcs-ignore.enabled", "false");

        let result = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
        assert_eq!(result, Ok(Some(false)));
    }

    /// **What is tested:** VCS ignore enabled configuration parsing for various boolean representations
    /// **Why it is tested:** Ensures that all standard git boolean values (true/false, 1/0, yes/no, on/off) are correctly parsed
    /// **Test conditions:** Tests multiple boolean representations in both upper and lower case
    /// **Expectations:** Should correctly parse all standard git boolean formats to appropriate boolean values
    #[test]
    fn test_get_vcs_ignore_enabled_various_values() {
        let test_cases = vec![
            ("true", Some(true)),
            ("TRUE", Some(true)),
            ("1", Some(true)),
            ("yes", Some(true)),
            ("on", Some(true)),
            ("false", Some(false)),
            ("FALSE", Some(false)),
            ("0", Some(false)),
            ("no", Some(false)),
            ("off", Some(false)),
        ];

        for (value, expected) in test_cases {
            let mock_reader = MockGitConfigReader::new()
                .with_config("diff-gitignore-filter.vcs-ignore.enabled", value);

            let result = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
            assert_eq!(result, Ok(expected), "Failed for value: {value}");
        }
    }

    /// **What is tested:** Error handling for invalid VCS ignore enabled configuration values
    /// **Why it is tested:** Validates that invalid boolean values result in appropriate configuration errors
    /// **Test conditions:** Mock git config with invalid boolean value for VCS ignore enabled setting
    /// **Expectations:** Should return InvalidGitConfig error for invalid boolean values
    #[test]
    fn test_get_vcs_ignore_enabled_invalid() {
        let mock_reader = MockGitConfigReader::new()
            .with_config("diff-gitignore-filter.vcs-ignore.enabled", "invalid");

        let result = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
        assert!(matches!(result, Err(ConfigError::InvalidGitConfig { .. })));
    }

    /// **What is tested:** Handling of unset VCS ignore enabled configuration
    /// **Why it is tested:** Ensures that missing configuration values are handled gracefully
    /// **Test conditions:** Mock git config without VCS ignore enabled setting
    /// **Expectations:** Should return Ok(None) when configuration is not set
    #[test]
    fn test_get_vcs_ignore_enabled_not_set() {
        let mock_reader = MockGitConfigReader::new();

        let result = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
        assert_eq!(result, Ok(None));
    }

    /// **What is tested:** VCS ignore patterns parsing for valid comma-separated values
    /// **Why it is tested:** Validates that comma-separated VCS patterns are correctly parsed into individual patterns
    /// **Test conditions:** Mock git config with valid comma-separated VCS patterns
    /// **Expectations:** Should return vector of individual patterns parsed from comma-separated string
    #[test]
    fn test_get_vcs_ignore_patterns_valid() {
        let mock_reader = MockGitConfigReader::new().with_config(
            "diff-gitignore-filter.vcs-ignore.patterns",
            ".git/,.svn/,.hg/",
        );

        let result = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert_eq!(
            result,
            Ok(Some(vec![
                ".git/".to_owned(),
                ".svn/".to_owned(),
                ".hg/".to_owned()
            ]))
        );
    }

    /// **What is tested:** VCS ignore patterns parsing with surrounding whitespace
    /// **Why it is tested:** Ensures that whitespace around patterns is properly trimmed during parsing
    /// **Test conditions:** Mock git config with VCS patterns containing extra whitespace
    /// **Expectations:** Should return clean patterns with whitespace removed
    #[test]
    fn test_get_vcs_ignore_patterns_with_whitespace() {
        let mock_reader = MockGitConfigReader::new().with_config(
            "diff-gitignore-filter.vcs-ignore.patterns",
            " .git/ , .svn/ , .hg/ ",
        );

        let result = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert_eq!(
            result,
            Ok(Some(vec![
                ".git/".to_owned(),
                ".svn/".to_owned(),
                ".hg/".to_owned()
            ]))
        );
    }

    /// **What is tested:** Error handling for empty VCS ignore patterns configuration
    /// **Why it is tested:** Validates that empty pattern strings result in appropriate configuration errors
    /// **Test conditions:** Mock git config with empty string for VCS patterns
    /// **Expectations:** Should return InvalidGitConfig error for empty pattern configuration
    #[test]
    fn test_get_vcs_ignore_patterns_empty_invalid() {
        let mock_reader =
            MockGitConfigReader::new().with_config("diff-gitignore-filter.vcs-ignore.patterns", "");

        let result = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert!(matches!(result, Err(ConfigError::InvalidGitConfig { .. })));
    }

    /// **What is tested:** Error handling for VCS patterns containing only commas
    /// **Why it is tested:** Ensures that patterns with only separators (no actual patterns) are rejected
    /// **Test conditions:** Mock git config with comma-only string for VCS patterns
    /// **Expectations:** Should return InvalidGitConfig error for comma-only pattern configuration
    #[test]
    fn test_get_vcs_ignore_patterns_only_commas_invalid() {
        let mock_reader = MockGitConfigReader::new()
            .with_config("diff-gitignore-filter.vcs-ignore.patterns", ",,,");

        let result = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert!(matches!(result, Err(ConfigError::InvalidGitConfig { .. })));
    }

    /// **What is tested:** Handling of unset VCS ignore patterns configuration
    /// **Why it is tested:** Validates that missing VCS pattern configuration is handled gracefully
    /// **Test conditions:** Mock git config without VCS patterns setting
    /// **Expectations:** Should return Ok(None) when VCS patterns configuration is not set
    #[test]
    fn test_get_vcs_ignore_patterns_not_set() {
        let mock_reader = MockGitConfigReader::new();

        let result = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert_eq!(result, Ok(None));
    }

    /// **What is tested:** Downstream filter configuration parsing for valid values
    /// **Why it is tested:** Ensures that downstream filter commands are correctly retrieved from git configuration
    /// **Test conditions:** Mock git config with valid downstream filter command
    /// **Expectations:** Should return the configured downstream filter command
    #[test]
    fn test_get_downstream_filter_valid() {
        let mock_reader =
            MockGitConfigReader::new().with_config("gitignore-diff.downstream-filter", "less");

        let result = GitConfig::get_downstream_filter_with_reader(&mock_reader);
        assert_eq!(result, Ok(Some("less".to_owned())));
    }

    /// **What is tested:** Downstream filter configuration handling for empty values
    /// **Why it is tested:** Validates that empty downstream filter values are treated as unset
    /// **Test conditions:** Mock git config with empty string for downstream filter
    /// **Expectations:** Should return None for empty downstream filter configuration
    #[test]
    fn test_get_downstream_filter_empty() {
        let mock_reader =
            MockGitConfigReader::new().with_config("gitignore-diff.downstream-filter", "");

        let result = GitConfig::get_downstream_filter_with_reader(&mock_reader);
        assert_eq!(result, Ok(None));
    }

    /// **What is tested:** Downstream filter configuration handling for whitespace-only values
    /// **Why it is tested:** Ensures that whitespace-only values are treated as unset rather than valid commands
    /// **Test conditions:** Mock git config with whitespace-only string for downstream filter
    /// **Expectations:** Should return None for whitespace-only downstream filter configuration
    #[test]
    fn test_get_downstream_filter_whitespace_only() {
        let mock_reader =
            MockGitConfigReader::new().with_config("gitignore-diff.downstream-filter", "   ");

        let result = GitConfig::get_downstream_filter_with_reader(&mock_reader);
        assert_eq!(result, Ok(None));
    }

    /// **What is tested:** Handling of unset downstream filter configuration
    /// **Why it is tested:** Validates that missing downstream filter configuration is handled gracefully
    /// **Test conditions:** Mock git config without downstream filter setting
    /// **Expectations:** Should return Ok(None) when downstream filter configuration is not set
    #[test]
    fn test_get_downstream_filter_not_set() {
        let mock_reader = MockGitConfigReader::new();

        let result = GitConfig::get_downstream_filter_with_reader(&mock_reader);
        assert_eq!(result, Ok(None));
    }
}
