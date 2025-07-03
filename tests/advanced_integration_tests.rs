//! Advanced integration tests for diff-gitignore-filter
//!
//! This module contains comprehensive end-to-end integration tests and edge case tests
//! that verify complex scenarios and boundary conditions of the diff-gitignore-filter tool.

use predicates::prelude::*;

mod common;
use common::framework::{TestFramework, TestRepo};
use common::{test_utilities::AdvancedTestRepo, TestData};
// Modern framework imports

/// Helper to create malformed diff content
fn create_malformed_diffs() -> Vec<(&'static str, &'static str)> {
    vec![
        ("incomplete_header", "diff --git a/file.txt\nindex 123..456\n+added line\n"),
        ("missing_hunks", "diff --git a/file.txt b/file.txt\nindex 123..456 100644\n--- a/file.txt\n+++ b/file.txt\n"),
        ("invalid_line_numbers", "diff --git a/file.txt b/file.txt\nindex 123..456 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -abc,def +ghi,jkl @@\n+line\n"),
        ("truncated_diff", "diff --git a/file.txt b/file.txt\nindex 123..456 100644\n--- a/file.txt\n+++ b/file.txt\n@@ -1,3 +1,4 @@\n line1\n"),
        ("mixed_line_endings", "diff --git a/file.txt b/file.txt\r\nindex 123..456 100644\r\n--- a/file.txt\n+++ b/file.txt\r\n@@ -1 +1,2 @@\r\n line\r\n+new line\n"),
    ]
}

// =============================================================================
// INTEGRATION TESTS - Complex End-to-End Scenarios
// =============================================================================

/// **What is tested:** Multi-repository environment filtering with complex gitignore patterns
/// **Why it is tested:** Ensures the filter works correctly across different repository structures and handles multi-repo diffs properly
/// **Test conditions:** Multi-repository setup with various gitignore patterns, complex diff input with multiple files
/// **Expectations:** Should include allowed files (main.rs) and exclude ignored files (debug.log) based on gitignore patterns
#[test]
fn test_multi_repository_filtering() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let multi_repo_diff = TestData::MULTI_REPO_DIFF;

    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .write_stdin(multi_repo_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("main.rs"))
        .stdout(predicate::str::contains("println!(\"Hello, world!\");")) // Actual content from the diff
        .stdout(predicate::str::contains("debug.log").not()); // Filtered by .gitignore (*.log)
}

/// **What is tested:** Integration with downstream processing tools in complex pipeline scenarios
/// **Why it is tested:** Verifies that filtered output can be properly piped to downstream tools like grep for further processing
/// **Test conditions:** Multi-repo environment with downstream grep filter to remove deletion lines, complex diff input
/// **Expectations:** Should pass filtered content to downstream tool and exclude ignored files while preserving content structure
#[test]
fn test_downstream_filter_with_complex_pipeline() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let complex_diff = TestData::DOWNSTREAM_PROCESSING_DIFF;

    // Test with grep as downstream filter
    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .arg("--downstream")
        .arg("grep -v '^-'") // Remove deletion lines
        .write_stdin(complex_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("Hello"))
        .stdout(predicate::str::contains("world!")) // Correct case and punctuation
        .stdout(predicate::str::contains("debug.log").not());
}

/// **What is tested:** VCS pattern filtering with custom git configuration settings
/// **Why it is tested:** Ensures custom VCS ignore patterns can be configured via git config and work correctly
/// **Test conditions:** Custom git config for VCS patterns, diff containing VCS files (.git/, .svn/) and regular files
/// **Expectations:** Should filter out VCS files based on custom configuration while preserving regular source files
#[test]
fn test_vcs_patterns_with_custom_configuration() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    // Set up git config for custom VCS patterns using framework
    repo.set_git_config(
        "diff-gitignore-filter.vcs-ignore.patterns",
        ".git/,.svn/,.hg/,CVS/",
    )
    .expect("Failed to set git config");

    let vcs_diff = r#"diff --git a/.git/config b/.git/config
index 1234567..abcdefg 100644
--- a/.git/config
+++ b/.git/config
@@ -1 +1,2 @@
 [core]
+    editor = vim
diff --git a/.svn/entries b/.svn/entries
index 1111111..2222222 100644
--- a/.svn/entries
+++ b/.svn/entries
@@ -1 +1,2 @@
 12
+svn entry
diff --git a/src/main.rs b/src/main.rs
index 3333333..4444444 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1 +1,2 @@
 fn main() {}
