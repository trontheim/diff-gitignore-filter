//! Dedicated Framework Tests
//!
//! This file contains all tests that validate the test framework itself.
//! These tests are consolidated here to avoid duplication across multiple test files.

mod common;

use common::framework::{CommandOutput, Expectation, TestCase, TestCommand, TestData, TestRepo};

/// **What is tested:** Framework API accessibility and basic component instantiation
/// **Why it is tested:** Ensures that all public framework components are accessible and can be instantiated without errors
/// **Test conditions:** Clean test environment with framework components available
/// **Expectations:** All framework components should be instantiable without compilation or runtime errors
#[test]
fn test_framework_api() {
    // Test that the new framework API is accessible
    let _test = TestCase::new("example");
    let _repo = TestRepo::builder();
    let _cmd = TestCommand::new();
}

/// **What is tested:** Basic framework usage with repository creation and file/pattern setup
/// **Why it is tested:** Validates that the framework can create test repositories with files and patterns correctly
/// **Test conditions:** Basic test data with files and gitignore patterns
/// **Expectations:** Repository creation should succeed without errors
#[test]
fn test_framework_basic_usage() {
    let files: Vec<(String, String)> = TestData::BASIC_FILES
        .iter()
        .map(|(path, content)| (path.to_string(), content.unwrap_or("").to_string()))
        .collect();

    let patterns: Vec<String> = TestData::BASIC_PATTERNS
        .iter()
        .map(|s| s.to_string())
        .collect();

    let repo = TestRepo::builder()
        .with_patterns(patterns)
        .with_files(files)
        .build();

    assert!(repo.is_ok());
}

/// **What is tested:** TestCommand builder functionality with arguments and input
/// **Why it is tested:** Ensures that the command builder can properly construct commands with arguments
/// **Test conditions:** TestCommand with --help argument and test input
/// **Expectations:** Command should build correctly with proper argument count
#[test]
fn test_command_builder() {
    let cmd = TestCommand::new().arg("--help").input("test");

    // Test that it builds correctly using the public getter method
    assert_eq!(cmd.args_len(), 1);
}

/// **What is tested:** Expectation validation system with mock command output
/// **Why it is tested:** Verifies that the framework can validate command expectations correctly
/// **Test conditions:** Mock successful command output using echo command
/// **Expectations:** Expectation validation should work correctly with successful command output
#[test]
fn test_expectation_validation() {
    // Create a mock successful output for testing
    // We'll use a simpler approach that works cross-platform
    let mock_output = std::process::Command::new("echo")
        .arg("success")
        .output()
        .unwrap();

    let output = CommandOutput::from_output(mock_output);

    let expectation = Expectation::Success;
    assert!(expectation.validate(&output).is_ok());

    let contains_expectation = Expectation::Contains("success".to_string());
    assert!(contains_expectation.validate(&output).is_ok());
}
