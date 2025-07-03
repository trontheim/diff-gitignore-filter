//! Extended test utilities for complex test scenarios
//!
//! This module provides additional helper functions and utilities
//! that extend the core framework for specialized testing needs.

use std::path::Path;
use std::process::Command;
use std::sync::Mutex;

use crate::common::framework::{TestRepo, TestRepoBuilder};

/// Global mutex to ensure thread-safe directory changes in tests
/// This prevents race conditions when multiple tests change the current working directory
static DIRECTORY_CHANGE_MUTEX: Mutex<()> = Mutex::new(());

/// Thread-safe wrapper for changing the current working directory in tests
///
/// This function ensures that only one test at a time can change the current working directory,
/// preventing race conditions that can occur when multiple tests run in parallel.
///
/// # Arguments
/// * `new_dir` - The directory to change to
/// * `operation` - A closure that will be executed with the changed directory
///
/// # Returns
/// The result of the operation closure
///
/// # Example
/// ```rust
/// use std::path::Path;
///
/// let result = with_directory_change(Path::new("/tmp"), || {
///     // Your test code that needs to run in /tmp
///     std::env::current_dir()
/// })?;
/// ```
#[allow(dead_code)]
pub fn with_directory_change<T, F, E>(new_dir: &Path, operation: F) -> Result<T, E>
where
    F: FnOnce() -> Result<T, E>,
    E: From<std::io::Error>,
{
    // Acquire the mutex to ensure thread safety
    let _guard = DIRECTORY_CHANGE_MUTEX.lock().unwrap();

    // Save the original directory
    let original_dir = std::env::current_dir().map_err(E::from)?;

    // Change to the new directory
    std::env::set_current_dir(new_dir).map_err(E::from)?;

    // Execute the operation and ensure we restore the directory even if it panics
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation));

    // Always restore the original directory
    let restore_result = std::env::set_current_dir(original_dir);

    // Handle the results
    match result {
        Ok(op_result) => {
            // If restore failed, return that error
            restore_result.map_err(E::from)?;
            op_result
        }
        Err(panic_payload) => {
            // If restore failed, log it but still propagate the panic
            if let Err(e) = restore_result {
                eprintln!("Warning: Failed to restore directory after panic: {e}");
            }
            std::panic::resume_unwind(panic_payload);
        }
    }
}

/// Convenience function for operations that return `Box<dyn std::error::Error>`
///
/// This is a specialized version of `with_directory_change` for the common case
/// where operations return `Box<dyn std::error::Error>`.
#[allow(dead_code)]
pub fn with_directory_change_boxed<T, F>(
    new_dir: &Path,
    operation: F,
) -> Result<T, Box<dyn std::error::Error>>
where
    F: FnOnce() -> Result<T, Box<dyn std::error::Error>>,
{
    with_directory_change(new_dir, operation)
}

/// Advanced test repository builder with preset configurations
pub struct AdvancedTestRepo;