+// Regular file change
"#;

    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .arg("--vcs")
        .write_stdin(vcs_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("Regular file change"))
        .stdout(predicate::str::contains(".git/config").not())
        .stdout(predicate::str::contains(".svn/entries").not());

    // Clean up git config using framework
    repo.unset_git_config("diff-gitignore-filter.vcs-ignore.patterns");
}

/// **What is tested:** Various CLI parameter combinations and their interactions
/// **Why it is tested:** Verifies that different CLI flags work correctly together and produce expected filtering behavior
/// **Test conditions:** Multiple test scenarios with different flag combinations (--vcs, --downstream, --no-vcs, --vcs-pattern)
/// **Expectations:** Each parameter combination should produce appropriate filtering results without conflicts
#[test]
fn test_cli_parameter_combinations() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let test_diff = r#"diff --git a/.git/hooks/pre-commit b/.git/hooks/pre-commit
index 1234567..abcdefg 100644
--- a/.git/hooks/pre-commit
+++ b/.git/hooks/pre-commit
@@ -1 +1,2 @@
 #!/bin/bash
+echo "pre-commit hook"
diff --git a/src/lib.rs b/src/lib.rs
index 1111111..2222222 100644
--- a/src/lib.rs
+++ b/src/lib.rs
@@ -1 +1,2 @@
 pub fn lib() {}
+// Library change
diff --git a/debug.log b/debug.log
index 3333333..4444444 100644
--- a/debug.log
+++ b/debug.log
@@ -1 +1,2 @@
 log content
+more log content
"#;

    // Test: --vcs filtering (without downstream to check filtered content)
    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .arg("--vcs")
        .write_stdin(test_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains(".git/hooks").not())
        .stdout(predicate::str::contains("debug.log").not());

    // Test: --vcs + --downstream combination (check that downstream receives filtered input)
    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .arg("--vcs")
        .arg("--downstream")
        .arg("wc -l")
        .write_stdin(test_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("7")); // Only src/lib.rs diff should be passed to wc -l

    // Test: --no-vcs + custom vcs-pattern
    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .arg("--no-vcs")
        .arg("--vcs-pattern")
        .arg(".custom/,.special/")
        .write_stdin(test_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("src/lib.rs"))
        .stdout(predicate::str::contains(".git/hooks")) // VCS filtering disabled
        .stdout(predicate::str::contains("debug.log").not()); // gitignore still active
}

/// **What is tested:** Filtering behavior with deeply nested and complex directory structures
/// **Why it is tested:** Ensures the filter handles complex directory hierarchies and special directory names correctly
/// **Test conditions:** Deep nested directories (level1/level2/.../level5), directories with spaces, various file types
/// **Expectations:** Should apply gitignore patterns correctly regardless of directory depth and handle special characters
#[test]
fn test_complex_directory_structure_filtering() {
    let temp_dir = AdvancedTestRepo::complex_directory_structure()
        .build()
        .unwrap()
        .into_temp_dir();

    let complex_structure_diff = r#"diff --git a/level1/level2/level3/level4/level5/deep_file.rs b/level1/level2/level3/level4/level5/deep_file.rs
index 1234567..abcdefg 100644
--- a/level1/level2/level3/level4/level5/deep_file.rs
+++ b/level1/level2/level3/level4/level5/deep_file.rs
@@ -1 +1,2 @@
 // Deep file
+// Modified deep file
diff --git a/level1/level2/level3/level4/level5/ignored.log b/level1/level2/level3/level4/level5/ignored.log
index 1111111..2222222 100644
--- a/level1/level2/level3/level4/level5/ignored.log
+++ b/level1/level2/level3/level4/level5/ignored.log
@@ -1 +1,2 @@
 deep log
+more deep log
diff --git a/dir with spaces/file.txt b/dir with spaces/file.txt
index 3333333..4444444 100644
--- a/dir with spaces/file.txt
+++ b/dir with spaces/file.txt
@@ -1 +1,2 @@
 content
+modified content
diff --git a/dir with spaces/temp.tmp b/dir with spaces/temp.tmp
index 5555555..6666666 100644
--- a/dir with spaces/temp.tmp
+++ b/dir with spaces/temp.tmp
@@ -1 +1,2 @@
 temp content
+more temp content
"#;

    let mut cmd = TestFramework::command();
    cmd.current_dir(temp_dir.path())
        .write_stdin(complex_structure_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("dir with spaces/file.txt"))
        .stdout(predicate::str::contains("modified content"))
        .stdout(predicate::str::contains("deep_file.rs").not()) // Ignored by pattern
        .stdout(predicate::str::contains("ignored.log").not()) // Ignored by pattern
        .stdout(predicate::str::contains("temp.tmp").not()); // Ignored by pattern
}

