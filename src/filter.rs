//! Diff filtering module
//!
//! This module provides the main filtering functionality that respects .gitignore patterns
//! and supports VCS pattern filtering with optional downstream processing.

use crate::error::{Error, Result};
use ignore::gitignore::{Gitignore, GitignoreBuilder};
use std::io::{BufRead, Write};
use std::path::Path;
use std::process::{Command, Stdio};

/// Main filter for processing Git diffs
pub struct Filter {
    /// Gitignore patterns for filtering
    gitignore: Option<Gitignore>,
    /// VCS patterns for filtering VCS-related files
    vcs_patterns: Vec<String>,
    /// Whether VCS filtering is enabled (true = filter out VCS files, false = include VCS files)
    vcs_filtering_enabled: bool,
    /// Optional downstream command for piping output
    downstream_command: Option<String>,
}

impl Filter {
    /// Create a new filter for the given root directory
    pub fn new<P: AsRef<Path>>(root: P) -> Result<Self> {
        // Log the root directory received from RootFinder

        // Build gitignore patterns
        let gitignore = Self::build_gitignore(root.as_ref())?;

        Ok(Filter {
            gitignore,
            vcs_patterns: Vec::new(),
            vcs_filtering_enabled: false, // Default: VCS filtering disabled (include VCS files)
            downstream_command: None,
        })
    }

    /// Add VCS patterns for filtering and enable VCS filtering
    pub fn with_vcs_patterns(mut self, patterns: Vec<String>) -> Self {
        self.vcs_patterns = patterns;
        self.vcs_filtering_enabled = true; // Enable VCS filtering when patterns are provided
        self
    }

    /// Add downstream command for piping output
    pub fn with_downstream(mut self, command: String) -> Self {
        self.downstream_command = Some(command);
        self
    }

    /// Build gitignore patterns from the repository
    fn build_gitignore(root: &Path) -> Result<Option<Gitignore>> {
        // Log the path where we search for .gitignore

        let mut builder = GitignoreBuilder::new(root);

        // Add .gitignore file if it exists
        let gitignore_path = root.join(".gitignore");
        if gitignore_path.exists() {
            builder.add(&gitignore_path);
        }

        match builder.build() {
            Ok(gitignore) => Ok(Some(gitignore)),
            Err(_e) => {
                // If gitignore building fails, continue without it
                Ok(None)
            }
        }
    }

    /// Process a diff stream and filter it according to patterns
    pub fn process_diff<R: BufRead, W: Write>(&self, reader: R, writer: W) -> Result<()> {
        // Process as UTF-8 text data directly without binary detection
        // The binary detection was causing issues with BufReader state
        if let Some(ref command) = self.downstream_command {
            self.process_with_downstream(reader, command)
        } else {
            self.process_direct(reader, writer)
        }
    }

    /// Process diff directly to the writer with streaming optimization
    fn process_direct<R: BufRead, W: Write>(&self, mut reader: R, mut writer: W) -> Result<()> {
        // Read all data as bytes first, then process as UTF-8
        let mut all_data = Vec::new();
        reader
            .read_to_end(&mut all_data)
            .map_err(|e| Error::processing_error(format!("Failed to read input data: {e}")))?;

        // Convert to UTF-8 string with lossy conversion for robustness
        let content = String::from_utf8_lossy(&all_data);

        // First pass: Check if any sections should be filtered out by VCS patterns
        let mut has_vcs_filtered_content = false;
        if self.vcs_filtering_enabled {
            for line in content.lines() {
                if line.starts_with("diff --git") {
                    if let Some(file_path) = self.extract_file_path(line) {
                        if self.is_vcs_file(&file_path) {
                            has_vcs_filtered_content = true;
                            break;
                        }
                    }
                }
            }
        }

        // Only check for binary content if we don't have VCS content to filter
        // This ensures VCS filtering takes precedence over binary content preservation
        if !has_vcs_filtered_content {
            let is_binary = self.is_git_diff_with_binary_content(&all_data);

            if is_binary {
                // For Git diffs with binary content, pass through unchanged
                match writer.write_all(&all_data) {
                    Ok(()) => {}
                    Err(e) => {
                        // Check if this is a broken pipe error - this should be handled gracefully
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            // For broken pipe, we should exit gracefully rather than treating it as an error
                            return Ok(());
                        }
                        return Err(Error::processing_error(format!(
                            "Failed to write binary data: {e}"
                        )));
                    }
                }
                return Ok(());
            }
        }

