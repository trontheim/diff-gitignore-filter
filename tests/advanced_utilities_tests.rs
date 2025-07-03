//! Tests for advanced test utilities
//!
//! This module tests the extended functionality provided by the test_utilities module,
//! including preset project configurations and validation utilities.

mod common;

use common::test_utilities::{AdvancedTestRepo, TestScenario, TestValidation};

/// **What is tested:** Rust project preset creation and structure validation
/// **Why it is tested:** Ensures the Rust project preset creates a complete, valid Rust project structure with proper dependencies
/// **Test conditions:** Creates a Rust project preset and validates file structure, Cargo.toml content, and gitignore patterns
/// **Expectations:** Should create all expected Rust project files with correct content and proper gitignore configuration
#[test]
fn test_rust_project_preset() {
    let repo = AdvancedTestRepo::rust_project().build().unwrap();

    // Verify Rust project structure
    assert!(repo.path().join("Cargo.toml").exists());
    assert!(repo.path().join("src/main.rs").exists());
    assert!(repo.path().join("src/lib.rs").exists());
    assert!(repo.path().join("src/utils.rs").exists());
    assert!(repo.path().join("tests/integration.rs").exists());
    assert!(repo.path().join("README.md").exists());

    // Verify Cargo.toml content
    let cargo_content = std::fs::read_to_string(repo.path().join("Cargo.toml")).unwrap();
    assert!(cargo_content.contains("name = \"test-project\""));
    assert!(cargo_content.contains("edition = \"2021\""));
    assert!(cargo_content.contains("serde"));
    assert!(cargo_content.contains("tokio"));

    // Verify gitignore patterns
    TestValidation::validate_gitignore_patterns(
        repo.path(),
        &["target/", "Cargo.lock", "*.tmp", ".env"],
    )
    .unwrap();

    // Verify git initialization
    TestValidation::validate_git_init(repo.path()).unwrap();
}

/// **What is tested:** Node.js project preset creation and structure validation
/// **Why it is tested:** Ensures the Node.js project preset creates a complete, valid Node.js project with proper dependencies
/// **Test conditions:** Creates a Node.js project preset and validates file structure, package.json content, and gitignore patterns
/// **Expectations:** Should create all expected Node.js project files with correct dependencies and proper gitignore configuration
#[test]
fn test_nodejs_project_preset() {
    let repo = AdvancedTestRepo::nodejs_project().build().unwrap();

    // Verify Node.js project structure
    assert!(repo.path().join("package.json").exists());
    assert!(repo.path().join("index.js").exists());
    assert!(repo.path().join("src/app.js").exists());
    assert!(repo.path().join("test/app.test.js").exists());
    assert!(repo.path().join("README.md").exists());

    // Verify package.json content
    let package_content = std::fs::read_to_string(repo.path().join("package.json")).unwrap();
    assert!(package_content.contains("\"name\": \"test-project\""));
    assert!(package_content.contains("\"express\""));
    assert!(package_content.contains("\"jest\""));

    // Verify gitignore patterns
    TestValidation::validate_gitignore_patterns(
        repo.path(),
        &["node_modules/", "*.log", ".env", "dist/", "build/"],
    )
    .unwrap();
}

/// **What is tested:** Python project preset creation and structure validation
/// **Why it is tested:** Ensures the Python project preset creates a complete, valid Python project with proper dependencies
/// **Test conditions:** Creates a Python project preset and validates file structure, setup.py content, and gitignore patterns
/// **Expectations:** Should create all expected Python project files with correct dependencies and proper gitignore configuration
#[test]
fn test_python_project_preset() {
    let repo = AdvancedTestRepo::python_project().build().unwrap();

    // Verify Python project structure
    assert!(repo.path().join("setup.py").exists());
    assert!(repo.path().join("requirements.txt").exists());
    assert!(repo.path().join("src/__init__.py").exists());
    assert!(repo.path().join("src/main.py").exists());
    assert!(repo.path().join("tests/__init__.py").exists());
    assert!(repo.path().join("tests/test_main.py").exists());
    assert!(repo.path().join("README.md").exists());

    // Verify setup.py content
    let setup_content = std::fs::read_to_string(repo.path().join("setup.py")).unwrap();
    assert!(setup_content.contains("name=\"test-project\""));
    assert!(setup_content.contains("requests"));
    assert!(setup_content.contains("pytest"));

    // Verify gitignore patterns
    TestValidation::validate_gitignore_patterns(
        repo.path(),
        &["__pycache__/", "*.pyc", "*.pyo", ".pytest_cache/", "venv/"],
    )
    .unwrap();
}