// =============================================================================
// EDGE CASE TESTS
// =============================================================================

/// **What is tested:** Graceful handling of malformed or incomplete diff input
/// **Why it is tested:** Ensures the tool doesn't crash when receiving invalid or corrupted diff data
/// **Test conditions:** Various types of malformed diffs (incomplete headers, missing hunks, invalid line numbers, etc.)
/// **Expectations:** Should handle malformed input gracefully without crashing, potentially with degraded functionality
#[test]
fn test_malformed_diff_handling() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let malformed_diffs = create_malformed_diffs();

    for (test_name, malformed_diff) in malformed_diffs {
        let mut cmd = TestFramework::command();
        let _result = cmd
            .current_dir(repo.path())
            .write_stdin(malformed_diff)
            .assert()
            .success(); // Should handle malformed input gracefully

        // Verify that malformed input doesn't crash the program
        // The exact output depends on how malformed the input is
        println!("Malformed diff test '{test_name}' completed successfully");
    }
}

/// **What is tested:** Handling of extremely long file paths near filesystem limits
/// **Why it is tested:** Ensures the tool can process files with very long paths without buffer overflows or crashes
/// **Test conditions:** Generated very long file paths with multiple nested directories, mixed with normal files
/// **Expectations:** Should process long paths correctly and apply filtering rules regardless of path length
#[test]
fn test_very_long_file_paths() {
    // Using new framework with LOG_ONLY_PATTERNS (*.log)
    let repo = TestRepo::builder()
        .with_patterns(vec!["*.log".to_string()])
        .build_temp_dir()
        .unwrap();

    // Create a very long path (close to filesystem limits)
    let long_component = "very_long_directory_name_that_exceeds_normal_expectations";
    let mut long_path = String::new();
    for i in 0..10 {
        long_path.push_str(&format!("{long_component}{i}/"));
    }
    long_path.push_str("final_file.rs");

    let long_path_diff = format!(
        r#"diff --git a/{long_path} b/{long_path}
index 1234567..abcdefg 100644
--- a/{long_path}
+++ b/{long_path}
@@ -1 +1,2 @@
 // Long path file
+// Modified long path file
diff --git a/short.log b/short.log
index 1111111..2222222 100644
--- a/short.log
+++ b/short.log
@@ -1 +1,2 @@
 log content
+more log content
"#
    );

    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .write_stdin(long_path_diff.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("final_file.rs"))
        .stdout(predicate::str::contains("Modified long path file"))
        .stdout(predicate::str::contains("short.log").not());
}

/// **What is tested:** Unicode support in filenames and file content
/// **Why it is tested:** Ensures proper handling of international characters, emojis, and various Unicode encodings
/// **Test conditions:** Files with Cyrillic, Chinese, emoji, and accented characters in names and content
/// **Expectations:** Should correctly process Unicode filenames and content while applying gitignore patterns
#[test]
fn test_unicode_filenames_and_content() {
    let temp_dir = AdvancedTestRepo::unicode_test_repo()
        .build()
        .unwrap()
        .into_temp_dir();

    let unicode_diff = r#"diff --git a/файл.txt b/файл.txt
index 1234567..abcdefg 100644
--- a/файл.txt
+++ b/файл.txt
@@ -1 +1,2 @@
 Content of файл.txt
+Дополнительный контент
diff --git a/文件.txt b/文件.txt
index 1111111..2222222 100644
--- a/文件.txt
+++ b/文件.txt
@@ -1 +1,2 @@
 Content of 文件.txt
+额外内容
diff --git a/🚀rocket.txt b/🚀rocket.txt
index 3333333..4444444 100644
--- a/🚀rocket.txt
+++ b/🚀rocket.txt
@@ -1 +1,2 @@
 Content of 🚀rocket.txt
+🚀 More rocket content
diff --git a/café_résumé.txt b/café_résumé.txt
index 5555555..6666666 100644
--- a/café_résumé.txt
+++ b/café_résumé.txt
@@ -1 +1,2 @@
 Content of café_résumé.txt
+Contenu supplémentaire
diff --git a/regular.log b/regular.log
index 7777777..8888888 100644
--- a/regular.log
+++ b/regular.log
@@ -1 +1,2 @@
 regular log
+more log content
"#;

    let mut cmd = TestFramework::command();
    cmd.current_dir(temp_dir.path())
        .write_stdin(unicode_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("文件.txt"))
        .stdout(predicate::str::contains("额外内容"))
        .stdout(predicate::str::contains("файл.txt").not()) // Ignored by gitignore
        .stdout(predicate::str::contains("🚀rocket.txt").not()) // Ignored by gitignore
        .stdout(predicate::str::contains("café_résumé.txt").not()) // Ignored by gitignore
        .stdout(predicate::str::contains("regular.log").not()); // Ignored by gitignore
}