        // Functional state management using Option and closures
        let mut current_section_state: Option<(Vec<String>, bool)> = None;

        // Use a smaller buffer size for better memory efficiency
        const MAX_BUFFER_SIZE: usize = 1024; // Limit buffer to 1KB of lines

        // Helper closure for flushing buffer functionally
        let flush_buffer = |writer: &mut W, buffer: &[String]| -> Result<()> {
            for line in buffer {
                match writeln!(writer, "{line}") {
                    Ok(()) => {}
                    Err(e) => {
                        // Check for broken pipe in line writing as well
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            return Ok(()); // Convert broken pipe to success
                        }
                        return Err(Error::processing_error(format!(
                            "Failed to write line: {e}"
                        )));
                    }
                }
            }
            Ok(())
        };

        // Process lines using functional iterator with try_for_each
        content.lines().try_for_each(|line| -> Result<()> {
            if line.starts_with("diff --git") {
                // Flush previous section if it should be included
                if let Some((ref buffer, should_include)) = current_section_state {
                    if should_include {
                        flush_buffer(&mut writer, buffer)?;
                    }
                }

                // Start new section using functional approach
                let file_path = self.extract_file_path(line);
                let should_include = file_path
                    .as_ref()
                    .map(|path| self.should_include_file(path))
                    .unwrap_or(false);

                current_section_state = Some((vec![line.to_string()], should_include));
            } else if let Some((ref mut buffer, should_include)) = current_section_state {
                buffer.push(line.to_string());

                // Flush buffer periodically using functional approach
                if should_include && buffer.len() >= MAX_BUFFER_SIZE {
                    flush_buffer(&mut writer, buffer)?;
                    buffer.clear();
                }
            } else {
                // Header lines before any diff - write immediately
                match writeln!(writer, "{line}") {
                    Ok(()) => {}
                    Err(e) => {
                        // Check for broken pipe in header writing
                        if e.kind() == std::io::ErrorKind::BrokenPipe {
                            return Ok(()); // Convert broken pipe to success
                        }
                        return Err(Error::processing_error(format!(
                            "Failed to write header line: {e}"
                        )));
                    }
                }
            }
            Ok(())
        })?;

        // Process remaining buffer for last section using functional approach
        current_section_state
            .filter(|(_, should_include)| *should_include)
            .map(|(buffer, _)| flush_buffer(&mut writer, &buffer))
            .transpose()?;

        Ok(())
    }

    /// Process diff with downstream command
    fn process_with_downstream<R: BufRead>(&self, reader: R, command: &str) -> Result<()> {
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(command)
            .stdin(Stdio::piped())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|e| {
                Error::DownstreamSpawnFailed(format!(
                    "Failed to spawn downstream command '{command}': {e}"
                ))
            })?;

        let stdin = child.stdin.take().ok_or_else(|| {
            Error::processing_error("Failed to get stdin of downstream command".to_string())
        })?;

        // Process diff to downstream command's stdin
        let process_result = self.process_direct(reader, stdin);

        let exit_status = child.wait().map_err(|e| {
            Error::processing_error(format!("Failed to wait for downstream command: {e}"))
        })?;

        if !exit_status.success() {
            return Err(Error::DownstreamProcessFailed(format!(
                "Downstream command '{}' failed with exit code: {:?}",
                command,
                exit_status.code()
            )));
        }

        // Return the result from process_direct - if it was a broken pipe, we should handle it gracefully
        process_result
    }

    /// Extract file path from diff header line
    fn extract_file_path(&self, line: &str) -> Option<String> {
        // Parse "diff --git a/path b/path" format using functional combinators
        let result = line.strip_prefix("diff --git ").and_then(|remaining| {
            // Find positions of "a/" and " b/" using functional approach
            let a_pos = remaining.find("a/")?;
            let b_pos = remaining.find(" b/")?;

            // Ensure "a/" comes before " b/"
            if a_pos + 2 < b_pos {
                Some(remaining[a_pos + 2..b_pos].to_string())
            } else {
                None
            }
        });

        result
    }

    /// Check if a file should be included based on gitignore and VCS patterns
    fn should_include_file(&self, file_path: &str) -> bool {
        // Check VCS patterns first - only if VCS filtering is enabled
        if self.vcs_filtering_enabled && self.is_vcs_file(file_path) {
            return false; // Exclude VCS files when VCS filtering is enabled
        }

        // Check gitignore patterns using functional combinators
        self.gitignore.as_ref().is_none_or(|gitignore| {
            let path = Path::new(file_path);

            // First try as a file
            match gitignore.matched(path, false) {
                ignore::Match::Ignore(_) => false,
                ignore::Match::Whitelist(_) => true,
                ignore::Match::None => {
                    // Check parent directories using find_map for cleaner iteration
                    path.ancestors()
                        .skip(1) // Skip the file itself
                        .take_while(|parent| *parent != Path::new(""))
                        .find_map(|parent| match gitignore.matched(parent, true) {
                            ignore::Match::Ignore(_) => Some(false),
                            ignore::Match::Whitelist(_) => Some(true),
                            ignore::Match::None => None,
                        })
                        .unwrap_or(true)
                }
            }
        })
    }

    /// Check if a file matches VCS patterns
    fn is_vcs_file(&self, file_path: &str) -> bool {
        self.vcs_patterns
            .iter()
            .any(|pattern| self.matches_vcs_pattern(file_path, pattern))
    }

    /// Helper method to check if file path matches a specific VCS pattern
    /// Optimized to reduce string allocations by using strip_suffix and direct comparisons
    fn matches_vcs_pattern(&self, file_path: &str, pattern: &str) -> bool {
        match pattern.strip_suffix("/*") {
            Some(dir_pattern) => {
                // For directory patterns (ending with /*), check directory boundaries
                file_path.starts_with(&format!("{dir_pattern}/"))
                    || file_path.contains(&format!("/{dir_pattern}/"))
            }
            None => {
                // For exact patterns, improved logic to handle nested paths correctly
                if file_path == pattern {
                    return true;
                }

                // Check if pattern ends with "/" (directory pattern without "/*")
                if pattern.ends_with('/') {
                    // For directory patterns like ".git/", check if file path contains this directory
                    // This handles cases like "jira-timesheet-cli/.git/COMMIT_EDITMSG"
                    file_path.contains(pattern) || file_path.starts_with(pattern)
                } else {
                    // For non-directory patterns, check prefix match or nested match
                    file_path.starts_with(&format!("{pattern}/"))
                        || file_path.contains(&format!("/{pattern}/"))
                        || file_path.contains(&format!("/{pattern}"))
                        || (file_path.starts_with(pattern)
                            && (file_path.len() == pattern.len()
                                || file_path.chars().nth(pattern.len()) == Some('/')))
                }
            }
        }
    }

    /// Check if the data contains binary content that should be preserved unchanged
    ///
    /// This method handles:
    /// 1. Pure binary data (no diff headers) - pass through unchanged
    /// 2. Git diffs with binary content - pass through unchanged
    /// 3. Regular text content - process normally
    fn is_git_diff_with_binary_content(&self, data: &[u8]) -> bool {
        // If data is empty, treat as text
        if data.is_empty() {
            return false;
        }

        // Check for any non-UTF-8 content that should be preserved
        // This includes null bytes and other binary sequences
        let has_binary_content = data.iter().any(|&b| {
            // Check for null bytes or high-bit bytes that might be binary
            b == 0 || (b >= 0x80 && !self.is_valid_utf8_sequence(data, b))
        });

        // If it contains binary content, preserve it unchanged
        has_binary_content
    }

    /// Helper to check if a byte sequence is valid UTF-8
    fn is_valid_utf8_sequence(&self, data: &[u8], _byte: u8) -> bool {
        // For simplicity, if String::from_utf8 would succeed, it's valid UTF-8
        // Otherwise, we should preserve the binary content
        std::str::from_utf8(data).is_ok()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use tempfile::TempDir;

    fn create_test_repo() -> Result<TempDir> {
        let temp_dir = TempDir::new().map_err(|e| Error::processing_error(e.to_string()))?;

        // Create .git directory
        fs::create_dir(temp_dir.path().join(".git"))
            .map_err(|e| Error::processing_error(e.to_string()))?;

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "*.log\n")
            .map_err(|e| Error::processing_error(e.to_string()))?;

        Ok(temp_dir)
    }

    /// **What is tested:** Basic filter creation with default settings
    /// **Why it is tested:** Ensures that a filter can be created successfully with proper default values and gitignore loading
    /// **Test conditions:** Creates a temporary repository with .git directory and .gitignore file
    /// **Expectations:** Filter should have gitignore loaded, empty VCS patterns, VCS filtering disabled, and no downstream command
    #[test]
    fn test_filter_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;

        assert!(filter.gitignore.is_some());
        assert!(filter.vcs_patterns.is_empty());
        assert!(!filter.vcs_filtering_enabled); // Default: VCS filtering disabled
        assert!(filter.downstream_command.is_none());
        Ok(())
    }

    /// **What is tested:** Filter configuration with VCS patterns
    /// **Why it is tested:** Verifies that VCS patterns are properly stored and VCS filtering is automatically enabled when patterns are added
    /// **Test conditions:** Creates filter and adds VCS patterns using with_vcs_patterns method
    /// **Expectations:** VCS patterns should be stored correctly and VCS filtering should be enabled
    #[test]
    fn test_filter_with_vcs_patterns() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let patterns = vec![".git/".to_string(), ".svn/".to_string()];
        let filter = Filter::new(temp_dir.path())?.with_vcs_patterns(patterns.clone());

        assert_eq!(filter.vcs_patterns, patterns);
        assert!(filter.vcs_filtering_enabled); // VCS filtering should be enabled when patterns are added
        Ok(())
    }

    /// **What is tested:** Filter configuration with downstream command
    /// **Why it is tested:** Ensures that downstream commands can be properly configured for piping filtered output
    /// **Test conditions:** Creates filter and adds downstream command using with_downstream method
    /// **Expectations:** Downstream command should be stored correctly in the filter
    #[test]
    fn test_filter_with_downstream() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let command = "cat".to_string();
        let filter = Filter::new(temp_dir.path())?.with_downstream(command.clone());

        assert_eq!(filter.downstream_command, Some(command));
        Ok(())
    }

    /// **What is tested:** File path extraction from git diff headers
    /// **Why it is tested:** Critical for identifying which files are being modified to apply filtering rules correctly
    /// **Test conditions:** Tests various diff header formats including normal paths and paths with spaces
    /// **Expectations:** Should correctly extract file paths from valid diff headers and return None for invalid formats
    #[test]
    fn test_extract_file_path() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;

        let line = "diff --git a/src/main.rs b/src/main.rs";
        let path = filter.extract_file_path(line);
        assert_eq!(path, Some("src/main.rs".to_string()));

        let line = "diff --git a/test file.txt b/test file.txt";
        let path = filter.extract_file_path(line);
        assert_eq!(path, Some("test file.txt".to_string()));

        let invalid_line = "not a diff line";
        let path = filter.extract_file_path(invalid_line);
        assert_eq!(path, None);
        Ok(())
    }

    /// **What is tested:** VCS file pattern matching functionality
    /// **Why it is tested:** Ensures that VCS files are correctly identified based on configured patterns for filtering
    /// **Test conditions:** Creates filter with VCS patterns and tests various file paths
    /// **Expectations:** VCS files should match patterns, regular source files should not match
    #[test]
    fn test_is_vcs_file() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let patterns = vec![".git/".to_string(), ".svn/".to_string()];
        let filter = Filter::new(temp_dir.path())?.with_vcs_patterns(patterns);

        assert!(filter.is_vcs_file(".git/config"));
        assert!(filter.is_vcs_file(".svn/entries"));
        assert!(!filter.is_vcs_file("src/main.rs"));
        assert!(!filter.is_vcs_file("README.md"));
        Ok(())
    }

    /// **What is tested:** File inclusion logic when VCS filtering is enabled
    /// **Why it is tested:** Verifies that VCS files are properly excluded when VCS filtering is active
    /// **Test conditions:** Creates filter with VCS patterns enabled and tests file inclusion decisions
    /// **Expectations:** VCS files should be excluded, regular files should be included
    #[test]
    fn test_should_include_file_vcs_enabled() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = create_test_repo()?;
        let patterns = vec![".git/".to_string()];
        let filter = Filter::new(temp_dir.path())?.with_vcs_patterns(patterns);

        // When VCS filtering is enabled, VCS files should be excluded
        assert!(!filter.should_include_file(".git/config"));
        assert!(filter.should_include_file("src/main.rs"));
        Ok(())
    }

    /// **What is tested:** File inclusion logic when VCS filtering is disabled (default behavior)
    /// **Why it is tested:** Ensures that all files are included when VCS filtering is not explicitly enabled
    /// **Test conditions:** Creates filter without VCS patterns and tests file inclusion decisions
    /// **Expectations:** Both VCS files and regular files should be included
    #[test]
    fn test_should_include_file_vcs_disabled() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;

        // When VCS filtering is disabled (default), VCS files should be included
        assert!(filter.should_include_file(".git/config"));
        assert!(filter.should_include_file("src/main.rs"));
        Ok(())
    }

    /// **What is tested:** Basic diff processing functionality without filtering
    /// **Why it is tested:** Validates core diff processing pipeline works correctly for standard input
    /// **Test conditions:** Processes a simple diff with file changes using default filter settings
    /// **Expectations:** Output should contain the complete diff including headers and content changes
    #[test]
    fn test_process_diff_basic() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;

        let diff_content = r"diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 hello
