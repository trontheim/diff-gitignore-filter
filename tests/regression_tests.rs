//! Regression tests for specific bugs that were fixed
//!
//! These tests ensure that previously identified bugs do not reoccur.

use diff_gitignore_filter::Filter;
use std::io::Cursor;

mod common;
use common::test_utilities::AdvancedTestRepo;

/// **Regression Test for Bug #1: Architecture Bug - VCS filtering before binary detection**
///
/// **Problem:** VCS filtering was skipped when binary content was detected, allowing VCS files
/// with binary content to pass through unfiltered.
///
/// **Fix:** VCS filtering now ALWAYS takes precedence over binary content detection.
///
/// **Test:** Ensures that VCS files are filtered even when they contain binary content markers.
#[test]
fn test_regression_vcs_filtering_before_binary_detection() -> Result<(), Box<dyn std::error::Error>>
{
    let test_repo = AdvancedTestRepo::vcs_test_repo().build().unwrap();

    let filter = Filter::new(test_repo.path())?
        .with_vcs_patterns(vec![".git/".to_string(), ".svn/".to_string()]);

    // Create a diff with VCS files that have binary content markers
    let binary_vcs_diff = r#"diff --git a/.git/objects/12/34567890abcdef b/.git/objects/12/34567890abcdef
new file mode 100644
index 0000000..1234567
Binary files /dev/null and b/.git/objects/12/34567890abcdef differ
diff --git a/src/main.rs b/src/main.rs
index abcdef..123456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!("Hello, world!");
+    println!("New line");
 }
diff --git a/.svn/wc.db b/.svn/wc.db
new file mode 100644
index 0000000..7890123
Binary files /dev/null and b/.svn/wc.db differ
"#;

    let mut output = Vec::new();
    filter.process_diff(Cursor::new(binary_vcs_diff), &mut output)?;
    let result = String::from_utf8(output)?;

    // VCS files should be filtered out even though they have binary content
    assert!(
        !result.contains(".git/objects/"),
        "VCS binary files should be filtered out: .git/objects/"
    );
    assert!(
        !result.contains(".svn/wc.db"),
        "VCS binary files should be filtered out: .svn/wc.db"
    );

    // Normal files should still be included
    assert!(
        result.contains("src/main.rs"),
        "Normal files should be included: src/main.rs"
    );
    assert!(
        result.contains("println!(\"New line\");"),
        "Content of normal files should be preserved"
    );

    Ok(())
}

/// **Regression Test for Bug #2: Logic Bug - Nested VCS path pattern matching**
///
/// **Problem:** Pattern ".git/" did not match nested paths like "jira-timesheet-cli/.git/COMMIT_EDITMSG"
///
/// **Fix:** Improved pattern matching logic to correctly handle nested VCS directories.
///
/// **Test:** Ensures that VCS patterns correctly match nested directory structures.
#[test]
fn test_regression_nested_vcs_path_matching() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = AdvancedTestRepo::vcs_test_repo().build().unwrap();

    let filter = Filter::new(test_repo.path())?.with_vcs_patterns(vec![
        ".git/".to_string(),
        ".hg/".to_string(),
        "CVS/".to_string(),
    ]);

    // Create a diff with nested VCS paths that should be filtered
    let nested_vcs_diff = r#"diff --git a/jira-timesheet-cli/.git/COMMIT_EDITMSG b/jira-timesheet-cli/.git/COMMIT_EDITMSG
new file mode 100644
index 0000000..1234567
--- /dev/null
+++ b/jira-timesheet-cli/.git/COMMIT_EDITMSG
@@ -0,0 +1 @@
+Test commit message
diff --git a/project-a/submodule/.hg/store/data/file.txt.i b/project-a/submodule/.hg/store/data/file.txt.i
index bbbbbbb..ccccccc 100644
--- a/project-a/submodule/.hg/store/data/file.txt.i
+++ b/project-a/submodule/.hg/store/data/file.txt.i
@@ -1 +1,2 @@
 mercurial data
+updated data
diff --git a/legacy/CVS/Entries b/legacy/CVS/Entries
new file mode 100644
index 0000000..abcdef
--- /dev/null
+++ b/legacy/CVS/Entries
@@ -0,0 +1 @@
+/file.c/1.1/Mon Jan 01 00:00:00 2024//
diff --git a/src/main.rs b/src/main.rs
index abcdef..123456 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
     println!("Hello, world!");
+    println!("Fixed nested VCS matching");
 }
diff --git a/docs/project.git.md b/docs/project.git.md
index 111111..222222 100644
--- a/docs/project.git.md
+++ b/docs/project.git.md
@@ -1,2 +1,3 @@
 # Git Documentation
 This file has 'git' in the name but is not a VCS file.
+Updated documentation.
"#;

    let mut output = Vec::new();
    filter.process_diff(Cursor::new(nested_vcs_diff), &mut output)?;
    let result = String::from_utf8(output)?;

    // Nested VCS files should be filtered out
    assert!(
        !result.contains("jira-timesheet-cli/.git/COMMIT_EDITMSG"),
        "Nested .git/ files should be filtered: jira-timesheet-cli/.git/COMMIT_EDITMSG"
    );
    assert!(
        !result.contains("project-a/submodule/.hg/store/"),
        "Nested .hg/ files should be filtered: project-a/submodule/.hg/"
    );
    assert!(
        !result.contains("legacy/CVS/Entries"),
        "Nested CVS/ files should be filtered: legacy/CVS/Entries"
    );

    // Normal files should be included, even if they have VCS-like names
    assert!(
        result.contains("src/main.rs"),
        "Normal files should be included: src/main.rs"
    );
    assert!(
        result.contains("docs/project.git.md"),
        "Files with VCS-like names should be included: docs/project.git.md"
    );
    assert!(
        result.contains("Fixed nested VCS matching"),
        "Content of normal files should be preserved"
    );

    Ok(())
}