/// **What is tested:** Handling of empty input and whitespace-only diff content
/// **Why it is tested:** Ensures robust behavior when receiving minimal or empty input data
/// **Test conditions:** Various empty and whitespace-only inputs (empty string, spaces, tabs, newlines)
/// **Expectations:** Should handle empty input gracefully without errors or unexpected output
#[test]
fn test_empty_and_whitespace_only_diffs() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let test_cases = vec![
        ("empty", ""),
        ("whitespace_only", "   \n\t\n   \n"),
        ("newlines_only", "\n\n\n\n\n"),
        ("mixed_whitespace", " \t \n \r\n \t\t \n"),
    ];

    for (test_name, input) in test_cases {
        let mut cmd = TestFramework::command();
        cmd.current_dir(repo.path())
            .write_stdin(input)
            .assert()
            .success()
            .stdout(predicate::str::is_empty().or(predicate::str::contains("\n")));

        println!("Empty/whitespace test '{test_name}' completed successfully");
    }
}

/// **What is tested:** Filtering behavior with unusual directory naming conventions
/// **Why it is tested:** Ensures the tool handles edge cases in directory naming (hidden, double-hidden, case variations)
/// **Test conditions:** Directories with unusual names (.hidden, ..double_hidden, ALLCAPS, mixedCASE)
/// **Expectations:** Should apply gitignore patterns correctly regardless of directory naming conventions
#[test]
fn test_unusual_directory_structures() {
    let temp_dir = AdvancedTestRepo::unusual_directory_structure()
        .build()
        .unwrap()
        .into_temp_dir();

    let unusual_structure_diff = r#"diff --git a/.hidden/file.txt b/.hidden/file.txt
index 1234567..abcdefg 100644
--- a/.hidden/file.txt
+++ b/.hidden/file.txt
@@ -1 +1,2 @@
 content
+modified content
diff --git a/..double_hidden/file.txt b/..double_hidden/file.txt
index 1111111..2222222 100644
--- a/..double_hidden/file.txt
+++ b/..double_hidden/file.txt
@@ -1 +1,2 @@
 content
+modified content
diff --git a/ALLCAPS/file.txt b/ALLCAPS/file.txt
index 3333333..4444444 100644
--- a/ALLCAPS/file.txt
+++ b/ALLCAPS/file.txt
@@ -1 +1,2 @@
 content
+modified content
diff --git a/mixedCASE/file.txt b/mixedCASE/file.txt
index 5555555..6666666 100644
--- a/mixedCASE/file.txt
+++ b/mixedCASE/file.txt
@@ -1 +1,2 @@
 content
+modified content
"#;

    let mut cmd = TestFramework::command();
    cmd.current_dir(temp_dir.path())
        .write_stdin(unusual_structure_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("ALLCAPS/file.txt"))
        .stdout(predicate::str::contains("mixedCASE/file.txt"))
        .stdout(predicate::str::contains(".hidden/file.txt").not())
        .stdout(predicate::str::contains("..double_hidden/file.txt").not());
}

/// **What is tested:** Performance characteristics when processing large diff files
/// **Why it is tested:** Ensures the tool maintains reasonable performance with large inputs and doesn't have scalability issues
/// **Test conditions:** Generated large diff with 100 regular files and 50 ignored files, timed execution
/// **Expectations:** Should complete processing within reasonable time limits (< 30 seconds) while maintaining accuracy
#[test]
fn test_large_diff_performance() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    // Generate a large diff with many files
    let mut large_diff = String::new();

    // Add many regular files
    for i in 0..100 {
        large_diff.push_str(&format!(
            r#"diff --git a/src/file{}.rs b/src/file{}.rs
index {}..{} 100644
--- a/src/file{}.rs
+++ b/src/file{}.rs
@@ -1 +1,2 @@
 pub fn function{}() {{}}
+// Modified function{}
"#,
            i,
            i,
            i * 1000,
            i * 1000 + 1,
            i,
            i,
            i,
            i
        ));
    }

    // Add many ignored files
    for i in 0..50 {
        large_diff.push_str(&format!(
            r#"diff --git a/debug{}.log b/debug{}.log
index {}..{} 100644
--- a/debug{}.log
+++ b/debug{}.log
@@ -1 +1,2 @@
 log entry {}
+more log entry {}
"#,
            i,
            i,
            i * 2000,
            i * 2000 + 1,
            i,
            i,
            i,
            i
        ));
    }

    let start_time = std::time::Instant::now();

    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .write_stdin(large_diff.as_str())
        .assert()
        .success()
        .stdout(predicate::str::contains("src/file0.rs"))
        .stdout(predicate::str::contains("src/file99.rs"))
        .stdout(predicate::str::contains("debug0.log").not())
        .stdout(predicate::str::contains("debug49.log").not());

    let elapsed = start_time.elapsed();
    println!("Large diff processing took: {elapsed:?}");

    // Performance assertion: should complete within reasonable time
    assert!(
        elapsed.as_secs() < 30,
        "Large diff processing took too long: {elapsed:?}"
    );
}