impl AdvancedTestRepo {
    /// Create a Rust project structure with Cargo.toml and standard directories
    pub fn rust_project() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "target/".to_string(),
                "Cargo.lock".to_string(),
                "*.tmp".to_string(),
                ".env".to_string(),
            ])
            .with_files(vec![
                ("Cargo.toml".to_string(), Self::cargo_toml_content()),
                ("src/main.rs".to_string(), "fn main() {\n    println!(\"Hello, world!\");\n}".to_string()),
                ("src/lib.rs".to_string(), "//! Library crate\n\npub mod utils;\n".to_string()),
                ("src/utils.rs".to_string(), "//! Utility functions\n\npub fn helper() -> String {\n    \"helper\".to_string()\n}\n".to_string()),
                ("tests/integration.rs".to_string(), "#[test]\nfn integration_test() {\n    assert_eq!(2 + 2, 4);\n}\n".to_string()),
                ("README.md".to_string(), "# Test Project\n\nA test project for framework validation.\n".to_string()),
            ])
    }

    /// Create a Node.js project structure with package.json and standard directories
    pub fn nodejs_project() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "node_modules/".to_string(),
                "*.log".to_string(),
                ".env".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
            ])
            .with_files(vec![
                ("package.json".to_string(), Self::package_json_content()),
                ("index.js".to_string(), "console.log('Hello, Node.js!');\n".to_string()),
                ("src/app.js".to_string(), "const express = require('express');\nconst app = express();\n\nmodule.exports = app;\n".to_string()),
                ("test/app.test.js".to_string(), "const app = require('../src/app');\n\ndescribe('App', () => {\n  it('should exist', () => {\n    expect(app).toBeDefined();\n  });\n});\n".to_string()),
                ("README.md".to_string(), "# Node.js Test Project\n\nA Node.js project for testing.\n".to_string()),
            ])
    }

    /// Create a Python project structure with setup.py and standard directories
    pub fn python_project() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "__pycache__/".to_string(),
                "*.pyc".to_string(),
                "*.pyo".to_string(),
                ".pytest_cache/".to_string(),
                "venv/".to_string(),
                ".env".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
                "*.egg-info/".to_string(),
            ])
            .with_files(vec![
                ("setup.py".to_string(), Self::setup_py_content()),
                ("requirements.txt".to_string(), "pytest>=6.0\nrequests>=2.25\n".to_string()),
                ("src/__init__.py".to_string(), "\"\"\"Test package\"\"\"\n__version__ = \"0.1.0\"\n".to_string()),
                ("src/main.py".to_string(), "\"\"\"Main module\"\"\"\n\ndef main():\n    print(\"Hello, Python!\")\n\nif __name__ == \"__main__\":\n    main()\n".to_string()),
                ("tests/__init__.py".to_string(), "".to_string()),
                ("tests/test_main.py".to_string(), "\"\"\"Tests for main module\"\"\"\nimport pytest\nfrom src.main import main\n\ndef test_main():\n    # Test that main runs without error\n    main()\n".to_string()),
                ("README.md".to_string(), "# Python Test Project\n\nA Python project for testing.\n".to_string()),
            ])
    }

    /// Create a multi-language project with mixed file types
    pub fn polyglot_project() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "target/".to_string(),
                "node_modules/".to_string(),
                "__pycache__/".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                ".env".to_string(),
                "dist/".to_string(),
                "build/".to_string(),
            ])
            .with_files(vec![
                // Rust files
                ("Cargo.toml".to_string(), Self::cargo_toml_content()),
                (
                    "src/main.rs".to_string(),
                    "fn main() { println!(\"Rust\"); }".to_string(),
                ),
                // Node.js files
                ("package.json".to_string(), Self::package_json_content()),
                (
                    "index.js".to_string(),
                    "console.log('Node.js');".to_string(),
                ),
                // Python files
                ("setup.py".to_string(), Self::setup_py_content()),
                ("main.py".to_string(), "print('Python')".to_string()),
                // Go files
                ("go.mod".to_string(), "module test\n\ngo 1.19\n".to_string()),
                (
                    "main.go".to_string(),
                    "package main\n\nimport \"fmt\"\n\nfunc main() {\n\tfmt.Println(\"Go\")\n}\n"
                        .to_string(),
                ),
                // Documentation
                (
                    "README.md".to_string(),
                    "# Polyglot Project\n\nA multi-language project for testing.\n".to_string(),
                ),
            ])
    }

    fn cargo_toml_content() -> String {
        r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
tokio = { version = "1.0", features = ["full"] }

[dev-dependencies]
tempfile = "3.0"
"#
        .to_string()
    }

    fn package_json_content() -> String {
        r#"{
  "name": "test-project",
  "version": "1.0.0",
  "description": "A test Node.js project",
  "main": "index.js",
  "scripts": {
    "start": "node index.js",
    "test": "jest"
  },
  "dependencies": {
    "express": "^4.18.0"
  },
  "devDependencies": {
    "jest": "^28.0.0"
  }
}
"#
        .to_string()
    }

    fn setup_py_content() -> String {
        r#"from setuptools import setup, find_packages

setup(
    name="test-project",
    version="0.1.0",
    description="A test Python project",
    packages=find_packages(),
    install_requires=[
        "requests>=2.25.0",
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "black>=21.0",
            "flake8>=3.8",
        ]
    },
    python_requires=">=3.8",
)
"#
        .to_string()
    }

    /// Create a multi-repository environment with nested repositories
    pub fn multi_repo_environment() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "*.log".to_string(),
                "*.tmp".to_string(),
                "target/".to_string(),
                "node_modules/".to_string(),
                ".env".to_string(),
            ])
            .with_files(vec![
                ("main.rs".to_string(), "fn main() {}".to_string()),
                ("debug.log".to_string(), "main repo log".to_string()),
                (
                    "subproject/.gitignore".to_string(),
                    "*.cache\n*.build\ndist/".to_string(),
                ),
                (
                    "subproject/lib.rs".to_string(),
                    "pub fn lib() {}".to_string(),
                ),
                (
                    "subproject/cache.cache".to_string(),
                    "cached data".to_string(),
                ),
            ])
    }

    /// Create a repository with complex directory structure and patterns
    pub fn complex_directory_structure() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "# Complex patterns".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                "!important.log".to_string(),
                "/build/".to_string(),
                "**/cache/".to_string(),
                "level*/level*/ignored.*".to_string(),
                "dir with spaces/*.tmp".to_string(),
                "**/deep_file.rs".to_string(),
            ])
            .with_files(vec![
                (
                    "level1/level2/level3/level4/level5/deep_file.rs".to_string(),
                    "// Deep file".to_string(),
                ),
                (
                    "level1/level2/level3/level4/level5/ignored.log".to_string(),
                    "deep log".to_string(),
                ),
                (
                    "dir with spaces/file.txt".to_string(),
                    "content".to_string(),
                ),
                (
                    "dir-with-dashes/file.txt".to_string(),
                    "content".to_string(),
                ),
                (
                    "dir_with_underscores/file.txt".to_string(),
                    "content".to_string(),
                ),
                ("dir.with.dots/file.txt".to_string(), "content".to_string()),
                ("UPPERCASE_DIR/file.txt".to_string(), "content".to_string()),
            ])
    }

    /// Create a repository with Unicode filenames and content
    pub fn unicode_test_repo() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "файл.txt".to_string(),
                "*.log".to_string(),
                "🚀*".to_string(),
                "café_*".to_string(),
            ])
            .with_files(vec![
                ("файл.txt".to_string(), "Content of файл.txt".to_string()),
                ("文件.txt".to_string(), "Content of 文件.txt".to_string()),
                (
                    "ファイル.txt".to_string(),
                    "Content of ファイル.txt".to_string(),
                ),
                (
                    "αρχείο.txt".to_string(),
                    "Content of αρχείο.txt".to_string(),
                ),
                (
                    "🚀rocket.txt".to_string(),
                    "Content of 🚀rocket.txt".to_string(),
                ),
                (
                    "café_résumé.txt".to_string(),
                    "Content of café_résumé.txt".to_string(),
                ),
            ])
    }

    /// Create a test repository with VCS directories and files
    pub fn vcs_test_repo() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                "*.log".to_string(),
                "*.tmp".to_string(),
                "target/".to_string(),
            ])
            .with_files(vec![
                (
                    ".git/config".to_string(),
                    "[core]\n    bare = false\n".to_string(),
                ),
                (".svn/entries".to_string(), "12\n".to_string()),
                ("_svn/entries".to_string(), "12\n".to_string()),
                (
                    ".hg/hgrc".to_string(),
                    "[ui]\nusername = test\n".to_string(),
                ),
                ("CVS/Entries".to_string(), "D/src////\n".to_string()),
                ("CVSROOT/config".to_string(), "SystemAuth=no\n".to_string()),
                (
                    ".bzr/branch-format".to_string(),
                    "Bazaar-NG meta directory, format 1\n".to_string(),
                ),
                ("src/main.rs".to_string(), "fn main() {}\n".to_string()),
                ("README.md".to_string(), "# Test Project\n".to_string()),
                ("debug.log".to_string(), "log content\n".to_string()),
                ("target/debug/main".to_string(), "binary\n".to_string()),
            ])
    }

    /// Create a repository with unusual directory structures
    pub fn unusual_directory_structure() -> TestRepoBuilder {
        TestRepoBuilder::new()
            .with_patterns(vec![
                ".hidden".to_string(),
                "..double_hidden".to_string(),
                "*.log".to_string(),
            ])
            .with_files(vec![
                (".hidden/file.txt".to_string(), "content".to_string()),
                (
                    "..double_hidden/file.txt".to_string(),
                    "content".to_string(),
                ),
                ("dir./file.txt".to_string(), "content".to_string()),
                ("dir../file.txt".to_string(), "content".to_string()),
                (
                    ".dir.with.many.dots/file.txt".to_string(),
                    "content".to_string(),
                ),
                (
                    "123numeric_start/file.txt".to_string(),
                    "content".to_string(),
                ),
                ("ALLCAPS/file.txt".to_string(), "content".to_string()),
                ("mixedCASE/file.txt".to_string(), "content".to_string()),
                ("under_score/file.txt".to_string(), "content".to_string()),
                ("hyphen-ated/file.txt".to_string(), "content".to_string()),
            ])
    }

    /// Create a simple test repository with custom gitignore content
    pub fn custom_gitignore_repo(gitignore_content: &str) -> TestRepoBuilder {
        let patterns: Vec<String> = gitignore_content.lines().map(|s| s.to_string()).collect();
        TestRepoBuilder::new().with_patterns(patterns)
    }

    /// Create a basic test git repository with minimal setup
    pub fn minimal_git_repo() -> TestRepoBuilder {
        TestRepoBuilder::new().with_git_configs(vec![
            ("user.name", "Test User"),
            ("user.email", "test@example.com"),
        ])
    }
}

