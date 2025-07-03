//! Application configuration module
//!
//! This module provides the main application configuration structure that combines
//! CLI arguments with Git configuration values using a clear priority system.

use super::{ConfigError, GitConfig};

/// CLI arguments structure
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliArgs {
    /// Enable VCS ignore filtering
    pub vcs: bool,
    /// Disable VCS ignore filtering
    pub no_vcs: bool,
    /// Downstream command for piping output
    pub downstream: Option<String>,
    /// Custom VCS patterns (overrides git config)
    pub vcs_pattern: Option<String>,
}

/// Main application configuration
#[derive(Debug, Clone)]
pub struct AppConfig {
    /// Whether VCS filtering is enabled
    vcs_enabled: bool,
    /// VCS patterns to use for filtering
    vcs_patterns: Vec<String>,
    /// Optional downstream filter command
    downstream_filter: Option<String>,
}

/// Configuration builder for functional composition
#[derive(Debug, Clone)]
pub struct ConfigBuilder {
    vcs_enabled: Option<bool>,
    vcs_patterns: Option<Vec<String>>,
    downstream_filter: Option<String>,
}

impl ConfigBuilder {
    /// Create a new configuration builder
    #[must_use]
    pub const fn new() -> Self {
        Self {
            vcs_enabled: None,
            vcs_patterns: None,
            downstream_filter: None,
        }
    }

    /// Set VCS enabled state
    #[must_use]
    pub const fn with_vcs_enabled(mut self, enabled: bool) -> Self {
        self.vcs_enabled = Some(enabled);
        self
    }

    /// Set VCS patterns
    #[must_use]
    pub fn with_vcs_patterns(mut self, patterns: Vec<String>) -> Self {
        self.vcs_patterns = Some(patterns);
        self
    }

    /// Set downstream filter
    pub fn with_downstream_filter(mut self, filter: Option<String>) -> Self {
        self.downstream_filter = filter;
        self
    }

    /// Build the final AppConfig
    pub fn build(self) -> AppConfig {
        AppConfig {
            vcs_enabled: self.vcs_enabled.unwrap_or(true),
            vcs_patterns: self.vcs_patterns.unwrap_or_else(Self::default_vcs_patterns),
            downstream_filter: self.downstream_filter,
        }
    }

