//! Unified Test Framework
//!
//! This module provides a clean, consistent API for all testing needs.
//! All components follow unified design patterns and naming conventions.

use assert_cmd::Command as AssertCommand;
use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};
use std::sync::Once;
use tempfile::TempDir;

static BINARY_BUILD: Once = Once::new();

/// Ensure the binary is built before running tests
fn ensure_binary_built() {
    BINARY_BUILD.call_once(|| {
        let output = std::process::Command::new("cargo")
            .args(["build"])
            .output()
            .expect("Failed to execute cargo build");

        if !output.status.success() {
            panic!(
                "Failed to build binary: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
    });
}

/// Main result type for the framework
pub type Result<T = ()> = std::result::Result<T, Box<dyn Error>>;

/// Main test case orchestrator
#[derive(Debug)]
pub struct TestCase {
    #[allow(dead_code)]
    name: String,
    #[allow(dead_code)]
    repo: Option<TestRepo>,
    #[allow(dead_code)]
    command: Option<TestCommand>,
    #[allow(dead_code)]
    expectations: Vec<Expectation>,
}

impl TestCase {
    /// Create a new test case
    #[allow(dead_code)]
    pub fn new<S: AsRef<str>>(name: S) -> Self {
        Self {
            name: name.as_ref().to_string(),
            repo: None,
            command: None,
            expectations: Vec::new(),
        }
    }

    /// Set the repository for this test
    #[allow(dead_code)]
    pub fn with_repo(mut self, repo: TestRepo) -> Self {
        self.repo = Some(repo);
        self
    }

    /// Set the command for this test
    #[allow(dead_code)]
    pub fn with_command(mut self, command: TestCommand) -> Self {
        self.command = Some(command);
        self
    }

    /// Add an expectation
    #[allow(dead_code)]
    pub fn expect(mut self, expectation: Expectation) -> Self {
        self.expectations.push(expectation);
        self
    }

    /// Expect success
    #[allow(dead_code)]
    pub fn expect_success(self) -> Self {
        self.expect(Expectation::Success)
    }

    /// Expect failure
    #[allow(dead_code)]
    pub fn expect_failure(self) -> Self {
        self.expect(Expectation::Failure)
    }

    /// Expect output contains text
    #[allow(dead_code)]
    pub fn expect_contains<S: AsRef<str>>(self, text: S) -> Self {
        self.expect(Expectation::Contains(text.as_ref().to_string()))
    }

    /// Expect output excludes text
    #[allow(dead_code)]
    pub fn expect_excludes<S: AsRef<str>>(self, text: S) -> Self {
        self.expect(Expectation::Excludes(text.as_ref().to_string()))
    }

    /// Run the test case
    #[allow(dead_code)]
    pub fn run(self) -> Result<TestResult> {
        let mut command = self.command.unwrap_or_default();

        if let Some(repo) = &self.repo {
            command = command.in_dir(repo.path());
        }

        let result = command.execute()?;

        // Validate expectations
        for expectation in &self.expectations {
            expectation.validate(&result)?;
        }

        Ok(TestResult {
            name: self.name,
            success: true,
            output: result,
        })
    }
}

/// Test repository
#[derive(Debug)]
pub struct TestRepo {
    #[allow(dead_code)]
    temp_dir: TempDir,
    path: PathBuf,
}

impl TestRepo {
    /// Create a new repository builder
    #[allow(dead_code)]
    pub fn builder() -> TestRepoBuilder {
        TestRepoBuilder::new()
    }

    /// Get the path to the repository
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// Convert this TestRepo into a TempDir for legacy compatibility
    #[allow(dead_code)]
    pub fn into_temp_dir(self) -> TempDir {
        self.temp_dir
    }

    /// Set a git config value in this repository
    pub fn set_git_config<K: AsRef<str>, V: AsRef<str>>(&self, key: K, value: V) -> Result<()> {
        Command::new("git")
            .current_dir(&self.path)
            .args(["config", key.as_ref(), value.as_ref()])
            .output()?;
        Ok(())
    }

    /// Unset a git config value in this repository
    pub fn unset_git_config<K: AsRef<str>>(&self, key: K) {
        Command::new("git")
            .current_dir(&self.path)
            .args(["config", "--unset", key.as_ref()])
            .output()
            .ok(); // Ignore errors during cleanup
    }

    /// Execute a git config command with custom arguments
    pub fn git_config_command(&self, args: &[&str]) -> Result<std::process::Output> {
        let output = Command::new("git")
            .current_dir(&self.path)
            .arg("config")
            .args(args)
            .output()?;
        Ok(output)
    }

    /// Set git config with automatic cleanup function
    pub fn set_git_config_with_cleanup<K: AsRef<str>, V: AsRef<str>>(
        &self,
        key: K,
        value: V,
    ) -> Result<impl FnOnce()> {
        self.set_git_config(key.as_ref(), value.as_ref())?;

        let path = self.path.clone();
        let key = key.as_ref().to_string();

        Ok(move || {
            Command::new("git")
                .current_dir(&path)
                .args(["config", "--unset", &key])
                .output()
                .ok();
        })
    }
}

/// Repository builder
#[derive(Debug)]
pub struct TestRepoBuilder {
    patterns: Vec<String>,
    files: Vec<(String, String)>,
    git_config: Vec<(String, String)>,
}

impl TestRepoBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
            files: Vec::new(),
            git_config: Vec::new(),
        }
    }

    /// Add gitignore patterns
    pub fn with_patterns<I, S>(mut self, patterns: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        self.patterns
            .extend(patterns.into_iter().map(|s| s.as_ref().to_string()));
        self
    }

    /// Add files
    pub fn with_files<I>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = (String, String)>,
    {
        self.files.extend(files);
        self
    }

    /// Add files from static data
    #[allow(dead_code)]
    pub fn with_static_files<I>(mut self, files: I) -> Self
    where
        I: IntoIterator<Item = (&'static str, Option<&'static str>)>,
    {
        self.files.extend(files.into_iter().map(|(path, content)| {
            (
                path.to_string(),
                content.unwrap_or("default content").to_string(),
            )
        }));
        self
    }

    /// Add git configuration
    #[allow(dead_code)]
    pub fn with_git_config<K: AsRef<str>, V: AsRef<str>>(mut self, key: K, value: V) -> Self {
        self.git_config
            .push((key.as_ref().to_string(), value.as_ref().to_string()));
        self
    }

    /// Add multiple git config entries at once
    pub fn with_git_configs<I, K, V>(mut self, configs: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: AsRef<str>,
    {
        for (key, value) in configs {
            self.git_config
                .push((key.as_ref().to_string(), value.as_ref().to_string()));
        }
        self
    }

    /// Build the repository
    pub fn build(self) -> Result<TestRepo> {
        let temp_dir = TempDir::new()?;
        self.build_at_path(temp_dir.path()).map(|_| TestRepo {
            path: temp_dir.path().to_path_buf(),
            temp_dir,
        })
    }

    /// Build repository at specific path
    pub fn build_at_path(self, path: &Path) -> Result<TestRepo> {
        // Create directory if it doesn't exist
        std::fs::create_dir_all(path)?;

        // Initialize git repository
        Command::new("git")
            .args(["init"])
            .current_dir(path)
            .output()?;

        // Set basic git config
        Command::new("git")
            .args(["config", "user.name", "Test User"])
            .current_dir(path)
            .output()?;

        Command::new("git")
            .args(["config", "user.email", "test@example.com"])
            .current_dir(path)
            .output()?;

        // Create .gitignore
        if !self.patterns.is_empty() {
            let gitignore_content = self.patterns.join("\n");
            std::fs::write(path.join(".gitignore"), gitignore_content)?;
        }

        // Create files
        for (file_path, content) in &self.files {
            let full_path = path.join(file_path);
            if let Some(parent) = full_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::write(full_path, content)?;
        }

        // Apply git config
        for (key, value) in &self.git_config {
            Command::new("git")
                .args(["config", key, value])
                .current_dir(path)
                .output()?;
        }

        let temp_dir = TempDir::new()?; // Placeholder
        Ok(TestRepo {
            path: path.to_path_buf(),
            temp_dir,
        })
    }

    /// Build and return TempDir for legacy compatibility
    #[allow(dead_code)]
    pub fn build_temp_dir(self) -> Result<TempDir> {
        let temp_dir = TempDir::new()?;
        self.build_at_path(temp_dir.path())?;
        Ok(temp_dir)
    }
}