/// **What is tested:** Multi-language polyglot project preset creation and validation
/// **Why it is tested:** Ensures the polyglot preset can create projects with multiple programming languages coexisting
/// **Test conditions:** Creates a polyglot project with Rust, Node.js, Python, and Go components
/// **Expectations:** Should create files for all languages with comprehensive gitignore patterns covering all ecosystems
#[test]
fn test_polyglot_project_preset() {
    let repo = AdvancedTestRepo::polyglot_project().build().unwrap();

    // Verify all language files exist
    assert!(repo.path().join("Cargo.toml").exists()); // Rust
    assert!(repo.path().join("package.json").exists()); // Node.js
    assert!(repo.path().join("setup.py").exists()); // Python
    assert!(repo.path().join("go.mod").exists()); // Go
    assert!(repo.path().join("src/main.rs").exists()); // Rust
    assert!(repo.path().join("index.js").exists()); // Node.js
    assert!(repo.path().join("main.py").exists()); // Python
    assert!(repo.path().join("main.go").exists()); // Go

    // Verify comprehensive gitignore patterns
    TestValidation::validate_gitignore_patterns(
        repo.path(),
        &[
            "target/",
            "node_modules/",
            "__pycache__/",
            "*.log",
            "*.tmp",
            ".env",
        ],
    )
    .unwrap();
}

/// **What is tested:** Test scenario builder functionality for multi-repository testing
/// **Why it is tested:** Validates that complex test scenarios can be built and executed across multiple repositories
/// **Test conditions:** Creates multiple repositories and builds a scenario with commands and expected outputs
/// **Expectations:** Should successfully build and execute test scenarios with multiple repositories and commands
#[test]
fn test_scenario_builder() {
    let rust_repo = AdvancedTestRepo::rust_project().build().unwrap();
    let nodejs_repo = AdvancedTestRepo::nodejs_project().build().unwrap();

    let scenario = TestScenario::new("multi_language_test")
        .with_repo(rust_repo)
        .with_repo(nodejs_repo)
        .with_command("cargo check")
        .with_command("npm test")
        .expect_output("Finished")
        .expect_output("PASS");

    // Test that scenario can be executed (basic validation)
    scenario.run().unwrap();
}