/// Test scenario builder for complex integration tests
pub struct TestScenario {
    name: String,
    repos: Vec<TestRepo>,
    commands: Vec<String>,
    expected_outputs: Vec<String>,
}

impl TestScenario {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            repos: Vec::new(),
            commands: Vec::new(),
            expected_outputs: Vec::new(),
        }
    }

    pub fn with_repo(mut self, repo: TestRepo) -> Self {
        self.repos.push(repo);
        self
    }

    pub fn with_command(mut self, command: &str) -> Self {
        self.commands.push(command.to_string());
        self
    }

    pub fn expect_output(mut self, output: &str) -> Self {
        self.expected_outputs.push(output.to_string());
        self
    }

    #[allow(dead_code)]
    pub fn run(self) -> Result<(), Box<dyn std::error::Error>> {
        println!("Running test scenario: {}", self.name);

        for (i, repo) in self.repos.iter().enumerate() {
            println!("  Repository {}: {}", i + 1, repo.path().display());
        }

        for (i, command) in self.commands.iter().enumerate() {
            println!("  Command {}: {}", i + 1, command);
        }

        // Here you would implement the actual test execution logic
        // For now, we just validate that the setup is correct

        Ok(())
    }
}

/// Utility functions for test validation
pub struct TestValidation;

impl TestValidation {
    /// Validate that a repository has the expected structure
    pub fn validate_repo_structure(
        repo_path: &Path,
        expected_files: &[&str],
    ) -> Result<(), String> {
        for file in expected_files {
            let file_path = repo_path.join(file);
            if !file_path.exists() {
                return Err(format!("Expected file not found: {file}"));
            }
        }
        Ok(())
    }