+world
";

        let input = Cursor::new(diff_content);
        let mut output = Vec::new();

        filter.process_diff(input, &mut output)?;

        let output_str = String::from_utf8(output)?;
        assert!(output_str.contains("diff --git a/test.txt b/test.txt"));
        assert!(output_str.contains("+world"));
        Ok(())
    }

    /// **What is tested:** Diff processing with VCS filtering enabled to exclude VCS files
    /// **Why it is tested:** Ensures that VCS files are properly filtered out when VCS filtering is active
    /// **Test conditions:** Processes diff containing both VCS files and regular files with VCS filtering enabled
    /// **Expectations:** VCS files should be excluded from output, regular files should be included
    #[test]
    fn test_process_diff_with_vcs_filtering_enabled(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let patterns = vec![".git/".to_string()];
        let filter = Filter::new(temp_dir.path())?.with_vcs_patterns(patterns);

        let diff_content = r"diff --git a/.git/config b/.git/config
index 1234567..abcdefg 100644
--- a/.git/config
+++ b/.git/config
@@ -1 +1,2 @@
 [core]
+    bare = false
diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 hello
+world
";

        let input = Cursor::new(diff_content);
        let mut output = Vec::new();

        filter.process_diff(input, &mut output)?;

        let output_str = String::from_utf8(output)?;
        // VCS filtering enabled: should exclude .git/config
        assert!(!output_str.contains(".git/config"));
        assert!(output_str.contains("test.txt"));
        assert!(output_str.contains("+world"));
        Ok(())
    }

    /// **What is tested:** Diff processing with VCS filtering disabled (default behavior)
    /// **Why it is tested:** Verifies that all files are included in output when VCS filtering is not enabled
    /// **Test conditions:** Processes diff containing both VCS files and regular files without VCS patterns
    /// **Expectations:** Both VCS files and regular files should be included in output
    #[test]
    fn test_process_diff_with_vcs_filtering_disabled(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;
        // No VCS patterns added, so VCS filtering is disabled

        let diff_content = r"diff --git a/.git/config b/.git/config
index 1234567..abcdefg 100644
--- a/.git/config
+++ b/.git/config
@@ -1 +1,2 @@
 [core]
+    bare = false
diff --git a/test.txt b/test.txt
index 1234567..abcdefg 100644
--- a/test.txt
+++ b/test.txt
@@ -1 +1,2 @@
 hello
+world
";

        let input = Cursor::new(diff_content);
        let mut output = Vec::new();

        filter.process_diff(input, &mut output)?;

        let output_str = String::from_utf8(output)?;
        // VCS filtering disabled: should include .git/config
        assert!(output_str.contains(".git/config"));
        assert!(output_str.contains("test.txt"));
        assert!(output_str.contains("+world"));
        Ok(())
    }

    /// **What is tested:** Handling of empty diff input
    /// **Why it is tested:** Ensures graceful handling of edge case where no diff content is provided
    /// **Test conditions:** Processes empty string input through the diff processing pipeline
    /// **Expectations:** Should handle empty input without errors and produce empty output
    #[test]
    fn test_process_diff_empty() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;

        let input = Cursor::new("");
        let mut output = Vec::new();

        filter.process_diff(input, &mut output)?;

        assert!(output.is_empty());
        Ok(())
    }
}
