//! diff-gitignore-filter library
//!
//! A pure stream-filter for Git diffs that respects .gitignore patterns.
//!
//! # Examples
//!
//! Basic usage:
//!
//! ```rust
//! use diff_gitignore_filter::Filter;
//! use std::io::Cursor;
//!
//! let filter = Filter::new(".")?;
//! let input = "diff --git a/ignored.log b/ignored.log\n";
//! let mut output = Vec::new();
//!
//! filter.process_diff(Cursor::new(input), &mut output)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```

pub mod config;
pub mod error;
pub mod filter;
pub mod root_finder;

pub use config::{AppConfig, ConfigError, GitConfig, GitConfigReader, SystemGitConfigReader};
pub use error::{Error, Result};
pub use filter::Filter;
pub use root_finder::RootFinder;

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use tempfile::TempDir;

    fn create_test_repo() -> Result<TempDir> {
        let temp_dir = TempDir::new()?;

        // Create .git directory
        fs::create_dir(temp_dir.path().join(".git"))?;

        // Create .gitignore
        fs::write(temp_dir.path().join(".gitignore"), "*.log\n")?;

        Ok(temp_dir)
    }

    /// **What is tested:** Basic library functionality integration test
    /// **Why it is tested:** Ensures that the main library components work together correctly for basic diff processing
    /// **Test conditions:** Creates a test repository with .gitignore and processes a simple diff through the filter
    /// **Expectations:** Filter should successfully process the diff and produce non-empty output
    #[test]
    fn test_basic_functionality() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let filter = Filter::new(temp_dir.path())?;
        let input = "diff --git a/test.txt b/test.txt\n";
        let mut output = Vec::new();

        filter.process_diff(Cursor::new(input), &mut output)?;

        assert!(!output.is_empty());
        Ok(())
    }
}
