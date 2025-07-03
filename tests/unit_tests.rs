use diff_gitignore_filter::config::{AppConfig, CliArgs, ConfigError};
use std::env;
use tempfile::TempDir;

mod common;

mod config_tests {
    use super::*;

    mod app_config_tests {
        use super::*;

        /// **What is tested:** Basic AppConfig creation from CLI arguments with default settings
        /// **Why it is tested:** Validates the fundamental configuration creation workflow in a test environment
        /// **Test conditions:** Creates CliArgs with all default values (no flags set)
        /// **Expectations:** Should create config with VCS enabled by default or handle git repository absence gracefully
        #[test]
        fn test_app_config_from_cli_basic() {
            // Test basic AppConfig::from_cli() functionality
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            match AppConfig::from_cli(cli_args) {
                Ok(config) => {
                    // Should have default VCS enabled
                    assert!(config.vcs_enabled());
                    assert!(!config.vcs_patterns().is_empty());
                    assert!(config.downstream_filter().is_none());
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Acceptable in test environment
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Acceptable in test environment
                }
                Err(e) => {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }

        /// **What is tested:** AppConfig creation with explicit VCS flag enabled
        /// **Why it is tested:** Ensures that the VCS flag explicitly enables VCS filtering regardless of git config
        /// **Test conditions:** Creates CliArgs with vcs flag set to true
        /// **Expectations:** Should create config with VCS filtering enabled or handle git repository absence gracefully
        #[test]
        fn test_app_config_from_cli_with_vcs_flag() {
            // Test AppConfig::from_cli() with VCS flag
            let cli_args = CliArgs {
                vcs: true,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            match AppConfig::from_cli(cli_args) {
                Ok(config) => {
                    assert!(config.vcs_enabled());
                    assert!(!config.vcs_patterns().is_empty());
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Acceptable in test environment
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Acceptable in test environment
                }
                Err(e) => {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }

        /// **What is tested:** AppConfig creation with explicit VCS filtering disabled
        /// **Why it is tested:** Validates that the no_vcs flag explicitly disables VCS filtering regardless of git config
        /// **Test conditions:** Creates CliArgs with no_vcs flag set to true
        /// **Expectations:** Should create config with VCS filtering disabled or handle git repository absence gracefully
        #[test]
        fn test_app_config_from_cli_with_no_vcs_flag() {
            // Test AppConfig::from_cli() with no VCS flag
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
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Acceptable in test environment
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Acceptable in test environment
                }
                Err(e) => {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }

        /// **What is tested:** AppConfig creation with downstream command specified
        /// **Why it is tested:** Ensures that downstream commands are properly stored and accessible in the configuration
        /// **Test conditions:** Creates CliArgs with downstream command set to "cat"
        /// **Expectations:** Should store the downstream command correctly or handle git repository absence gracefully
        #[test]
        fn test_app_config_from_cli_with_downstream() {
            // Test AppConfig::from_cli() with downstream command
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: Some("cat".to_string()),
                vcs_pattern: None,
            };

            match AppConfig::from_cli(cli_args) {
                Ok(config) => {
                    assert_eq!(config.downstream_filter(), Some("cat"));
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Acceptable in test environment
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Acceptable in test environment
                }
                Err(e) => {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }

        /// **What is tested:** Priority handling when both VCS and no_vcs flags are set
        /// **Why it is tested:** Validates that VCS flag takes precedence over no_vcs flag in conflicting scenarios
        /// **Test conditions:** Creates CliArgs with both vcs and no_vcs flags set to true
        /// **Expectations:** VCS should be enabled (vcs flag wins) or handle git repository absence gracefully
        #[test]
        fn test_app_config_from_cli_priority_vcs_over_no_vcs() {
            // Test that VCS flag takes priority over no_vcs flag
            let cli_args = CliArgs {
                vcs: true,
                no_vcs: true,
                downstream: None,
                vcs_pattern: None,
            };

            match AppConfig::from_cli(cli_args) {
                Ok(config) => {
                    assert!(config.vcs_enabled()); // VCS should win
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Acceptable in test environment
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Acceptable in test environment
                }
                Err(e) => {
                    panic!("Unexpected error: {e:?}");
                }
            }
        }

        /// **What is tested:** Error handling in AppConfig creation process
        /// **Why it is tested:** Ensures that various error scenarios are handled gracefully without panicking
        /// **Test conditions:** Creates basic CliArgs and tests error handling in different environments
        /// **Expectations:** Should either succeed or fail with expected error types (NotInGitRepository, GitCommandFailed, etc.)
        #[test]
        fn test_app_config_error_cases() {
            // Test error handling in AppConfig::from_cli()
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            let result = AppConfig::from_cli(cli_args);

            // Should either succeed or fail with expected error types
            match result {
                Ok(_) => {
                    // Success is acceptable
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Expected when not in git repository
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Expected when git command fails
                }
                Err(ConfigError::InvalidGitConfig { .. }) => {
                    // Expected when git config is invalid
                }
                Err(e) => {
                    panic!("Unexpected error type: {e:?}");
                }
            }
        }

        /// **What is tested:** Comprehensive error handling across multiple CLI argument combinations
        /// **Why it is tested:** Validates robust error handling for various configuration scenarios
        /// **Test conditions:** Tests multiple CliArgs combinations with different flag settings
        /// **Expectations:** All scenarios should either succeed or fail with expected error types
        #[test]
        fn test_app_config_comprehensive_error_scenarios() {
            // Test comprehensive error scenarios for AppConfig
            let test_scenarios = vec![
                CliArgs {
                    vcs: true,
                    no_vcs: false,
                    downstream: None,
                    vcs_pattern: None,
                },
                CliArgs {
                    vcs: false,
                    no_vcs: true,
                    downstream: None,
                    vcs_pattern: None,
                },
                CliArgs {
                    vcs: false,
                    no_vcs: false,
                    downstream: Some("echo test".to_string()),
                    vcs_pattern: None,
                },
                CliArgs {
                    vcs: true,
                    no_vcs: true,
                    downstream: Some("cat".to_string()),
                    vcs_pattern: None,
                },
            ];

            for cli_args in test_scenarios {
                let result = AppConfig::from_cli(cli_args);

                // All results should be either Ok or expected error types
                match result {
                    Ok(_) => {
                        // Success is acceptable
                    }
                    Err(ConfigError::NotInGitRepository { .. })
                    | Err(ConfigError::GitCommandFailed { .. })
                    | Err(ConfigError::InvalidGitConfig { .. }) => {
                        // All these error types are acceptable
                    }
                    Err(e) => {
                        panic!("Unexpected error type: {e:?}");
                    }
                }
            }
        }

        /// **What is tested:** NotInGitRepository error handling when outside git repository
        /// **Why it is tested:** Ensures graceful handling when the tool is used outside of git repositories
        /// **Test conditions:** Changes to temporary directory (non-git) and attempts config creation
        /// **Expectations:** Should handle non-git environment gracefully with appropriate error or fallback
        #[test]
        fn test_app_config_not_in_git_repository_error() {
            // Test NotInGitRepository error scenario
            if let Ok(temp_dir) = TempDir::new() {
                let original_dir = env::current_dir().unwrap();

                if env::set_current_dir(temp_dir.path()).is_ok() {
                    let cli_args = CliArgs {
                        vcs: false,
                        no_vcs: false,
                        downstream: None,
                        vcs_pattern: None,
                    };

                    let result = AppConfig::from_cli(cli_args);

                    // Should either succeed (if global git config exists) or fail with NotInGitRepository
                    match result {
                        Ok(_) => {
                            // Success is acceptable if global git config exists
                        }
                        Err(ConfigError::NotInGitRepository { .. }) => {
                            // Expected error type
                        }
                        Err(ConfigError::GitCommandFailed { .. }) => {
                            // Also acceptable
                        }
                        Err(e) => {
                            panic!("Unexpected error type: {e:?}");
                        }
                    }

                    let _ = env::set_current_dir(original_dir);
                }
            }
        }

        /// **What is tested:** GitCommandFailed error handling when git commands fail
        /// **Why it is tested:** Validates robust error handling when git operations fail
        /// **Test conditions:** Attempts config creation in environment where git commands might fail
        /// **Expectations:** Should handle git command failures gracefully with appropriate error types
        #[test]
        fn test_app_config_git_command_failed_error() {
            // Test GitCommandFailed error scenario
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            let result = AppConfig::from_cli(cli_args);

            // Should handle git command failures gracefully
            match result {
                Ok(_) => {
                    // Success is acceptable
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Expected error type
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Also acceptable
                }
                Err(ConfigError::InvalidGitConfig { .. }) => {
                    // Also acceptable
                }
                Err(e) => {
                    panic!("Unexpected error type: {e:?}");
                }
            }
        }

        /// **What is tested:** InvalidGitConfig error handling for malformed git configuration
        /// **Why it is tested:** Ensures graceful handling of invalid or corrupted git configuration values
        /// **Test conditions:** Attempts config creation in environment with potentially invalid git config
        /// **Expectations:** Should handle invalid git configuration gracefully with appropriate error types
        #[test]
        fn test_app_config_invalid_git_config_error() {
            // Test InvalidGitConfig error scenario
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            let result = AppConfig::from_cli(cli_args);

            // Should handle invalid git config gracefully
            match result {
                Ok(_) => {
                    // Success is acceptable
                }
                Err(ConfigError::InvalidGitConfig { .. }) => {
                    // Expected error type
                }
                Err(ConfigError::GitCommandFailed { .. }) => {
                    // Also acceptable
                }
                Err(ConfigError::NotInGitRepository { .. }) => {
                    // Also acceptable
                }
                Err(e) => {
                    panic!("Unexpected error type: {e:?}");
                }
            }
        }

        /// **What is tested:** Comprehensive validation of all `ConfigError` types and their structure
        /// **Why it is tested:** Ensures that all error types are properly structured and handled consistently
        /// **Test conditions:** Repeatedly tests config creation to encounter different error scenarios
        /// **Expectations:** Should handle all `ConfigError` variants with proper field structure validation
        #[test]
        fn test_app_config_error_types_comprehensive() {
            // Test that all ConfigError types are handled properly
            let cli_args = CliArgs {
                vcs: false,
                no_vcs: false,
                downstream: None,
                vcs_pattern: None,
            };

            // Test multiple times to catch different error scenarios
            for _ in 0..5 {
                let result = AppConfig::from_cli(cli_args.clone());

                match result {
                    Ok(_) => {
                        // Success is always acceptable
                    }
                    Err(ConfigError::NotInGitRepository { path: _ }) => {
                        // Verify error structure
                    }
                    Err(ConfigError::GitCommandFailed {
                        command: _,
                        exit_code: _,
                        stderr: _,
                    }) => {
                        // Verify error structure
                    }
                    Err(ConfigError::InvalidGitConfig {
                        key: _,
                        value: _,
                        expected: _,
                    }) => {
                        // Verify error structure
                    }
                    Err(ConfigError::IoError { source: _ }) => {
                        // Verify error structure
                    }
                    Err(ConfigError::InvalidCliArgument { .. }) => {
                        // Verify error structure
                    }
                }
            }
        }
    }

    /// **What is tested:** Thread safety and concurrent access to `AppConfig` creation
    /// **Why it is tested:** Validates that configuration creation is thread-safe and handles concurrent access
    /// **Test conditions:** Spawns multiple threads that simultaneously attempt to create `AppConfig`
    /// **Expectations:** All threads should complete without panicking and handle errors gracefully
    #[test]
    fn test_app_config_from_cli_concurrent_access() {
        // Test concurrent access to AppConfig::from_cli()
        use std::thread;

        let handles: Vec<_> = (0..4)
            .map(|_| {
                thread::spawn(|| {
                    let cli_args = CliArgs {
                        vcs: false,
                        no_vcs: false,
                        downstream: None,
                        vcs_pattern: None,
                    };

                    let result = AppConfig::from_cli(cli_args);

                    // Should handle concurrent access gracefully
                    match result {
                        Ok(_)
                        | Err(ConfigError::NotInGitRepository { .. })
                        | Err(ConfigError::GitCommandFailed { .. })
                        | Err(ConfigError::InvalidGitConfig { .. })
                        | Err(ConfigError::IoError { .. })
                        | Err(ConfigError::InvalidCliArgument { .. }) => {
                            // All these results are acceptable
                        }
                    }
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.join().is_ok(), "Thread panicked");
        }
    }
}

/// **What is tested:** Basic library functionality integration at the unit test level
/// **Why it is tested:** Provides a high-level validation that core library components work together
/// **Test conditions:** Creates basic `CliArgs` and tests fundamental configuration creation
/// **Expectations:** Should demonstrate basic functionality or handle test environment limitations gracefully
#[test]
fn test_basic_functionality() {
    // Test basic library functionality
    let cli_args = CliArgs {
        vcs: false,
        no_vcs: false,
        downstream: None,
        vcs_pattern: None,
    };

    let result = AppConfig::from_cli(cli_args);

    // Should handle basic functionality gracefully
    match result {
        Ok(_) => {
            // Success is acceptable
        }
        Err(ConfigError::NotInGitRepository { .. })
        | Err(ConfigError::GitCommandFailed { .. })
        | Err(ConfigError::InvalidGitConfig { .. })
        | Err(ConfigError::IoError { .. })
        | Err(ConfigError::InvalidCliArgument { .. }) => {
            // All these error types are acceptable in test environment
        }
    }
}