    /// Validate that gitignore patterns are working correctly
    pub fn validate_gitignore_patterns(repo_path: &Path, patterns: &[&str]) -> Result<(), String> {
        let gitignore_path = repo_path.join(".gitignore");
        if !gitignore_path.exists() {
            return Err("No .gitignore file found".to_string());
        }

        let content = std::fs::read_to_string(gitignore_path)
            .map_err(|e| format!("Failed to read .gitignore: {e}"))?;

        for pattern in patterns {
            if !content.contains(pattern) {
                return Err(format!("Pattern not found in .gitignore: {pattern}"));
            }
        }

        Ok(())
    }

    /// Validate git repository initialization
    pub fn validate_git_init(repo_path: &Path) -> Result<(), String> {
        let git_dir = repo_path.join(".git");
        if !git_dir.exists() {
            return Err("Git repository not initialized".to_string());
        }

        // Check if git config is set
        let output = Command::new("git")
            .args(["config", "user.name"])
            .current_dir(repo_path)
            .output()
            .map_err(|e| format!("Failed to check git config: {e}"))?;

        if !output.status.success() {
            return Err("Git user.name not configured".to_string());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// **What is tested:** Rust project creation with proper file structure and gitignore patterns
    /// **Why it is tested:** Validates that the Rust project preset creates all expected files and configurations
    /// **Test conditions:** AdvancedTestRepo with Rust project preset
    /// **Expectations:** Should create Cargo.toml, source files, test files, and proper gitignore patterns
    #[test]
    fn test_rust_project_creation() {
        let repo = AdvancedTestRepo::rust_project().build().unwrap();

        // Validate expected files exist
        assert!(repo.path().join("Cargo.toml").exists());
        assert!(repo.path().join("src/main.rs").exists());
        assert!(repo.path().join("src/lib.rs").exists());
        assert!(repo.path().join("src/utils.rs").exists());
        assert!(repo.path().join("tests/integration.rs").exists());

        // Validate gitignore patterns
        TestValidation::validate_gitignore_patterns(
            repo.path(),
            &["target/", "Cargo.lock", "*.tmp"],
        )
        .unwrap();
    }

    /// **What is tested:** Node.js project creation with proper file structure and dependencies
    /// **Why it is tested:** Validates that the Node.js project preset creates all expected files and gitignore patterns
    /// **Test conditions:** AdvancedTestRepo with Node.js project preset
    /// **Expectations:** Should create package.json, JavaScript files, and proper Node.js gitignore patterns
    #[test]
    fn test_nodejs_project_creation() {
        let repo = AdvancedTestRepo::nodejs_project().build().unwrap();

        // Validate expected files exist
        assert!(repo.path().join("package.json").exists());
        assert!(repo.path().join("index.js").exists());
        assert!(repo.path().join("src/app.js").exists());

        // Validate gitignore patterns
        TestValidation::validate_gitignore_patterns(
            repo.path(),
            &["node_modules/", "*.log", "dist/"],
        )
        .unwrap();
    }

    /// **What is tested:** Python project creation with proper file structure and setup configuration
    /// **Why it is tested:** Validates that the Python project preset creates all expected files and gitignore patterns
    /// **Test conditions:** AdvancedTestRepo with Python project preset
    /// **Expectations:** Should create setup.py, Python source files, and proper Python gitignore patterns
    #[test]
    fn test_python_project_creation() {
        let repo = AdvancedTestRepo::python_project().build().unwrap();

        // Validate expected files exist
        assert!(repo.path().join("setup.py").exists());
        assert!(repo.path().join("src/__init__.py").exists());
        assert!(repo.path().join("src/main.py").exists());
        assert!(repo.path().join("tests/test_main.py").exists());

        // Validate gitignore patterns
        TestValidation::validate_gitignore_patterns(
            repo.path(),
            &["__pycache__/", "*.pyc", "venv/"],
        )
        .unwrap();
    }

    /// **What is tested:** Polyglot project creation with multiple programming languages
    /// **Why it is tested:** Validates that the polyglot preset creates files for all supported languages with comprehensive gitignore
    /// **Test conditions:** AdvancedTestRepo with polyglot project preset
    /// **Expectations:** Should create files for Rust, Node.js, Python, Go and comprehensive gitignore patterns
    #[test]
    fn test_polyglot_project_creation() {
        let repo = AdvancedTestRepo::polyglot_project().build().unwrap();

        // Validate files from all languages exist
        assert!(repo.path().join("Cargo.toml").exists());
        assert!(repo.path().join("package.json").exists());
        assert!(repo.path().join("setup.py").exists());
        assert!(repo.path().join("go.mod").exists());

        // Validate comprehensive gitignore patterns
        TestValidation::validate_gitignore_patterns(
            repo.path(),
            &["target/", "node_modules/", "__pycache__/", "*.log"],
        )
        .unwrap();
    }

    /// **What is tested:** TestScenario builder functionality with commands and expectations
    /// **Why it is tested:** Validates that test scenarios can be built with repositories, commands, and output expectations
    /// **Test conditions:** Rust project repository with cargo commands and expected outputs
    /// **Expectations:** Scenario should build correctly with proper name, commands, and expectations
    #[test]
    fn test_scenario_builder() {
        let repo = AdvancedTestRepo::rust_project().build().unwrap();

        let scenario = TestScenario::new("test_rust_build")
            .with_repo(repo)
            .with_command("cargo check")
            .with_command("cargo test")
            .expect_output("Finished")
            .expect_output("test result: ok");

        // Just test that the scenario can be built
        assert_eq!(scenario.name, "test_rust_build");
        assert_eq!(scenario.commands.len(), 2);
        assert_eq!(scenario.expected_outputs.len(), 2);
    }

    /// **What is tested:** TestValidation utilities for repository structure and gitignore pattern validation
    /// **Why it is tested:** Ensures that validation utilities can properly verify repository components and configurations
    /// **Test conditions:** Rust project repository with expected files and gitignore patterns
    /// **Expectations:** Validation should succeed for proper repository structure and gitignore patterns
    #[test]
    fn test_validation_utilities() {
        let repo = AdvancedTestRepo::rust_project().build().unwrap();

        // Test structure validation
        TestValidation::validate_repo_structure(
            repo.path(),
            &["Cargo.toml", "src/main.rs", ".gitignore"],
        )
        .unwrap();

        // Test git initialization validation
        TestValidation::validate_git_init(repo.path()).unwrap();
    }
}

/// **What is tested:** Specialized repository presets including multi-repo, complex directories, Unicode, and VCS structures
/// **Why it is tested:** Validates that advanced repository presets can create complex test environments with special cases
/// **Test conditions:** Various specialized repository configurations with complex structures and Unicode filenames
/// **Expectations:** All specialized presets should create expected files and directory structures correctly
#[test]
fn test_specialized_repo_presets() {
    // Test multi-repo environment
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();
    assert!(repo.path().join("main.rs").exists());
    assert!(repo.path().join("subproject/lib.rs").exists());
    assert!(repo.path().join("subproject/.gitignore").exists());

    // Test complex directory structure
    let repo = AdvancedTestRepo::complex_directory_structure()
        .build()
        .unwrap();
    assert!(repo
        .path()
        .join("level1/level2/level3/level4/level5/deep_file.rs")
        .exists());
    assert!(repo.path().join("dir with spaces/file.txt").exists());
    assert!(repo.path().join("UPPERCASE_DIR/file.txt").exists());

    // Test Unicode repository
    let repo = AdvancedTestRepo::unicode_test_repo().build().unwrap();
    assert!(repo.path().join("файл.txt").exists());
    assert!(repo.path().join("🚀rocket.txt").exists());
    assert!(repo.path().join("café_résumé.txt").exists());

    // Test VCS repository
    let repo = AdvancedTestRepo::vcs_test_repo().build().unwrap();
    assert!(repo.path().join(".git/config").exists());
    assert!(repo.path().join(".svn/entries").exists());
    assert!(repo.path().join("CVS/Entries").exists());

    // Test unusual directory structure
    let repo = AdvancedTestRepo::unusual_directory_structure()
        .build()
        .unwrap();
    assert!(repo.path().join(".hidden/file.txt").exists());
    assert!(repo.path().join("123numeric_start/file.txt").exists());
    assert!(repo.path().join("hyphen-ated/file.txt").exists());
}

/// **What is tested:** Git configuration integration with setting, reading, and cleanup operations
/// **Why it is tested:** Validates that git config operations work correctly within test repositories
/// **Test conditions:** Minimal git repository with config set/get/unset operations
/// **Expectations:** Should successfully set, read, and clean up git configuration values
#[test]
fn test_git_config_integration() {
    let repo = AdvancedTestRepo::minimal_git_repo().build().unwrap();

    // Test setting git config
    repo.set_git_config("test.key", "test.value").unwrap();

    // Test reading git config
    let output = repo.git_config_command(&["--get", "test.key"]).unwrap();
    let output_str = String::from_utf8_lossy(&output.stdout);
    let value = output_str.trim();
    assert_eq!(value, "test.value");

    // Test cleanup
    repo.unset_git_config("test.key");

    // Verify cleanup worked
    let output = repo.git_config_command(&["--get", "test.key"]).unwrap();
    assert!(!output.status.success());
}

/// **What is tested:** Git configuration with automatic cleanup functionality
/// **Why it is tested:** Ensures that git config cleanup mechanisms work properly to avoid test pollution
/// **Test conditions:** Minimal git repository with automatic cleanup scope testing
/// **Expectations:** Config should be set during scope and automatically cleaned up when scope ends
#[test]
fn test_git_config_with_cleanup() {
    let repo = AdvancedTestRepo::minimal_git_repo().build().unwrap();

    // Test automatic cleanup
    {
        let cleanup = repo
            .set_git_config_with_cleanup("test.cleanup", "cleanup.value")
            .unwrap();

        // Verify config is set
        let output = repo.git_config_command(&["--get", "test.cleanup"]).unwrap();
        let output_str = String::from_utf8_lossy(&output.stdout);
        let value = output_str.trim();
        assert_eq!(value, "cleanup.value");

        // Call cleanup
        cleanup();
    }

    // Verify config was cleaned up
    let output = repo.git_config_command(&["--get", "test.cleanup"]).unwrap();
    assert!(!output.status.success());
}

/// **What is tested:** Custom gitignore repository creation with specific patterns
/// **Why it is tested:** Validates that repositories can be created with custom gitignore patterns for specialized testing
/// **Test conditions:** Repository with custom gitignore patterns including wildcards and negation
/// **Expectations:** Should create repository with gitignore file containing all specified custom patterns
#[test]
fn test_custom_gitignore_repo() {
    let custom_patterns = "*.custom\n/build/\n!important.custom";
    let repo = AdvancedTestRepo::custom_gitignore_repo(custom_patterns)
        .build()
        .unwrap();

    let gitignore_content = std::fs::read_to_string(repo.path().join(".gitignore")).unwrap();
    assert!(gitignore_content.contains("*.custom"));
    assert!(gitignore_content.contains("/build/"));
    assert!(gitignore_content.contains("!important.custom"));
}