impl Default for TestRepoBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Test command
#[derive(Debug, Clone)]
pub struct TestCommand {
    #[allow(dead_code)]
    binary: String,
    args: Vec<String>,
    #[allow(dead_code)]
    working_dir: Option<PathBuf>,
    #[allow(dead_code)]
    stdin_input: Option<String>,
}

impl TestCommand {
    /// Create a new command
    pub fn new() -> Self {
        Self {
            binary: "diff-gitignore-filter".to_string(),
            args: Vec::new(),
            working_dir: None,
            stdin_input: None,
        }
    }

    /// Set working directory
    #[allow(dead_code)]
    pub fn in_dir<P: AsRef<Path>>(mut self, dir: P) -> Self {
        self.working_dir = Some(dir.as_ref().to_path_buf());
        self
    }

    /// Add argument
    pub fn arg<S: AsRef<str>>(mut self, arg: S) -> Self {
        self.args.push(arg.as_ref().to_string());
        self
    }

    /// Set stdin input
    #[allow(dead_code)]
    pub fn input<S: AsRef<str>>(mut self, input: S) -> Self {
        self.stdin_input = Some(input.as_ref().to_string());
        self
    }

    /// Get the number of arguments (for testing)
    #[allow(dead_code)]
    pub(crate) fn args_len(&self) -> usize {
        self.args.len()
    }