/// **What is tested:** Processing of diffs containing binary files with Git binary patches
/// **Why it is tested:** Ensures the tool correctly handles binary file diffs without corrupting or misprocessing them
/// **Test conditions:** Diff containing binary files (PNG, PDF) with Git binary patches and regular text files
/// **Expectations:** Should preserve binary file diffs and apply filtering rules to both binary and text files
#[test]
fn test_binary_file_handling() {
    let repo = AdvancedTestRepo::multi_repo_environment().build().unwrap();

    let binary_diff = r#"diff --git a/image.png b/image.png
index 1234567..abcdefg 100644
GIT binary patch
delta 123
zcmV-?0F3;kP<x>4U}WI*3=9kL0{{R30RRC20H6W+1ONa40RR91
delta 456
zcmV-?0F3;kP<x>4U}WI*3=9kL0{{R30RRC20H6W+1ONa40RR92

diff --git a/document.pdf b/document.pdf
index 7777777..8888888 100644
GIT binary patch
delta 789
zcmV-?0F3;kP<x>4U}WI*3=9kL0{{R30RRC20H6W+1ONa40RR93
delta 101112
zcmV-?0F3;kP<x>4U}WI*3=9kL0{{R30RRC20H6W+1ONa40RR94

diff --git a/src/main.rs b/src/main.rs
index 1111111..2222222 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1 +1,2 @@
 fn main() {}
+// Text file change
"#;

    let mut cmd = TestFramework::command();
    cmd.current_dir(repo.path())
        .write_stdin(binary_diff)
        .assert()
        .success()
        .stdout(predicate::str::contains("image.png"))
        .stdout(predicate::str::contains("document.pdf"))
        .stdout(predicate::str::contains("src/main.rs"))
        .stdout(predicate::str::contains("Text file change"))
        .stdout(predicate::str::contains("GIT binary patch"));
}

/// **What is tested:** Thread safety and concurrent processing capabilities
/// **Why it is tested:** Ensures the tool can handle multiple concurrent executions without race conditions or data corruption
/// **Test conditions:** Multiple threads running the same filtering operation simultaneously on the same repository
/// **Expectations:** All concurrent executions should complete successfully with consistent results
#[test]
fn test_concurrent_processing_safety() {
    let temp_dir = AdvancedTestRepo::multi_repo_environment()
        .build()
        .unwrap()
        .into_temp_dir();
    let main_repo = temp_dir.path().to_path_buf(); // Clone the path to avoid lifetime issues

    let test_diff = r#"diff --git a/src/concurrent.rs b/src/concurrent.rs
index 1234567..abcdefg 100644
--- a/src/concurrent.rs
+++ b/src/concurrent.rs
@@ -1 +1,2 @@
 pub fn concurrent() {}
+// Concurrent modification
diff --git a/debug.log b/debug.log
index 1111111..2222222 100644
--- a/debug.log
+++ b/debug.log
@@ -1 +1,2 @@
 log entry
+concurrent log entry
"#;

    // Run multiple instances concurrently to test thread safety
    let handles: Vec<std::thread::JoinHandle<()>> = (0..5)
        .map(|i| {
            let main_repo_clone = main_repo.clone();
            let test_diff_clone = test_diff.to_string();

            std::thread::spawn(move || {
                let mut cmd = TestFramework::command();
                cmd.current_dir(main_repo_clone)
                    .write_stdin(test_diff_clone.as_str())
                    .assert()
                    .success()
                    .stdout(predicate::str::contains("src/concurrent.rs"))
                    .stdout(predicate::str::contains("Concurrent modification"))
                    .stdout(predicate::str::contains("debug.log").not());

                println!("Concurrent test instance {i} completed");
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().expect("Thread panicked");
    }

    println!("All concurrent processing tests completed successfully");
}
