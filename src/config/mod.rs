//! Configuration module for diff-gitignore-filter
//!
//! This module provides a unified configuration system that combines CLI arguments
//! with Git configuration values using strict error handling and clear priority logic.
//!
//! # Architecture
//!
//! The configuration system is built with a layered architecture:
//!
//! - [`git_reader`] - Low-level Git command abstraction with error handling
//! - [`git_config`] - Git-specific configuration operations with validation
//! - [`app_config`] - High-level application configuration with CLI integration
//!
//! # Error Handling
//!
//! The configuration system uses strict error handling:
//!
//! - Git command failures result in [`ConfigError`], not fallback to defaults
//! - Invalid Git configuration values result in [`ConfigError`], not fallback to defaults
//! - Only when Git configuration is not set (not on errors) are default values used
//!
//! # Priority Logic
//!
//! Configuration values are resolved with the following priority:
//!
//! 1. CLI parameters (highest priority)
//! 2. Git configuration values
//! 3. Hardcoded defaults (only when Git config not set)
//!
//! # Usage
//!
//! The main entry point is [`AppConfig::from_cli()`] which creates a fully
//! configured application instance:
//!
//! ```rust
//! use diff_gitignore_filter::config::{AppConfig, CliArgs, ConfigError};
//!
//! let cli_args = CliArgs {
//!     vcs: false,
//!     no_vcs: false,
//!     downstream: None,
//!     vcs_pattern: None,
//! };
//!
//! match AppConfig::from_cli(cli_args) {
//!     Ok(config) => {
//!         println!("VCS enabled: {}", config.vcs_enabled());
//!         println!("Patterns: {:?}", config.vcs_patterns());
//!         if let Some(filter) = config.downstream_filter() {
//!             println!("Downstream filter: {}", filter);
//!         }
//!     }
//!     Err(ConfigError::GitCommandFailed { command, exit_code, stderr }) => {
//!         // Handle git command failure
//!     }
//!     Err(ConfigError::InvalidGitConfig { key, value, expected }) => {
//!         // Handle invalid git config
//!     }
//!     Err(ConfigError::NotInGitRepository { path }) => {
//!         // Handle not in git repository
//!     }
//!     Err(ConfigError::IoError { source }) => {
//!         // Handle IO error
//!     }
//!     Err(ConfigError::InvalidCliArgument { argument, value, expected }) => {
//!         // Handle invalid CLI argument
//!     }
//! }
//! ```
//!
//! # Testing
//!
//! The configuration system provides mock implementations for testing:
//!
//! - `MockGitConfigReader` for testing Git configuration scenarios
//! - All modules include comprehensive unit tests with error scenarios
//!
//! # Error Types
//!
//! All configuration errors are represented by [`ConfigError`] which provides
//! detailed, actionable error information for debugging and user feedback.

// Public modules
pub mod app_config;
pub mod git_config;
pub mod git_reader;

// Re-export public types for convenient access
pub use app_config::{AppConfig, CliArgs};
pub use git_config::{ConfigError, GitConfig};
pub use git_reader::{GitConfigReader, GitError, SystemGitConfigReader};

// Re-export mock types for testing
#[cfg(test)]
pub use git_reader::MockGitConfigReader;

#[cfg(test)]
mod integration_tests {
    use super::*;

    /// **What is tested:** Availability and accessibility of all public API types through the module
    /// **Why it is tested:** Ensures that the module correctly re-exports all necessary types for external use
    /// **Test conditions:** Creates instances of all public types (CliArgs, ConfigError, SystemGitConfigReader, etc.)
    /// **Expectations:** All public types should be accessible and instantiable through the module interface
    #[test]
    fn test_public_api_availability() {
        // Test that all public types are accessible through the module

        // AppConfig and CliArgs should be available
        let _cli_args = CliArgs {
            vcs: false,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        };

        // ConfigError should be available for error handling
        let _error = ConfigError::IoError {
            source: "test".to_owned(),
        };

        // GitConfigReader trait should be available
        let _reader = SystemGitConfigReader;

        // GitConfig should be available for direct access if needed
        // (though AppConfig::from_cli is the preferred interface)
        let _result = GitConfig::get_vcs_ignore_enabled();
    }

    /// **What is tested:** Required trait implementations for ConfigError type
    /// **Why it is tested:** Validates that ConfigError implements all necessary traits for proper error handling and usage
    /// **Test conditions:** Creates ConfigError instances and tests Debug, Display, Error, Clone, and PartialEq traits
    /// **Expectations:** ConfigError should implement all required traits without compilation errors
    #[test]
    fn test_error_types_implement_required_traits() {
        // ConfigError should implement required traits
        let error = ConfigError::IoError {
            source: "test".to_owned(),
        };

        // Should implement Debug
        let _debug = format!("{error:?}");

        // Should implement Display
        let _display = format!("{error}");

        // Should implement Error trait
        let _error_trait: &dyn std::error::Error = &error;

        // Should implement Clone
        let _cloned = error.clone();

        // Should implement PartialEq
        let error2 = ConfigError::IoError {
            source: "test".to_owned(),
        };
        assert_eq!(error, error2);
    }

