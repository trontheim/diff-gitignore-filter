//! Git Worktree Support Tests
//!
//! Tests for the Git worktree functionality in the RootFinder module.
//! Uses the unified test framework from tests/common.

use diff_gitignore_filter::root_finder::RootFinder;
use std::io::Cursor;
use std::process::Command;
use tempfile::TempDir;

mod common;
use common::*;

/// Test worktree creation and root detection
/// **What is tested:** Git worktree root detection functionality in RootFinder
/// **Why it is tested:** Ensures RootFinder correctly identifies worktree roots instead of main repository roots
/// **Test conditions:** Main repository with worktree, diff content referencing worktree files
/// **Expectations:** Should return worktree path as root, not main repository path
#[test]
fn test_worktree_root_detection() -> Result<()> {
    // Create main repository
    let main_repo = TestRepo::builder()
        .with_patterns(["*.log", "target/"])
        .with_static_files([
            ("src/main.rs", Some("fn main() {}")),
            ("README.md", Some("# Test Project")),
        ])
        .build()?;

    // Create worktree directory
    let worktree_temp = TempDir::new()?;
    let worktree_path = worktree_temp.path().join("feature-branch");

    // Create worktree using git command
    Command::new("git")
        .current_dir(main_repo.path())
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .output()
        .expect("Failed to create worktree");

    // Add files to worktree
    std::fs::write(worktree_path.join("feature.rs"), "// Feature code")?;
    std::fs::write(worktree_path.join(".gitignore"), "*.tmp\n")?;

    // Create diff content that references worktree files
    let diff_content = "diff --git a/feature.rs b/feature.rs\n\
         new file mode 100644\n\
         index 0000000..1234567\n\
         --- /dev/null\n\
         +++ b/feature.rs\n\
         @@ -0,0 +1,1 @@\n\
         +// Feature code\n"
        .to_string();

    // Test root finding from worktree directory
    let diff_reader = Cursor::new(diff_content);
    let found_root = RootFinder::find_root(worktree_path.clone(), diff_reader)?;

    // Should return the worktree path, not the main repository path
    assert_eq!(found_root, worktree_path);
    assert_ne!(found_root, main_repo.path());

    Ok(())
}

/// Test worktree priority scoring
/// **What is tested:** Priority scoring mechanism for worktree vs main repository selection
/// **Why it is tested:** Ensures RootFinder prioritizes worktree when executed from worktree directory
/// **Test conditions:** Main repository with worktree, shared files, execution from worktree directory
/// **Expectations:** Should prioritize worktree root when running from worktree context
#[test]
fn test_worktree_priority_scoring() -> Result<()> {
    // Create main repository
    let main_repo = TestRepo::builder()
        .with_patterns(["*.log"])
        .with_static_files([("main.rs", Some("fn main() {}"))])
        .build()?;

    // Create worktree
    let worktree_temp = TempDir::new()?;
    let worktree_path = worktree_temp.path().join("worktree");

    Command::new("git")
        .current_dir(main_repo.path())
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test",
        ])
        .output()
        .expect("Failed to create worktree");

    // Add .gitignore to worktree
    std::fs::write(worktree_path.join(".gitignore"), "*.tmp\n")?;

    // Create the shared file in worktree
    std::fs::write(worktree_path.join("shared.rs"), "// Shared code")?;

    // Test with diff that references the worktree file
    let diff_content = "diff --git a/shared.rs b/shared.rs\n\
                       new file mode 100644\n\
                       index 0000000..1234567\n\
                       --- /dev/null\n\
                       +++ b/shared.rs\n\
                       @@ -0,0 +1,1 @@\n\
                       +// Shared code\n";

    // Test from the worktree directory
    let diff_reader = Cursor::new(diff_content);
    let found_root = RootFinder::find_root(worktree_path.clone(), diff_reader)?;

    // Should find the worktree root since we're running from there
    assert_eq!(found_root, worktree_path);

    Ok(())
}

/// Test worktree without .gitignore
/// **What is tested:** Worktree handling when no .gitignore file is present
/// **Why it is tested:** Ensures worktree detection works even without gitignore files
/// **Test conditions:** Worktree without .gitignore file, diff content referencing worktree files
/// **Expectations:** Should correctly identify worktree root despite missing .gitignore
#[test]
fn test_worktree_without_gitignore() -> Result<()> {
    // Create main repository without .gitignore
    let main_repo = TestRepo::builder()
        .with_static_files([("main.rs", Some("fn main() {}"))])
        .build()?;

    // Create worktree without .gitignore
    let worktree_temp = TempDir::new()?;
    let worktree_path = worktree_temp.path().join("worktree");

    Command::new("git")
        .current_dir(main_repo.path())
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test",
        ])
        .output()
        .expect("Failed to create worktree");

    // Add file to worktree
    std::fs::write(worktree_path.join("worktree_file.rs"), "// Worktree file")?;

    let diff_content = "diff --git a/worktree_file.rs b/worktree_file.rs\n\
                       new file mode 100644\n\
                       index 0000000..1234567\n\
                       --- /dev/null\n\
                       +++ b/worktree_file.rs\n\
                       @@ -0,0 +1,1 @@\n\
                       +// Worktree file\n";

    let diff_reader = Cursor::new(diff_content);
    let found_root = RootFinder::find_root(worktree_path.clone(), diff_reader)?;

    // Should find the worktree root
    assert_eq!(found_root, worktree_path);

    Ok(())
}