/// **What is tested:** Test validation utility functions for repository structure and configuration
/// **Why it is tested:** Ensures validation utilities correctly identify valid and invalid repository configurations
/// **Test conditions:** Tests both successful and failed validations for repository structure, gitignore patterns, and git initialization
/// **Expectations:** Should correctly validate repository components and provide meaningful error messages for failures
#[test]
fn test_validation_utilities() {
    let repo = AdvancedTestRepo::rust_project().build().unwrap();

    // Test successful structure validation
    TestValidation::validate_repo_structure(
        repo.path(),
        &["Cargo.toml", "src/main.rs", "src/lib.rs", ".gitignore"],
    )
    .unwrap();

    // Test failed structure validation
    let result = TestValidation::validate_repo_structure(repo.path(), &["nonexistent.file"]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Expected file not found"));

    // Test successful gitignore validation
    TestValidation::validate_gitignore_patterns(repo.path(), &["target/", "*.tmp"]).unwrap();

    // Test failed gitignore validation
    let result = TestValidation::validate_gitignore_patterns(repo.path(), &["nonexistent_pattern"]);
    assert!(result.is_err());
    assert!(result.unwrap_err().contains("Pattern not found"));

    // Test git initialization validation
    TestValidation::validate_git_init(repo.path()).unwrap();
}

/// **What is tested:** Quality and correctness of generated project file content
/// **Why it is tested:** Ensures that preset-generated files contain meaningful, syntactically correct content
/// **Test conditions:** Examines content of generated Rust project files (main.rs, lib.rs, utils.rs, tests)
/// **Expectations:** Generated files should contain proper syntax, meaningful content, and follow language conventions
#[test]
fn test_project_content_quality() {
    // Test that generated project files have reasonable content
    let repo = AdvancedTestRepo::rust_project().build().unwrap();

    // Check main.rs content
    let main_content = std::fs::read_to_string(repo.path().join("src/main.rs")).unwrap();
    assert!(main_content.contains("fn main()"));
    assert!(main_content.contains("println!"));

    // Check lib.rs content
    let lib_content = std::fs::read_to_string(repo.path().join("src/lib.rs")).unwrap();
    assert!(lib_content.contains("//!"));
    assert!(lib_content.contains("pub mod utils"));

    // Check utils.rs content
    let utils_content = std::fs::read_to_string(repo.path().join("src/utils.rs")).unwrap();
    assert!(utils_content.contains("pub fn helper"));
    assert!(utils_content.contains("String"));

    // Check test file content
    let test_content = std::fs::read_to_string(repo.path().join("tests/integration.rs")).unwrap();
    assert!(test_content.contains("#[test]"));
    assert!(test_content.contains("assert_eq!"));
}

/// **What is tested:** Compatibility and coexistence of different language project presets
/// **Why it is tested:** Ensures that different language presets can be created independently without conflicts
/// **Test conditions:** Creates separate Rust, Node.js, and Python projects and validates their independence
/// **Expectations:** All language presets should create valid git repositories with proper gitignore and README files
#[test]
fn test_cross_language_compatibility() {
    // Test that different language presets can coexist
    let rust_repo = AdvancedTestRepo::rust_project().build().unwrap();
    let nodejs_repo = AdvancedTestRepo::nodejs_project().build().unwrap();
    let python_repo = AdvancedTestRepo::python_project().build().unwrap();

    // All repos should be valid git repositories
    TestValidation::validate_git_init(rust_repo.path()).unwrap();
    TestValidation::validate_git_init(nodejs_repo.path()).unwrap();
    TestValidation::validate_git_init(python_repo.path()).unwrap();

    // All repos should have proper gitignore files
    assert!(rust_repo.path().join(".gitignore").exists());
    assert!(nodejs_repo.path().join(".gitignore").exists());
    assert!(python_repo.path().join(".gitignore").exists());

    // All repos should have README files
    assert!(rust_repo.path().join("README.md").exists());
    assert!(nodejs_repo.path().join("README.md").exists());
    assert!(python_repo.path().join("README.md").exists());
}

/// **What is tested:** Extensibility of project presets with custom files and patterns
/// **Why it is tested:** Validates that presets can be extended with additional files and gitignore patterns
/// **Test conditions:** Extends a Rust preset with custom files and patterns, then validates the additions
/// **Expectations:** Should preserve original preset structure while adding custom files and gitignore patterns
#[test]
fn test_preset_extensibility() {
    // Test that presets can be extended with additional files and patterns
    let repo = AdvancedTestRepo::rust_project()
        .with_patterns(vec!["custom_pattern/".to_string()])
        .with_files(vec![
            ("custom_file.txt".to_string(), "Custom content".to_string()),
            (
                "config/app.toml".to_string(),
                "[app]\nname = \"test\"".to_string(),
            ),
        ])
        .build()
        .unwrap();

    // Verify original preset files still exist
    assert!(repo.path().join("Cargo.toml").exists());
    assert!(repo.path().join("src/main.rs").exists());

    // Verify additional files were added
    assert!(repo.path().join("custom_file.txt").exists());
    assert!(repo.path().join("config/app.toml").exists());

    // Verify additional patterns were added to gitignore
    let gitignore_content = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("target/")); // Original pattern
    assert!(gitignore_content.contains("custom_pattern/")); // Added pattern

    // Verify custom file content
    let custom_content = std::fs::read_to_string(repo.path().join("custom_file.txt")).unwrap();
    assert_eq!(custom_content, "Custom content");
}