    /// **What is tested:** Required trait implementations for GitError type
    /// **Why it is tested:** Ensures that GitError implements all necessary traits for proper error handling in Git operations
    /// **Test conditions:** Creates GitError instances and tests Debug, Display, Error, Clone, and PartialEq traits
    /// **Expectations:** GitError should implement all required traits and support proper error chaining
    #[test]
    fn test_git_error_types_implement_required_traits() {
        // GitError should implement required traits
        let error = GitError::IoError {
            command: "git config".to_owned(),
            error: "test".to_owned(),
        };

        // Should implement Debug
        let _debug = format!("{error:?}");

        // Should implement Display
        let _display = format!("{error}");

        // Should implement Error trait
        let _error_trait: &dyn std::error::Error = &error;

        // Should implement Clone
        let _cloned = error.clone();

        // Should implement PartialEq
        let error2 = GitError::IoError {
            command: "git config".to_owned(),
            error: "test".to_owned(),
        };
        assert_eq!(error, error2);
    }

    /// **What is tested:** Required trait implementations for AppConfig and CliArgs types
    /// **Why it is tested:** Validates that configuration types implement necessary traits for proper usage and testing
    /// **Test conditions:** Creates AppConfig and CliArgs instances and tests Debug, Clone, and PartialEq traits
    /// **Expectations:** Both types should implement required traits and handle Result types properly
    #[test]
    fn test_app_config_and_cli_args_traits() {
        // AppConfig should implement required traits
        let config = AppConfig::from_cli(CliArgs {
            vcs: true,
            no_vcs: false,
            downstream: None,
            vcs_pattern: None,
        });

        // Should handle Result properly
        match config {
            Ok(config) => {
                // Should implement Debug
                let _debug = format!("{config:?}");

                // Should implement Clone
                let _cloned = config.clone();
            }
            Err(_) => {
                // Error case is acceptable in test environment
            }
        }

        // CliArgs should implement required traits
        let cli_args = CliArgs {
            vcs: false,
            no_vcs: true,
            downstream: Some("filter".to_owned()),
            vcs_pattern: None,
        };

        // Should implement Debug
        let _debug = format!("{cli_args:?}");

        // Should implement Clone
        let _cloned = cli_args.clone();

        // Should implement PartialEq
        let cli_args2 = CliArgs {
            vcs: false,
            no_vcs: true,
            downstream: Some("filter".to_owned()),
            vcs_pattern: None,
        };
        assert_eq!(cli_args, cli_args2);
    }

    /// **What is tested:** Availability and functionality of MockGitConfigReader in test builds
    /// **Why it is tested:** Ensures that the mock implementation is properly available for testing scenarios
    /// **Test conditions:** Creates MockGitConfigReader with test configuration and verifies functionality
    /// **Expectations:** MockGitConfigReader should be available in test builds and return configured values
    #[test]
    fn test_mock_reader_availability_in_tests() {
        // MockGitConfigReader should be available in test builds
        let mock_reader = MockGitConfigReader::new().with_config("test.key", "test-value");

        let result = mock_reader.get_config("test.key");
        assert_eq!(result, Ok(Some("test-value".to_owned())));
    }

    /// **What is tested:** Integration between all configuration components (GitConfig, MockGitConfigReader, etc.)
    /// **Why it is tested:** Validates that all configuration components work together correctly in a complete workflow
    /// **Test conditions:** Creates mock reader with full configuration and tests GitConfig operations
    /// **Expectations:** All components should integrate seamlessly and return expected configuration values
    #[test]
    fn test_integration_with_all_components() {
        // Test that all components work together
        use crate::config::git_reader::MockGitConfigReader;

        // Create a mock reader with test configuration
        let mock_reader = MockGitConfigReader::new()
            .with_config("diff-gitignore-filter.vcs-ignore.enabled", "true")
            .with_config("diff-gitignore-filter.vcs-ignore.patterns", ".git/,.svn/")
            .with_config("gitignore-diff.downstream-filter", "test-filter");

        // Test GitConfig with mock reader
        let vcs_enabled = GitConfig::get_vcs_ignore_enabled_with_reader(&mock_reader);
        assert_eq!(vcs_enabled, Ok(Some(true)));

        let patterns = GitConfig::get_vcs_ignore_patterns_with_reader(&mock_reader);
        assert_eq!(
            patterns,
            Ok(Some(vec![".git/".to_owned(), ".svn/".to_owned()]))
        );

        let downstream = GitConfig::get_downstream_filter_with_reader(&mock_reader);
        assert_eq!(downstream, Ok(Some("test-filter".to_owned())));
    }
}