    /// Enable VCS filtering
    #[allow(dead_code)]
    pub fn vcs_filter(self) -> Self {
        self.arg("--vcs")
    }

    /// Set downstream command
    #[allow(dead_code)]
    pub fn downstream<S: AsRef<str>>(self, command: S) -> Self {
        self.arg("--downstream").arg(command.as_ref())
    }

    /// Execute the command
    #[allow(dead_code)]
    pub fn execute(self) -> Result<CommandOutput> {
        let mut cmd = AssertCommand::cargo_bin(&self.binary)?;

        if let Some(dir) = &self.working_dir {
            cmd.current_dir(dir);
        }

        cmd.args(&self.args);

        let output = if let Some(input) = &self.stdin_input {
            cmd.write_stdin(input.as_str()).output()?
        } else {
            cmd.output()?
        };

        Ok(CommandOutput::from_output(output))
    }
}

impl Default for TestCommand {
    fn default() -> Self {
        Self::new()
    }
}

/// Framework utility functions
#[allow(dead_code)]
pub struct TestFramework;

impl TestFramework {
    /// Create a command with automatic binary building
    #[allow(dead_code)]
    pub fn command() -> assert_cmd::Command {
        ensure_binary_built();
        assert_cmd::Command::cargo_bin("diff-gitignore-filter").expect("Failed to find binary")
    }
}

/// Command output
#[derive(Debug)]
pub struct CommandOutput {
    output: Output,
    stdout: String,
    #[allow(dead_code)]
    stderr: String,
}

impl CommandOutput {
    /// Create CommandOutput from std::process::Output (for testing)
    pub(crate) fn from_output(output: Output) -> Self {
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        Self {
            output,
            stdout,
            stderr,
        }
    }

    /// Check if command was successful
    pub fn is_success(&self) -> bool {
        self.output.status.success()
    }

    /// Get stdout
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Get stderr
    #[allow(dead_code)]
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    /// Get exit code
    #[allow(dead_code)]
    pub fn exit_code(&self) -> Option<i32> {
        self.output.status.code()
    }
}

/// Test expectation
#[derive(Debug, Clone)]
pub enum Expectation {
    Success,
    #[allow(dead_code)]
    Failure,
    Contains(String),
    #[allow(dead_code)]
    Excludes(String),
    #[allow(dead_code)]
    ExitCode(i32),
}

impl Expectation {
    /// Validate expectation against output
    pub fn validate(&self, output: &CommandOutput) -> Result<()> {
        match self {
            Expectation::Success => {
                if output.is_success() {
                    Ok(())
                } else {
                    Err("Expected success but command failed".to_string().into())
                }
            }
            Expectation::Failure => {
                if !output.is_success() {
                    Ok(())
                } else {
                    Err("Expected failure but command succeeded".to_string().into())
                }
            }
            Expectation::Contains(text) => {
                if output.stdout().contains(text) {
                    Ok(())
                } else {
                    Err(format!("Expected output to contain '{text}'").into())
                }
            }
            Expectation::Excludes(text) => {
                if !output.stdout().contains(text) {
                    Ok(())
                } else {
                    Err(format!("Expected output to exclude '{text}'").into())
                }
            }
            Expectation::ExitCode(code) => {
                if output.exit_code() == Some(*code) {
                    Ok(())
                } else {
                    Err(format!(
                        "Expected exit code {} but got {:?}",
                        code,
                        output.exit_code()
                    )
                    .into())
                }
            }
        }
    }
}

/// Test result
#[derive(Debug)]
pub struct TestResult {
    #[allow(dead_code)]
    pub name: String,
    #[allow(dead_code)]
    pub success: bool,
    #[allow(dead_code)]
    pub output: CommandOutput,
}

impl TestResult {
    /// Check if test was successful
    #[allow(dead_code)]
    pub fn is_success(&self) -> bool {
        self.success
    }