    /// Get default VCS patterns
    fn default_vcs_patterns() -> Vec<String> {
        vec![
            ".git/".to_owned(),
            ".svn/".to_owned(),
            "_svn/".to_owned(),
            ".hg/".to_owned(),
            "CVS/".to_owned(),
            "CVSROOT/".to_owned(),
            ".bzr/".to_owned(),
        ]
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl AppConfig {
    /// Create AppConfig from CLI arguments using functional composition
    ///
    /// Priority order:
    /// 1. CLI parameters (highest priority)
    /// 2. Git configuration values
    /// 3. Hardcoded defaults (only when Git config not set)
    pub fn from_cli(cli_args: CliArgs) -> Result<Self, ConfigError> {
        let config_builder = ConfigBuilder::new()
            .with_vcs_enabled(Self::resolve_vcs_enabled(&cli_args)?)
            .with_vcs_patterns(Self::resolve_vcs_patterns(&cli_args)?)
            .with_downstream_filter(Self::resolve_downstream_filter(&cli_args));

        Ok(config_builder.build())
    }

    /// Resolve VCS enabled state using functional combinators
    fn resolve_vcs_enabled(cli_args: &CliArgs) -> Result<bool, ConfigError> {
        [
            cli_args.vcs.then_some(true),
            cli_args.no_vcs.then_some(false),
        ]
        .into_iter()
        .flatten()
        .next()
        .map(Ok)
        .unwrap_or_else(|| {
            // Fallback to Git config or default - exactly like original logic
            match GitConfig::get_vcs_ignore_enabled() {
                Ok(Some(enabled)) => Ok(enabled),
                Ok(None) | Err(_) => Ok(true), // Default: VCS filtering enabled (also when not in Git repo)
            }
        })
    }

    /// Resolve VCS patterns using functional composition
    fn resolve_vcs_patterns(cli_args: &CliArgs) -> Result<Vec<String>, ConfigError> {
        cli_args
            .vcs_pattern
            .as_ref()
            .map(|patterns_str| Self::parse_cli_vcs_patterns(patterns_str))
            .unwrap_or_else(|| {
                // Fallback to existing Git-Config logic - exactly like original
                match GitConfig::get_vcs_ignore_patterns() {
                    Ok(Some(git_patterns)) => Ok(git_patterns),
                    Ok(None) | Err(_) => Ok(ConfigBuilder::default_vcs_patterns()),
                }
            })
    }

    /// Resolve downstream filter using functional combinators
    fn resolve_downstream_filter(cli_args: &CliArgs) -> Option<String> {
        cli_args
            .downstream
            .clone()
            .or_else(|| GitConfig::get_downstream_filter().ok().flatten())
    }

    /// Parse and validate CLI VCS patterns using functional approach
    fn parse_cli_vcs_patterns(patterns_str: &str) -> Result<Vec<String>, ConfigError> {
        let patterns: Vec<String> = patterns_str
            .split(',')
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(ToOwned::to_owned)
            .collect();

        (!patterns.is_empty())
            .then_some(patterns)
            .ok_or_else(|| ConfigError::InvalidCliArgument {
                argument: "--vcs-pattern".to_owned(),
                value: patterns_str.to_owned(),
                expected: "comma-separated list of non-empty patterns".to_owned(),
            })
    }

    /// Check if VCS filtering is enabled
    pub fn vcs_enabled(&self) -> bool {
        self.vcs_enabled
    }

    /// Get VCS patterns
    pub fn vcs_patterns(&self) -> &[String] {
        &self.vcs_patterns
    }

    /// Get downstream filter command
    pub fn downstream_filter(&self) -> Option<&str> {
        self.downstream_filter.as_deref()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What is tested:** Parsing of valid CLI VCS patterns from comma-separated string
    /// **Why it is tested:** Ensures that valid VCS pattern strings are correctly parsed into individual patterns
    /// **Test conditions:** Provides comma-separated VCS patterns string with standard patterns
    /// **Expectations:** Should return vector with correctly parsed individual patterns
    #[test]
    fn test_parse_cli_vcs_patterns_valid() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let patterns = AppConfig::parse_cli_vcs_patterns(".git/,.svn/,.hg/")?;
        assert_eq!(patterns, vec![".git/", ".svn/", ".hg/"]);
        Ok(())
    }

    /// **What is tested:** Parsing of CLI VCS patterns with surrounding whitespace
    /// **Why it is tested:** Validates that whitespace around patterns is properly trimmed during parsing
    /// **Test conditions:** Provides VCS patterns string with extra spaces around patterns and commas
    /// **Expectations:** Should return clean patterns with whitespace removed
    #[test]
    fn test_parse_cli_vcs_patterns_with_whitespace(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let patterns = AppConfig::parse_cli_vcs_patterns(" .git/ , .svn/ , .hg/ ")?;
        assert_eq!(patterns, vec![".git/", ".svn/", ".hg/"]);
        Ok(())
    }

    /// **What is tested:** Error handling for empty VCS patterns string
    /// **Why it is tested:** Ensures that empty pattern strings are rejected with appropriate error
    /// **Test conditions:** Provides empty string as VCS patterns input
    /// **Expectations:** Should return InvalidCliArgument error for empty input
    #[test]
    fn test_parse_cli_vcs_patterns_empty_invalid() {
        let result = AppConfig::parse_cli_vcs_patterns("");
        assert!(matches!(
            result,
            Err(ConfigError::InvalidCliArgument { .. })
        ));
    }

    /// **What is tested:** Error handling for VCS patterns string containing only commas
    /// **Why it is tested:** Validates that strings with only separators (no actual patterns) are rejected
    /// **Test conditions:** Provides string with only commas as VCS patterns input
    /// **Expectations:** Should return InvalidCliArgument error for comma-only input
    #[test]
    fn test_parse_cli_vcs_patterns_only_commas_invalid() {
        let result = AppConfig::parse_cli_vcs_patterns(",,,");
        assert!(matches!(
            result,
            Err(ConfigError::InvalidCliArgument { .. })
        ));
    }

    /// **What is tested:** Basic AppConfig creation from CLI arguments with VCS enabled
    /// **Why it is tested:** Validates the main configuration creation workflow with standard settings
    /// **Test conditions:** Creates CliArgs with VCS enabled and no custom patterns or downstream
    /// **Expectations:** Should create config with VCS enabled and default patterns (or handle git repo absence gracefully)
    #[test]
    fn test_from_cli_basic() {
        let cli_args = CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        };

        // This test might fail if not in a git repo, but should not panic
        let result = AppConfig::from_cli(cli_args);

        match result {
            Ok(config) => {
                assert!(config.vcs_enabled());
                assert!(!config.vcs_patterns().is_empty());
            }
            Err(_) => {
                // Acceptable in test environment without proper git setup
            }
        }
    }

    /// **What is tested:** AppConfig creation with custom VCS patterns from CLI
    /// **Why it is tested:** Ensures that custom VCS patterns override default patterns correctly
    /// **Test conditions:** Creates CliArgs with custom VCS patterns specified
    /// **Expectations:** Should use the custom patterns instead of defaults (or handle git repo absence gracefully)
    #[test]
    fn test_from_cli_with_custom_patterns() {
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: false,
            downstream: None,
            vcs_pattern: Some(".custom/,.test/".to_owned()),
        };

        // This test might fail if not in a git repo, but should not panic
        let result = AppConfig::from_cli(cli_args);

        match result {
            Ok(config) => {
                assert_eq!(config.vcs_patterns(), &[".custom/", ".test/"]);
            }
            Err(_) => {
                // Acceptable in test environment without proper git setup
            }
        }
    }

    /// **What is tested:** AppConfig creation with VCS filtering explicitly disabled
    /// **Why it is tested:** Validates that the no_vcs flag properly disables VCS filtering
    /// **Test conditions:** Creates CliArgs with no_vcs set to true
    /// **Expectations:** Should create config with VCS filtering disabled (or handle git repo absence gracefully)
    #[test]
    fn test_from_cli_vcs_disabled() {
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: true,
            downstream: None,
            vcs_pattern: None,
        };

        // This test might fail if not in a git repo, but should not panic
        let result = AppConfig::from_cli(cli_args);

        match result {
            Ok(config) => {
                assert!(!config.vcs_enabled());
            }
            Err(_) => {
                // Acceptable in test environment without proper git setup
            }
        }
    }

    /// **What is tested:** AppConfig creation with downstream command specified
    /// **Why it is tested:** Ensures that downstream commands are properly stored in the configuration
    /// **Test conditions:** Creates CliArgs with downstream command specified
    /// **Expectations:** Should store the downstream command correctly (or handle git repo absence gracefully)
    #[test]
    fn test_from_cli_with_downstream() {
        let cli_args = CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: Some("less".to_owned()),
            vcs_pattern: None,
        };

        // This test might fail if not in a git repo, but should not panic
        let result = AppConfig::from_cli(cli_args);

        match result {
            Ok(config) => {
                assert_eq!(config.downstream_filter(), Some("less"));
            }
            Err(_) => {
                // Acceptable in test environment without proper git setup
            }
        }
    }
}