/// Test bare repository handling
/// **What is tested:** Root detection for bare Git repositories
/// **Why it is tested:** Ensures RootFinder handles bare repositories correctly (no working directory)
/// **Test conditions:** Bare Git repository, diff content with test files
/// **Expectations:** Should return the bare repository directory itself as root
#[test]
fn test_bare_repository_root() -> Result<()> {
    let temp_dir = TempDir::new()?;
    let bare_repo_path = temp_dir.path().join("bare.git");

    // Create bare repository
    Command::new("git")
        .args(["init", "--bare", bare_repo_path.to_str().unwrap()])
        .output()
        .expect("Failed to create bare repository");

    // Test with diff content
    let diff_content = "diff --git a/test.rs b/test.rs\n\
                       new file mode 100644\n\
                       index 0000000..1234567\n\
                       --- /dev/null\n\
                       +++ b/test.rs\n\
                       @@ -0,0 +1,1 @@\n\
                       +// Test file\n";

    let diff_reader = Cursor::new(diff_content);
    let found_root = RootFinder::find_root(bare_repo_path.clone(), diff_reader)?;

    // For bare repositories, should return the git directory itself
    assert_eq!(found_root, bare_repo_path);

    Ok(())
}

/// Test worktree with external paths
/// **What is tested:** Worktree handling when diff contains external file paths
/// **Why it is tested:** Ensures proper fallback behavior when diff references files outside worktree
/// **Test conditions:** Worktree with diff containing both worktree and external file paths
/// **Expectations:** Should handle external paths gracefully, potentially delegating to outside repo workflow
#[test]
fn test_worktree_with_external_paths() -> Result<()> {
    // Create main repository
    let main_repo = TestRepo::builder()
        .with_patterns(["*.log"])
        .with_static_files([("main.rs", Some("fn main() {}"))])
        .build()?;

    // Create worktree
    let worktree_temp = TempDir::new()?;
    let worktree_path = worktree_temp.path().join("worktree");

    Command::new("git")
        .current_dir(main_repo.path())
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "test",
        ])
        .output()
        .expect("Failed to create worktree");

    // Create worktree file
    std::fs::write(worktree_path.join("worktree_file.rs"), "// Worktree file")?;

    // Create external file
    let external_temp = TempDir::new()?;
    let external_file = external_temp.path().join("external.rs");
    std::fs::write(&external_file, "// External file")?;

    // Create diff with both worktree and external paths
    let diff_content = format!(
        "diff --git a/worktree_file.rs b/worktree_file.rs\n\
         new file mode 100644\n\
         --- /dev/null\n\
         +++ b/worktree_file.rs\n\
         @@ -0,0 +1,1 @@\n\
         +// Worktree file\n\
         diff --git a/{} b/{}\n\
         new file mode 100644\n\
         --- /dev/null\n\
         +++ b/{}\n\
         @@ -0,0 +1,1 @@\n\
         +// External file\n",
        external_file.display(),
        external_file.display(),
        external_file.display()
    );

    let diff_reader = Cursor::new(diff_content);
    let found_root = RootFinder::find_root(worktree_path, diff_reader)?;

    // Should delegate to outside repo workflow due to external paths
    // The result should be a valid path, current directory, or relative path
    assert!(
        found_root.exists()
            || found_root == std::path::PathBuf::from(".")
            || found_root.is_relative()
    );

    Ok(())
}

/// Integration test using the unified test framework
/// **What is tested:** Integration of worktree functionality with the unified test framework
/// **Why it is tested:** Ensures worktree support works correctly with the complete filtering pipeline
/// **Test conditions:** Worktree with specific .gitignore, test files, diff processing through framework
/// **Expectations:** Should apply worktree-specific gitignore patterns and filter correctly
#[test]
fn test_worktree_integration_with_framework() -> Result<()> {
    // Create main repository
    let main_repo = TestRepo::builder()
        .with_patterns(["*.log", "target/"])
        .with_static_files([
            ("src/main.rs", Some("fn main() {}")),
            (
                "Cargo.toml",
                Some("[package]\nname = \"test\"\nversion = \"0.1.0\""),
            ),
        ])
        .build()?;

    // Create worktree
    let worktree_temp = TempDir::new()?;
    let worktree_path = worktree_temp.path().join("feature");

    Command::new("git")
        .current_dir(main_repo.path())
        .args([
            "worktree",
            "add",
            worktree_path.to_str().unwrap(),
            "-b",
            "feature",
        ])
        .output()
        .expect("Failed to create worktree");

    // Add worktree-specific .gitignore
    std::fs::write(worktree_path.join(".gitignore"), "*.tmp\n*.bak\n")?;

    // Create test files in worktree
    std::fs::write(
        worktree_path.join("feature.rs"),
        "// Feature implementation",
    )?;
    std::fs::write(worktree_path.join("test.tmp"), "temporary file")?;

    // Create diff content
    let diff_content = "diff --git a/feature.rs b/feature.rs\n\
                       new file mode 100644\n\
                       index 0000000..abc123\n\
                       --- /dev/null\n\
                       +++ b/feature.rs\n\
                       @@ -0,0 +1,1 @@\n\
                       +// Feature implementation\n\
                       diff --git a/test.tmp b/test.tmp\n\
                       new file mode 100644\n\
                       index 0000000..def456\n\
                       --- /dev/null\n\
                       +++ b/test.tmp\n\
                       @@ -0,0 +1,1 @@\n\
                       +temporary file\n";

    // Test using the framework
    let result = TestCase::new("worktree_integration")
        .with_command(
            TestCommand::new()
                .in_dir(&worktree_path)
                .input(diff_content),
        )
        .expect_success()
        .expect_contains("feature.rs")
        .expect_excludes("test.tmp") // Should be filtered by worktree .gitignore
        .run()?;

    assert!(result.success);

    Ok(())
}