    /// Get test name
    #[allow(dead_code)]
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get command output
    #[allow(dead_code)]
    pub fn output(&self) -> &CommandOutput {
        &self.output
    }
}

/// Common test data
pub struct TestData;

impl TestData {
    /// Sample diff for testing
    #[allow(dead_code)]
    pub const SAMPLE_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
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
"#;

    /// VCS sample diff
    #[allow(dead_code)]
    pub const VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
diff --git a/.git/config b/.git/config
index 2222222..3333333 100644
--- a/.git/config
+++ b/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
"#;

    /// Basic gitignore patterns
    #[allow(dead_code)]
    pub const BASIC_PATTERNS: &'static [&'static str] =
        &["*.log", "*.tmp", "target/", "node_modules/", ".env"];

    /// Simple gitignore patterns
    #[allow(dead_code)]
    pub const SIMPLE_PATTERNS: &'static [&'static str] = &["*.log", "*.tmp", "target/"];

    /// Log-only patterns
    #[allow(dead_code)]
    pub const LOG_ONLY_PATTERNS: &'static [&'static str] = &["*.log"];

    /// Basic test files
    #[allow(dead_code)]
    pub const BASIC_FILES: &'static [(&'static str, Option<&'static str>)] = &[
        ("main.rs", Some("fn main() {}")),
        ("debug.log", Some("log content")),
        ("target/debug/main", Some("binary")),
    ];

    /// VCS sample diff
    #[allow(dead_code)]
    pub const VCS_SAMPLE_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
diff --git a/.git/config b/.git/config
index 2222222..3333333 100644
--- a/.git/config
+++ b/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
"#;

    /// Complex VCS diff
    #[allow(dead_code)]
    pub const COMPLEX_VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
diff --git a/README.md b/README.md
index 5555555..6666666 100644
--- a/README.md
+++ b/README.md
@@ -1,2 +1,3 @@
 # Project
 This is a test project.
+Updated documentation.
diff --git a/my.git.txt b/my.git.txt
index 7777777..8888888 100644
--- a/my.git.txt
+++ b/my.git.txt
@@ -1 +1,2 @@
 Some content
+More content
diff --git a/src/vcs/git_parser.rs b/src/vcs/git_parser.rs
index 9999999..aaaaaaa 100644
--- a/src/vcs/git_parser.rs
+++ b/src/vcs/git_parser.rs
@@ -1,2 +1,3 @@
 // Git parser implementation
 pub fn parse_git() {}
+// Updated parser
diff --git a/docs/.hg/store/data/readme.txt.i b/docs/.hg/store/data/readme.txt.i
index bbbbbbb..ccccccc 100644
--- a/docs/.hg/store/data/readme.txt.i
+++ b/docs/.hg/store/data/readme.txt.i
@@ -1 +1,2 @@
 mercurial data
+updated data
diff --git a/.git/config b/.git/config
index 2222222..3333333 100644
--- a/.git/config
+++ b/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
diff --git a/.svn/entries b/.svn/entries
index 3333333..4444444 100644
--- a/.svn/entries
+++ b/.svn/entries
@@ -1 +1,2 @@
 12
+13
diff --git a/deep/nested/path/.svn/entries b/deep/nested/path/.svn/entries
index ddddddd..eeeeeee 100644
--- a/deep/nested/path/.svn/entries
+++ b/deep/nested/path/.svn/entries
@@ -1 +1,2 @@
 nested svn
+updated nested
diff --git a/.hg/hgrc b/.hg/hgrc
index fffffff..1111111 100644
--- a/.hg/hgrc
+++ b/.hg/hgrc
@@ -1,2 +1,3 @@
 [ui]
 username = Test User
+editor = vim
diff --git a/CVS/Entries b/CVS/Entries
index 2222222..3333333 100644
--- a/CVS/Entries
+++ b/CVS/Entries
@@ -1 +1,2 @@
 /main.c/1.1/Mon Jan 01 00:00:00 2024//
+/test.c/1.1/Mon Jan 01 00:00:00 2024//
diff --git a/.bzr/branch-format b/.bzr/branch-format
index 4444444..5555555 100644
--- a/.bzr/branch-format
+++ b/.bzr/branch-format
@@ -1 +1,2 @@
 Bazaar-NG meta directory, format 1
+Updated format
"#;

    /// Multi repo diff
    #[allow(dead_code)]
    pub const MULTI_REPO_DIFF: &'static str = r#"diff --git a/main.rs b/main.rs