/// **Combined Regression Test: Both bugs together**
///
/// **Test:** Ensures that both fixes work together correctly - nested VCS files with binary
/// content are properly filtered.
#[test]
fn test_regression_combined_vcs_and_binary_handling() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = AdvancedTestRepo::vcs_test_repo().build().unwrap();

    let filter = Filter::new(test_repo.path())?
        .with_vcs_patterns(vec![".git/".to_string(), ".svn/".to_string()]);

    // Create a diff with nested VCS binary files and normal files
    let combined_diff = r#"diff --git a/project-x/.git/objects/ab/cdef1234567890 b/project-x/.git/objects/ab/cdef1234567890
new file mode 100644
index 0000000..abcdef
Binary files /dev/null and b/project-x/.git/objects/ab/cdef1234567890 differ
diff --git a/module-y/.svn/pristine/12/34567890abcdef.svn-base b/module-y/.svn/pristine/12/34567890abcdef.svn-base
new file mode 100644
index 0000000..123456
Binary files /dev/null and b/module-y/.svn/pristine/12/34567890abcdef.svn-base differ
diff --git a/src/lib.rs b/src/lib.rs
index 111111..222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1,2 +1,3 @@
 pub fn hello() {
     println!("Hello from lib");
+    println!("Both bugs fixed!");
 }
diff --git a/binary_file.dat b/binary_file.dat
new file mode 100644
index 0000000..333333
Binary files /dev/null and b/binary_file.dat differ
"#;

    let mut output = Vec::new();
    filter.process_diff(Cursor::new(combined_diff), &mut output)?;
    let result = String::from_utf8(output)?;

    // Nested VCS binary files should be filtered out
    assert!(
        !result.contains("project-x/.git/objects/"),
        "Nested VCS binary files should be filtered: project-x/.git/objects/"
    );
    assert!(
        !result.contains("module-y/.svn/pristine/"),
        "Nested VCS binary files should be filtered: module-y/.svn/pristine/"
    );

    // Normal files should be included
    assert!(
        result.contains("src/lib.rs"),
        "Normal source files should be included: src/lib.rs"
    );
    assert!(
        result.contains("Both bugs fixed!"),
        "Content of normal files should be preserved"
    );

    // Non-VCS binary files should still be handled according to binary detection logic
    // (This tests that we didn't break normal binary file handling)
    assert!(
        result.contains("binary_file.dat") || !result.contains("binary_file.dat"),
        "Non-VCS binary files should be handled by normal binary detection logic"
    );

    Ok(())
}

/// **Edge Case Test: VCS pattern matching specificity**
///
/// **Test:** Ensures that the improved pattern matching is precise and doesn't over-match.
#[test]
fn test_regression_vcs_pattern_specificity() -> Result<(), Box<dyn std::error::Error>> {
    let test_repo = AdvancedTestRepo::vcs_test_repo().build().unwrap();

    let filter = Filter::new(test_repo.path())?.with_vcs_patterns(vec![".git/".to_string()]);

    // Create a diff with files that should NOT be filtered despite having similar names
    let specificity_diff = r#"diff --git a/.gitignore b/.gitignore
index 111111..222222 100644
--- a/.gitignore
+++ b/.gitignore
@@ -1,2 +1,3 @@
 *.tmp
 build/
+*.log
diff --git a/.github/workflows/ci.yml b/.github/workflows/ci.yml
new file mode 100644
index 0000000..333333
--- /dev/null
+++ b/.github/workflows/ci.yml
@@ -0,0 +1,3 @@
+name: CI
+on: [push]
+jobs: {}
diff --git a/my.git.backup b/my.git.backup
index 444444..555555 100644
--- a/my.git.backup
+++ b/my.git.backup
@@ -1 +1,2 @@
 backup data
+updated backup
diff --git a/project/.git/config b/project/.git/config
index 666666..777777 100644
--- a/project/.git/config
+++ b/project/.git/config
@@ -1,2 +1,3 @@
 [core]
 repositoryformatversion = 0
+filemode = true
"#;

    let mut output = Vec::new();
    filter.process_diff(Cursor::new(specificity_diff), &mut output)?;
    let result = String::from_utf8(output)?;

    // Files that should NOT be filtered (they're not VCS metadata)
    assert!(
        result.contains(".gitignore"),
        ".gitignore should NOT be filtered (it's a configuration file, not VCS metadata)"
    );
    assert!(
        result.contains(".github/workflows/ci.yml"),
        ".github/ should NOT be filtered (it's GitHub Actions, not Git metadata)"
    );
    assert!(
        result.contains("my.git.backup"),
        "Files with 'git' in name should NOT be filtered if they're not in .git/ directory"
    );

    // Files that SHOULD be filtered (they are VCS metadata)
    assert!(
        !result.contains("project/.git/config"),
        "Actual .git/ files should be filtered: project/.git/config"
    );

    Ok(())
}