index 1234567..abcdefg 100644
--- a/main.rs
+++ b/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    /// Downstream processing diff
    #[allow(dead_code)]
    pub const DOWNSTREAM_PROCESSING_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    /// Nested VCS diff
    #[allow(dead_code)]
    pub const NESTED_VCS_DIFF: &'static str = r#"diff --git a/src/vcs/git_parser.rs b/src/vcs/git_parser.rs
index 1111111..2222222 100644
--- a/src/vcs/git_parser.rs
+++ b/src/vcs/git_parser.rs
@@ -1,2 +1,3 @@
 // Git parser implementation
 pub fn parse_git() {}
+// Updated parser
diff --git a/project/.git/config b/project/.git/config
index 2222222..3333333 100644
--- a/project/.git/config
+++ b/project/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
diff --git a/deep/nested/path/.svn/entries b/deep/nested/path/.svn/entries
index 3333333..4444444 100644
--- a/deep/nested/path/.svn/entries
+++ b/deep/nested/path/.svn/entries
@@ -1 +1,2 @@
 12
+13
"#;

    /// VCS-like filenames diff
    #[allow(dead_code)]
    pub const VCS_LIKE_FILENAMES_DIFF: &'static str = r#"diff --git a/git_helper.py b/git_helper.py
index 1234567..abcdefg 100644
--- a/git_helper.py
+++ b/git_helper.py
@@ -1,3 +1,4 @@
 def git_helper():
+def help_with_git():
     pass
diff --git a/svn_backup.txt b/svn_backup.txt
index 2222222..3333333 100644
--- a/svn_backup.txt
+++ b/svn_backup.txt
@@ -1,2 +1,3 @@
 SVN backup file
 Contains backup data
+More backup data
diff --git a/my.git.config b/my.git.config
index 4444444..5555555 100644
--- a/my.git.config
+++ b/my.git.config
@@ -1,2 +1,3 @@
 # My git config file
 user.name = Test User
+Additional config
diff --git a/tools/hg_converter.sh b/tools/hg_converter.sh
index 6666666..7777777 100644
--- a/tools/hg_converter.sh
+++ b/tools/hg_converter.sh
@@ -1,3 +1,4 @@
 #!/bin/bash
 # Mercurial converter script
 echo "Converting repository"
+Converting from HG
"#;

    /// Extended complex VCS diff
    #[allow(dead_code)]
    pub const EXTENDED_COMPLEX_VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    /// Unicode filename diff
    #[allow(dead_code)]
    pub const UNICODE_FILENAME_DIFF: &'static str = r#"diff --git a/src/测试文件.rs b/src/测试文件.rs
index 1234567..abcdefg 100644
--- a/src/测试文件.rs
+++ b/src/测试文件.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    /// VCS pattern sample diff
    #[allow(dead_code)]
    pub const VCS_PATTERN_SAMPLE_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
"#;

    /// Custom VCS diff
    #[allow(dead_code)]
    pub const CUSTOM_VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!("Hello, world!");
+    println!("Updated!");
 }
diff --git a/.custom/config b/.custom/config
index 2222222..3333333 100644
--- a/.custom/config
+++ b/.custom/config
@@ -1,2 +1,3 @@
 [custom]
     setting = value
+    new_setting = new_value
"#;

    /// Mixed VCS diff
    #[allow(dead_code)]
    pub const MIXED_VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!("Hello, world!");
+    println!("Updated!");
 }
diff --git a/.git/config b/.git/config
index 2222222..3333333 100644
--- a/.git/config
+++ b/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
diff --git a/.custom/config b/.custom/config
index 2222222..3333333 100644
--- a/.custom/config
+++ b/.custom/config
@@ -1,2 +1,3 @@
 [custom]
     setting = value
+    new_setting = new_value
"#;

    /// Multi custom VCS diff
    #[allow(dead_code)]
    pub const MULTI_CUSTOM_VCS_DIFF: &'static str = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!("Hello, world!");
+    println!("Updated!");
 }
diff --git a/.git/config b/.git/config
index 2222222..3333333 100644
--- a/.git/config
+++ b/.git/config
@@ -1,2 +1,3 @@
 [core]
     bare = false
+    autocrlf = true
diff --git a/.custom1/config b/.custom1/config
index 2222222..3333333 100644
--- a/.custom1/config
+++ b/.custom1/config
@@ -1,2 +1,3 @@
 [custom1]
     setting = value
+    new_setting = new_value
diff --git a/.custom2/config b/.custom2/config
index 2222222..3333333 100644
--- a/.custom2/config
+++ b/.custom2/config
@@ -1,2 +1,3 @@
 [custom2]
     setting = value
+    new_setting = new_value
"#;
}

// Framework tests have been moved to tests/framework_tests.rs to avoid duplication
// across multiple test files that import the common module
