//! Root finder module
//!
//! This module provides functionality to find the root directory of a Git repository
//! by analyzing diff content and filesystem structure.

use crate::error::{Error, Result};
use gix::discover;
use relative_path::{RelativePath, RelativePathBuf};
use std::collections::BTreeSet;
use std::io::BufRead;
use std::path::{Path, PathBuf};

/// Context information about a path's relationship to Git repositories
#[derive(Debug, Clone)]
enum PathContext {
    /// Directory is a Git repository
    InRepo,
    /// Directory is not a Git repository, but paths exist in filesystem
    OutsideRepo,
    /// Paths do not exist in the filesystem
    Virtual,
}

/// Represents a candidate root directory with priority scoring
#[derive(Debug, Clone)]
struct RootCandidate {
    /// Path to the candidate directory
    path: PathBuf,
    /// Priority score calculated using additive formula: 1 + git_bonus + gitignore_bonus
    priority_score: u8,
}

impl RootCandidate {
    /// Create a new RootCandidate with calculated priority score
    ///
    /// Uses worktree-aware scoring system:
    /// - Score 6: Git-Repository mit .gitignore
    /// - Score 5: Git-Repository ohne .gitignore
    /// - Score 4: Worktree mit .gitignore
    /// - Score 3: Worktree ohne .gitignore
    /// - Score 2: Non-Git mit .gitignore
    /// - Score 1: Non-Git ohne .gitignore
    fn new(path: PathBuf) -> Self {
        let repository = discover(&path);
        let is_git_repo = repository.is_ok();
        let is_worktree = repository
            .as_ref()
            .map(|repo| repo.workdir().is_some() && Self::is_worktree_directory(repo))
            .unwrap_or(false);
        let has_gitignore = path.join(".gitignore").exists();

        let priority_score = match (is_git_repo, is_worktree, has_gitignore) {
            (true, false, true) => 6,  // Git-Repository mit .gitignore
            (true, false, false) => 5, // Git-Repository ohne .gitignore
            (true, true, true) => 4,   // Worktree mit .gitignore
            (true, true, false) => 3,  // Worktree ohne .gitignore
            (false, _, true) => 2,     // Non-Git mit .gitignore
            (false, _, false) => 1,    // Non-Git ohne .gitignore
        };

        Self {
            path,
            priority_score,
        }
    }

    /// Check if the repository is a worktree
    ///
    /// A worktree is identified by having a .git file (not directory) that points to the main repository
    fn is_worktree_directory(repository: &gix::Repository) -> bool {
        // Check if the .git entry is a file (worktree) rather than a directory (main repo)
        let git_path = repository.workdir().map(|wd| wd.join(".git"));
        if let Some(git_path) = git_path {
            git_path.is_file()
        } else {
            false
        }
    }
}

/// Analysis information about a path
#[derive(Debug)]
struct PathAnalysis {
    /// The relative path being analyzed
    path: PathBuf,
    /// Whether the path is relative
    is_relative: bool,
    /// Whether the path exists in the filesystem
    exists: bool,
}

/// Utility for finding Git repository root directories
pub struct RootFinder;

impl RootFinder {
    /// Find the root directory of a Git repository
    ///
    /// This function implements the new Flow-Chart-Architektur:
    /// 1. Extract and analyze diff paths from diff_reader
    /// 2. Classify context (InRepo, OutsideRepo, Virtual)
    /// 3. Execute appropriate workflow based on context
    /// 4. Return the determined root path
    ///
    /// Flow: AnalyzeDirectory → ExtractDiffPaths → AnalyzePaths → ClassifyContext → \[Workflow\] → RootSelection
    pub fn find_root<R: BufRead>(current_dir: PathBuf, diff_reader: R) -> Result<PathBuf> {
        // 1. Extract and analyze diff paths
        let path_analyses = Self::extract_and_analyze_diff_paths(diff_reader)?;

        // 2. Classify context
        let context = Self::classify_context(&current_dir, &path_analyses);

        // Log context classification

        // 3. Execute appropriate workflow based on context

        // Log determined root after workflow

        match context {
            PathContext::InRepo => Self::process_in_repo_context(&current_dir, path_analyses),
            PathContext::OutsideRepo => Self::process_outside_repo_context(path_analyses),
            PathContext::Virtual => Self::process_virtual_context(&current_dir, path_analyses),
        }
    }

    /// Extract and analyze diff paths from diff reader
    ///
    /// Reads the diff_reader and extracts paths from "diff --git" lines.
    /// Analyzes each path with RelativePath and creates PathAnalysis objects.
    fn extract_and_analyze_diff_paths<R: BufRead>(diff_reader: R) -> Result<Vec<PathAnalysis>> {
        // Optimized functional approach: Single iterator chain without intermediate collection
        diff_reader
            .lines()
            .filter_map(|line_result| {
                match line_result {
                    Ok(line) if line.starts_with("diff --git ") => Some(Ok(line)),
                    Ok(_) => None, // Skip non-diff lines, continue iteration
                    Err(e) => Some(Err(Error::from(e))),
                }
            })
            .collect::<Result<Vec<String>>>()?
            .into_iter()
            .filter_map(|line| Self::parse_diff_header_line(&line))
            .flat_map(|(left_path, right_path)| {
                // Process both paths in a single iterator chain
                [left_path, right_path]
            })
            .map(Self::create_path_analysis)
            .collect()
    }

    /// Create PathAnalysis for a single path with proper error handling
    ///
    /// Centralizes path analysis logic and provides consistent error handling
    /// for path normalization and filesystem operations.
    fn create_path_analysis(path: String) -> Result<PathAnalysis> {
        let path_ref = Path::new(&path);
        let is_relative = !path_ref.is_absolute();

        // Normalize path using RelativePath for consistent handling
        let normalized = RelativePath::new(&path).normalize();
        let normalized_path = normalized.to_path("").to_path_buf();

        // Check existence with error handling for filesystem operations
        let exists = match path_ref.try_exists() {
            Ok(exists) => exists,
            Err(_) => {
                // Fallback to legacy exists() method for compatibility
                // This handles cases where try_exists() might fail due to permissions
                path_ref.exists()
            }
        };

        Ok(PathAnalysis {
            path: normalized_path,
            is_relative,
            exists,
        })
    }

    /// Parse diff header line in "diff --git a/path1 b/path2" format
    ///
    /// Extracts the two file paths from a git diff header line.
    fn parse_diff_header_line(line: &str) -> Option<(String, String)> {
        // Parse "diff --git a/path1 b/path2" format
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 4 && parts[0] == "diff" && parts[1] == "--git" {
            let left_path = parts[2].strip_prefix("a/").unwrap_or(parts[2]);
            let right_path = parts[3].strip_prefix("b/").unwrap_or(parts[3]);
            Some((left_path.to_string(), right_path.to_string()))
        } else {
            None
        }
    }

    /// Classify context as InRepo, OutsideRepo or Virtual
    ///
    /// Uses gix::discover() for Git repository detection.
    /// Classifies the context based on directory analysis and path existence.
    fn classify_context(directory: &Path, path_analyses: &[PathAnalysis]) -> PathContext {
        // 1. Check if directory is a Git repository
        if discover(directory).is_ok() {
            return PathContext::InRepo;
        }

        // 2. Check if paths exist in filesystem
        let all_paths_exist = path_analyses.iter().all(|analysis| analysis.exists);

        if all_paths_exist {
            PathContext::OutsideRepo
        } else {
            PathContext::Virtual
        }
    }

    /// Process paths in Git repository context
    ///
    /// Determines Git root and checks for external paths.
    /// Returns Git root or delegates to process_outside_repo_context if external paths found.
    fn process_in_repo_context(
        directory: &Path,
        path_analyses: Vec<PathAnalysis>,
    ) -> Result<PathBuf> {
        // 1. Determine Git root using worktree-aware logic
        let git_root = discover(directory)
            .map(|repo| Self::get_worktree_aware_root(&repo))
            .map_err(|err| {
                Error::processing_error(format!("Failed to discover Git repository: {err}"))
            })?;

        // 2. Check for external paths
        let has_external_paths = Self::check_for_external_paths(&path_analyses, &git_root);

        if !has_external_paths {
            // Only local paths → use Git root
            Ok(git_root)
        } else {
            // External paths present → delegate to OutsideRepo workflow
            Self::process_outside_repo_context(path_analyses)
        }
    }

    /// Get worktree-aware root directory
    ///
    /// Returns the appropriate root directory based on repository type:
    /// - For worktrees: Returns the worktree working directory
    /// - For bare repositories: Returns the git directory
    /// - For normal repositories: Returns the working directory
    fn get_worktree_aware_root(repository: &gix::Repository) -> PathBuf {
        if let Some(worktree_dir) = repository.workdir() {
            // For worktrees and normal repositories: Use the working directory
            worktree_dir.to_path_buf()
        } else {
            // For bare repositories: Use git_dir
            repository.git_dir().to_path_buf()
        }
    }

    /// Check if any paths are external to the Git repository
    ///
    /// Returns true if any absolute path lies outside the Git repository root.
    fn check_for_external_paths(path_analyses: &[PathAnalysis], git_root: &Path) -> bool {
        // Functional approach: filter absolute paths and check if any are external to git_root
        path_analyses
            .iter()
            .filter(|analysis| !analysis.is_relative)
            .any(|analysis| !analysis.path.starts_with(git_root))
    }

    /// Process paths outside Git repository context
    ///
    /// Extracts path pairs, performs suffix-based analysis, uses fallback if no suffix found,
    /// and applies root selection to candidates.
    fn process_outside_repo_context(path_analyses: Vec<PathAnalysis>) -> Result<PathBuf> {
        let path_pairs = Self::extract_path_pairs_from_analyses(&path_analyses);

        // Suffix-based analysis
        if let Some((left_root, right_root)) = Self::calculate_roots_by_suffix(&path_pairs) {
            Ok(Self::apply_root_selection(vec![left_root, right_root]))
        } else {
            // Fallback: Hierarchical search
            let fallback_roots = Self::hierarchical_fallback_search(&path_pairs);
            Ok(Self::apply_root_selection(fallback_roots))
        }
    }

    /// Extract path pairs from path analyses
    ///
    /// Groups paths pairwise (left, right) from consecutive PathAnalysis entries.
    fn extract_path_pairs_from_analyses(path_analyses: &[PathAnalysis]) -> Vec<(PathBuf, PathBuf)> {
        // Group paths pairwise (left, right) using functional iterator approach
        let pairs = path_analyses
            .chunks(2)
            .filter_map(|chunk| {
                if chunk.len() == 2 {
                    Some((chunk[0].path.clone(), chunk[1].path.clone()))
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        pairs
    }

    /// Calculate roots by suffix analysis
    ///
    /// Finds the longest common suffix of all path pairs and derives roots by suffix removal.
    fn calculate_roots_by_suffix(path_pairs: &[(PathBuf, PathBuf)]) -> Option<(PathBuf, PathBuf)> {
        if path_pairs.is_empty() {
            return None;
        }

        // Convert PathBuf pairs to String pairs for the new suffix analysis function
        let string_pairs = path_pairs
            .iter()
            .map(|(left, right)| {
                (
                    left.to_string_lossy().to_string(),
                    right.to_string_lossy().to_string(),
                )
            })
            .collect::<Vec<_>>();

        // Use the complete suffix analysis logic
        if let Some(common_suffix) = Self::find_common_suffix_between_strings(&string_pairs) {
            // Convert first pair to RelativePathBuf for root extraction
            let first_pair = &path_pairs[0];
            let left_rel = RelativePath::new(&first_pair.0.to_string_lossy()).normalize();
            let right_rel = RelativePath::new(&first_pair.1.to_string_lossy()).normalize();

            // Extract roots using the common suffix
            if let (Some(left_root), Some(right_root)) = (
                Self::extract_root_by_suffix_removal(&left_rel, &common_suffix),
                Self::extract_root_by_suffix_removal(&right_rel, &common_suffix),
            ) {
                Some((
                    left_root.to_path("").to_path_buf(),
                    right_root.to_path("").to_path_buf(),
                ))
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Find common suffix for a single path pair using RelativePath
    fn find_common_suffix_for_pair(
        left_path: &RelativePathBuf,
        right_path: &RelativePathBuf,
    ) -> Option<String> {
        let left_components: Vec<_> = left_path.components().collect();
        let right_components: Vec<_> = right_path.components().collect();

        // Functional approach: find common suffix using iterators
        let common_len = left_components
            .iter()
            .rev()
            .zip(right_components.iter().rev())
            .take_while(|(left, right)| left == right)
            .count();

        // Functional suffix construction with early return
        (common_len > 0)
            .then(|| {
                let suffix_start = left_components.len() - common_len;
                (suffix_start < left_components.len()).then(|| {
                    left_components[suffix_start..]
                        .iter()
                        .map(|comp| comp.as_str())
                        .collect::<Vec<_>>()
                        .join("/")
                })
            })
            .flatten()
    }

    /// Find common suffix between two path strings
    fn find_common_suffix_between_paths(suffix1: &str, suffix2: &str) -> Option<String> {
        let path1 = RelativePath::new(suffix1).normalize();
        let path2 = RelativePath::new(suffix2).normalize();
        Self::find_common_suffix_for_pair(&path1, &path2)
    }

    /// Find longest common suffix between multiple path pairs
    ///
    /// Implements the complete suffix analysis logic as described in the documentation:
    /// 1. For each pair, determine the suffix
    /// 2. Iteratively calculate the longest common suffix of all pairs (reduction)
    /// 3. Returns the common suffix or None if no common suffix exists
    ///
    /// This function performs the full logic and does not just delegate to the pair function.
    fn find_common_suffix_between_strings(path_pairs: &[(String, String)]) -> Option<String> {
        if path_pairs.is_empty() {
            return None;
        }

        // Convert string pairs to RelativePathBuf pairs
        let relative_pairs: Vec<(RelativePathBuf, RelativePathBuf)> = path_pairs
            .iter()
            .map(|(left, right)| {
                (
                    RelativePath::new(left).normalize(),
                    RelativePath::new(right).normalize(),
                )
            })
            .collect();

        // Start with suffix of the first pair
        let mut common_suffix =
            Self::find_common_suffix_for_pair(&relative_pairs[0].0, &relative_pairs[0].1)?;

        // Reduce for each additional pair
        for (left_path, right_path) in &relative_pairs[1..] {
            if let Some(pair_suffix) = Self::find_common_suffix_for_pair(left_path, right_path) {
                // Find common part between current common suffix and pair suffix
                common_suffix =
                    Self::find_common_suffix_between_paths(&common_suffix, &pair_suffix)
                        .unwrap_or_else(String::new);
                if common_suffix.is_empty() {
                    return None;
                }
            } else {
                return None; // No suffix found for this pair
            }
        }

        Some(common_suffix)
    }

    /// Extract root by removing suffix from RelativePath
    fn extract_root_by_suffix_removal(
        full_path: &RelativePathBuf,
        suffix: &str,
    ) -> Option<RelativePathBuf> {
        let full_components: Vec<_> = full_path.components().collect();
        let suffix_components: Vec<_> = RelativePath::new(suffix).components().collect();

        // Functional approach: Check length and suffix match in one chain
        (full_components.len() >= suffix_components.len())
            .then(|| {
                let start_idx = full_components.len() - suffix_components.len();
                let path_suffix = &full_components[start_idx..];

                // Functional suffix matching
                path_suffix
                    .iter()
                    .zip(suffix_components.iter())
                    .all(|(a, b)| a == b)
                    .then(|| {
                        // Functional root extraction
                        if start_idx > 0 {
                            let root_str = full_components[..start_idx]
                                .iter()
                                .map(|comp| comp.as_str())
                                .collect::<Vec<_>>()
                                .join("/");
                            RelativePath::new(&root_str).normalize()
                        } else {
                            RelativePath::new(".").normalize()
                        }
                    })
            })
            .flatten()
    }

    /// Hierarchical fallback search when suffix analysis fails
    ///
    /// Collects parent directories of all path pairs as fallback candidates.
    /// Removes duplicates and returns a list of candidates.
    /// See docs/implementation_guide.md:373-399 and docs/flowchart_extended.md:269
    fn hierarchical_fallback_search(path_pairs: &[(PathBuf, PathBuf)]) -> Vec<PathBuf> {
        // Functional approach: Use flat_map and iterator chains with BTreeSet for automatic sorting and deduplication
        let candidates: Vec<PathBuf> = path_pairs
            .iter()
            .flat_map(|(left_path, right_path)| {
                let left_rel = RelativePath::new(&left_path.to_string_lossy()).normalize();
                let right_rel = RelativePath::new(&right_path.to_string_lossy()).normalize();

                // Collect parent directories to avoid borrow checker issues
                let mut parents = Vec::new();
                if let Some(left_parent) = left_rel.parent() {
                    parents.push(left_parent.to_path("").to_path_buf());
                }
                if let Some(right_parent) = right_rel.parent() {
                    parents.push(right_parent.to_path("").to_path_buf());
                }
                parents
            })
            .collect::<BTreeSet<_>>() // Automatic sorting and deduplication
            .into_iter()
            .collect();

        if candidates.is_empty() {
            vec![PathBuf::from(".")]
        } else {
            candidates
        }
    }

    /// Apply root selection logic to candidate paths
    ///
    /// Evaluates all candidates with the Priority Score System.
    /// Sorts descending by score and returns the candidate with the highest score.
    /// Fallback: Returns current directory if no candidates.
    /// See docs/implementation_guide.md:357-371 and docs/flowchart_extended.md:119-187, 290-302
    fn apply_root_selection(candidates: Vec<PathBuf>) -> PathBuf {
        if candidates.is_empty() {
            return PathBuf::from(".");
        }

        // Functional approach: Create, sort, and select in one iterator chain
        candidates
            .into_iter()
            .map(RootCandidate::new)
            .max_by_key(|candidate| candidate.priority_score)
            .map(|candidate| candidate.path)
            .unwrap_or_else(|| PathBuf::from("."))
    }

    /// Process virtual context paths
    ///
    /// Follows documented virtual workflow logic exclusively:
    /// - Uses left virtual root from suffix analysis
    /// - Falls back to heuristic if suffix analysis fails
    fn process_virtual_context(
        _current_dir: &Path,
        path_analyses: Vec<PathAnalysis>,
    ) -> Result<PathBuf> {
        // Standard virtual path logic according to documentation
        let virtual_pairs = Self::extract_path_pairs_from_analyses(&path_analyses);

        if let Some((left_root, _right_root)) =
            Self::calculate_virtual_roots_by_suffix(&virtual_pairs)
        {
            Ok(left_root) // Always use left root for virtual paths
        } else {
            Ok(Self::virtual_heuristic_fallback(&virtual_pairs))
        }
    }

    /// Calculate virtual roots by suffix analysis
    ///
    /// Same logic as calculate_roots_by_suffix, but for virtual paths.
    fn calculate_virtual_roots_by_suffix(
        virtual_pairs: &[(PathBuf, PathBuf)],
    ) -> Option<(PathBuf, PathBuf)> {
        // Same logic as calculate_roots_by_suffix, but for virtual paths
        Self::calculate_roots_by_suffix(virtual_pairs)
    }

    /// Virtual heuristic fallback when suffix analysis fails
    ///
    /// Uses the first directory of the first path as fallback.
    fn virtual_heuristic_fallback(virtual_pairs: &[(PathBuf, PathBuf)]) -> PathBuf {
        // Fallback: Use the first directory of the first path
        if let Some((left_path, _)) = virtual_pairs.first() {
            if let Some(parent) = left_path.parent() {
                return parent.to_path_buf();
            }
        }

        PathBuf::from(".")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::io::Cursor;
    use tempfile::TempDir;

    /// Check if a directory is a Git repository root (test helper function)
    fn is_git_root<P: AsRef<Path>>(path: P) -> bool {
        path.as_ref().join(".git").exists()
    }

    fn create_test_repo() -> Result<TempDir> {
        let temp_dir = TempDir::new().map_err(|e| Error::processing_error(e.to_string()))?;

        // Initialize a real Git repository using gix
        gix::init(temp_dir.path())
            .map_err(|e| Error::processing_error(format!("Failed to initialize git repo: {e}")))?;

        Ok(temp_dir)
    }

    fn create_nested_test_repo() -> Result<(TempDir, PathBuf)> {
        let temp_dir = TempDir::new().map_err(|e| Error::processing_error(e.to_string()))?;

        // Initialize a real Git repository using gix
        gix::init(temp_dir.path())
            .map_err(|e| Error::processing_error(format!("Failed to initialize git repo: {e}")))?;

        // Create nested directory structure
        let nested_dir = temp_dir.path().join("src").join("subdir");
        fs::create_dir_all(&nested_dir).map_err(|e| Error::processing_error(e.to_string()))?;

        Ok((temp_dir, nested_dir))
    }

    /// **What is tested:** Root finding when starting from a Git repository root directory
    /// **Why it is tested:** Ensures that the root finder correctly identifies the current directory as the root when already at repository root
    /// **Test conditions:** Creates a Git repository and calls find_root from the repository root with empty diff
    /// **Expectations:** Should return the repository root directory path
    #[test]
    fn test_find_root_from_repo_root() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let empty_diff = Cursor::new("");
        let root = RootFinder::find_root(temp_dir.path().to_path_buf(), empty_diff)?;

        assert_eq!(root, temp_dir.path());
        Ok(())
    }

    /// **What is tested:** Root finding when starting from a nested directory within a Git repository
    /// **Why it is tested:** Validates that the root finder can traverse up the directory tree to find the repository root
    /// **Test conditions:** Creates a Git repository with nested directory structure and calls find_root from nested location
    /// **Expectations:** Should return the repository root directory, not the nested directory
    #[test]
    fn test_find_root_from_nested_directory() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let (temp_dir, nested_dir) = create_nested_test_repo()?;
        let empty_diff = Cursor::new("");
        let root = RootFinder::find_root(nested_dir, empty_diff)?;

        assert_eq!(root, temp_dir.path());
        Ok(())
    }

    /// **What is tested:** Root finding behavior when not in a Git repository
    /// **Why it is tested:** Ensures graceful handling of non-Git environments and proper fallback behavior
    /// **Test conditions:** Creates a temporary directory without Git initialization and empty diff input
    /// **Expectations:** Should return current directory (".") as fallback when no Git repository is found
    #[test]
    fn test_find_root_no_git_repo() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        // Don't create .git directory

        let empty_diff = Cursor::new("");
        let result = RootFinder::find_root(temp_dir.path().to_path_buf(), empty_diff);
        assert!(result.is_ok());

        // With empty diff and no Git repository, virtual context returns "."
        let returned_path = result?;
        assert_eq!(returned_path, PathBuf::from("."));
        Ok(())
    }

    /// **What is tested:** Helper function for detecting Git repository roots
    /// **Why it is tested:** Validates the test utility function used to verify Git repository detection
    /// **Test conditions:** Tests both Git repository directory and non-Git directory
    /// **Expectations:** Should return true for Git repositories and false for non-Git directories
    #[test]
    fn test_is_git_root() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        assert!(is_git_root(temp_dir.path()));

        let non_git_dir = TempDir::new()?;
        assert!(!is_git_root(non_git_dir.path()));
        Ok(())
    }

    /// **What is tested:** Git repository discovery from various directory levels using gix library
    /// **Why it is tested:** Ensures that the underlying gix::discover function works correctly for repository traversal
    /// **Test conditions:** Creates nested Git repository structure and tests discovery from multiple directory levels
    /// **Expectations:** Should find the same repository root from all nested directory levels
    #[test]
    fn test_git_discovery_traversal() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let (temp_dir, nested_dir) = create_nested_test_repo()?;

        // Test that gix::discover can find the root from various levels
        let repo_from_nested = discover(&nested_dir)?;
        let root_from_nested = repo_from_nested
            .workdir()
            .unwrap_or_else(|| repo_from_nested.git_dir());
        assert_eq!(root_from_nested, temp_dir.path());

        let src_dir = temp_dir.path().join("src");
        let repo_from_src = discover(&src_dir)?;
        let root_from_src = repo_from_src
            .workdir()
            .unwrap_or_else(|| repo_from_src.git_dir());
        assert_eq!(root_from_src, temp_dir.path());

        let repo_from_root = discover(temp_dir.path())?;
        let root_from_root = repo_from_root
            .workdir()
            .unwrap_or_else(|| repo_from_root.git_dir());
        assert_eq!(root_from_root, temp_dir.path());
        Ok(())
    }

    /// **What is tested:** Priority scoring system for root candidates
    /// **Why it is tested:** Validates that the scoring algorithm correctly assigns priorities based on Git status and gitignore presence
    /// **Test conditions:** Creates different types of directories (Git/non-Git, with/without gitignore) and checks scores
    /// **Expectations:** Non-Git without gitignore should score 1, Git without gitignore should score 5
    #[test]
    fn test_root_candidate_priority_scoring() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        // Test Score 1: Non-Git directory without .gitignore
        let temp_dir = TempDir::new()?;
        let candidate = RootCandidate::new(temp_dir.path().to_path_buf());

        assert_eq!(candidate.priority_score, 1);

        // Test Score 5: Git repository without .gitignore
        let git_temp_dir = create_test_repo()?;
        let git_candidate = RootCandidate::new(git_temp_dir.path().to_path_buf());

        assert_eq!(git_candidate.priority_score, 5);
        Ok(())
    }

    /// **What is tested:** Priority scoring for candidates with gitignore files
    /// **Why it is tested:** Ensures that presence of gitignore files increases priority scores appropriately
    /// **Test conditions:** Creates directories with gitignore files in both Git and non-Git contexts
    /// **Expectations:** Non-Git with gitignore should score 2, Git with gitignore should score 6
    #[test]
    fn test_root_candidate_with_gitignore() -> std::result::Result<(), Box<dyn std::error::Error>> {
        use std::fs::File;

        // Test Score 2: Non-Git directory with .gitignore
        let temp_dir = TempDir::new()?;
        let gitignore_path = temp_dir.path().join(".gitignore");
        File::create(&gitignore_path)?;

        let candidate = RootCandidate::new(temp_dir.path().to_path_buf());
        assert_eq!(candidate.priority_score, 2);

        // Test Score 6: Git repository with .gitignore
        let git_temp_dir = create_test_repo()?;
        let git_gitignore_path = git_temp_dir.path().join(".gitignore");
        File::create(&git_gitignore_path)?;

        let git_candidate = RootCandidate::new(git_temp_dir.path().to_path_buf());
        assert_eq!(git_candidate.priority_score, 6);
        Ok(())
    }

    /// **What is tested:** PathAnalysis struct creation and field validation
    /// **Why it is tested:** Ensures that path analysis correctly identifies path properties (existence, relativity)
    /// **Test conditions:** Creates PathAnalysis for both existing and non-existing paths
    /// **Expectations:** Should correctly identify path existence and absolute/relative nature
    #[test]
    fn test_path_analysis_creation() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;
        let existing_path = temp_dir.path().to_path_buf();
        let non_existing_path = PathBuf::from("/non/existing/path");

        let existing_analysis = PathAnalysis {
            path: existing_path.clone(),
            is_relative: !existing_path.is_absolute(),
            exists: existing_path.exists(),
        };

        let non_existing_analysis = PathAnalysis {
            path: non_existing_path.clone(),
            is_relative: !non_existing_path.is_absolute(),
            exists: non_existing_path.exists(),
        };

        assert!(existing_analysis.exists);
        assert!(!existing_analysis.is_relative); // temp paths are usually absolute
        assert!(!non_existing_analysis.exists);
        assert!(!non_existing_analysis.is_relative); // absolute path
        Ok(())
    }

    /// **What is tested:** PathContext enum variants and Debug trait implementation
    /// **Why it is tested:** Validates that all context variants can be created and properly formatted for debugging
    /// **Test conditions:** Creates all PathContext variants and tests Debug formatting
    /// **Expectations:** All variants should be creatable and Debug output should contain variant names
    #[test]
    fn test_path_context_variants() {
        // Test that all PathContext variants can be created
        let _in_repo = PathContext::InRepo;
        let _outside_repo = PathContext::OutsideRepo;
        let _virtual = PathContext::Virtual;

        // Test Debug trait
        assert!(format!("{:?}", PathContext::InRepo).contains("InRepo"));
        assert!(format!("{:?}", PathContext::OutsideRepo).contains("OutsideRepo"));
        assert!(format!("{:?}", PathContext::Virtual).contains("Virtual"));
    }

    /// **What is tested:** Parsing of Git diff header lines to extract file paths
    /// **Why it is tested:** Critical for extracting file paths from diff content for root finding analysis
    /// **Test conditions:** Tests various diff header formats including valid, invalid, and incomplete lines
    /// **Expectations:** Should correctly extract path pairs from valid headers and return None for invalid formats
    #[test]
    fn test_parse_diff_header_line() {
        // Test valid diff header
        let line = "diff --git a/src/main.rs b/src/main.rs";
        let result = RootFinder::parse_diff_header_line(line);
        assert_eq!(
            result,
            Some(("src/main.rs".to_string(), "src/main.rs".to_string()))
        );

        // Test with different paths
        let line2 = "diff --git a/old/file.txt b/new/file.txt";
        let result2 = RootFinder::parse_diff_header_line(line2);
        assert_eq!(
            result2,
            Some(("old/file.txt".to_string(), "new/file.txt".to_string()))
        );

        // Test invalid line
        let invalid_line = "not a diff line";
        let result3 = RootFinder::parse_diff_header_line(invalid_line);
        assert_eq!(result3, None);

        // Test incomplete diff line
        let incomplete_line = "diff --git a/file.txt";
        let result4 = RootFinder::parse_diff_header_line(incomplete_line);
        assert_eq!(result4, None);
    }

    /// **What is tested:** Extraction and analysis of file paths from diff content
    /// **Why it is tested:** Validates the core functionality of parsing diff content to identify file paths for root finding
    /// **Test conditions:** Processes multi-file diff content with various file operations (modify, create)
    /// **Expectations:** Should extract all file paths correctly and create proper PathAnalysis objects
    #[test]
    fn test_extract_and_analyze_diff_paths() -> std::result::Result<(), Box<dyn std::error::Error>>
    {
        let diff_content = r#"diff --git a/src/main.rs b/src/main.rs
index 1234567..abcdefg 100644
--- a/src/main.rs
+++ b/src/main.rs
@@ -1,3 +1,4 @@
 fn main() {
+    println!("Hello, world!");
 }
diff --git a/README.md b/README.md
new file mode 100644
index 0000000..fedcba9
--- /dev/null
+++ b/README.md
@@ -0,0 +1 @@
+# Test Project
"#;

        let cursor = Cursor::new(diff_content);
        let result = RootFinder::extract_and_analyze_diff_paths(cursor);
        assert!(result.is_ok());

        let path_analyses = result?;
        assert_eq!(path_analyses.len(), 4); // 2 files × 2 paths each (a/ and b/)

        // Check that paths were extracted correctly
        let paths: Vec<String> = path_analyses
            .iter()
            .map(|analysis| analysis.path.to_string_lossy().to_string())
            .collect();

        assert!(paths.contains(&"src/main.rs".to_string()));
        assert!(paths.contains(&"README.md".to_string()));
        Ok(())
    }

    /// **What is tested:** Context classification for directories within Git repositories
    /// **Why it is tested:** Ensures that the context classifier correctly identifies Git repository environments
    /// **Test conditions:** Creates a Git repository and tests context classification with empty path analyses
    /// **Expectations:** Should classify context as InRepo for Git repository directories
    #[test]
    fn test_classify_context_in_repo() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = create_test_repo()?;
        let path_analyses = vec![]; // Empty for this test

        let context = RootFinder::classify_context(temp_dir.path(), &path_analyses);

        match context {
            PathContext::InRepo => {} // Expected
            _ => panic!("Expected InRepo context for git repository"),
        }
        Ok(())
    }

    /// **What is tested:** Context classification for directories outside Git repositories with existing files
    /// **Why it is tested:** Validates classification logic for non-Git environments where referenced files exist
    /// **Test conditions:** Creates non-Git directory with existing test file and analyzes context
    /// **Expectations:** Should classify context as OutsideRepo when files exist but no Git repository is present
    #[test]
    fn test_classify_context_outside_repo() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create a test file that exists
        let test_file = temp_dir.path().join("test.txt");
        fs::write(&test_file, "test content")?;

        let path_analyses = vec![PathAnalysis {
            path: test_file,
            is_relative: true,
            exists: true,
        }];

        let context = RootFinder::classify_context(temp_dir.path(), &path_analyses);

        match context {
            PathContext::OutsideRepo => {} // Expected
            _ => panic!("Expected OutsideRepo context for non-git directory with existing paths"),
        }
        Ok(())
    }

    /// **What is tested:** Context classification for virtual paths (non-existing files)
    /// **Why it is tested:** Ensures proper handling of diff content referencing files that don't exist in filesystem
    /// **Test conditions:** Creates non-Git directory with path analyses for non-existing files
    /// **Expectations:** Should classify context as Virtual when files don't exist and no Git repository is present
    #[test]
    fn test_classify_context_virtual() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let temp_dir = TempDir::new()?;

        // Create path analyses with non-existing files
        let path_analyses = vec![PathAnalysis {
            path: PathBuf::from("non/existing/file.txt"),
            is_relative: true,
            exists: false,
        }];

        let context = RootFinder::classify_context(temp_dir.path(), &path_analyses);

        match context {
            PathContext::Virtual => {} // Expected
            _ => panic!("Expected Virtual context for non-git directory with non-existing paths"),
        }
        Ok(())
    }

    /// **What is tested:** Common suffix finding algorithm for multiple path pairs
    /// **Why it is tested:** Critical for suffix-based root finding when paths share common directory structures
    /// **Test conditions:** Tests path pairs with actual common suffixes across multiple pairs
    /// **Expectations:** Should correctly identify "src/main.rs" as common suffix across all provided path pairs
    #[test]
    fn test_find_common_suffix_between_strings_basic() {
        // Test with paths that have actual common suffixes
        let path_pairs = vec![
            (
                "project/src/main.rs".to_string(),
                "other/src/main.rs".to_string(),
            ),
            (
                "app/src/main.rs".to_string(),
                "test/src/main.rs".to_string(),
            ),
        ];

        let result = RootFinder::find_common_suffix_between_strings(&path_pairs);
        // Should find "src/main.rs" as common suffix across all pairs
        assert_eq!(result, Some("src/main.rs".to_string()));
    }

    /// **What is tested:** Common suffix finding when no common suffix exists
    /// **Why it is tested:** Ensures algorithm correctly handles cases where paths don't share common suffixes
    /// **Test conditions:** Tests path pairs with completely different file structures
    /// **Expectations:** Should return None when no common suffix can be found across path pairs
    #[test]
    fn test_find_common_suffix_between_strings_no_common() {
        let path_pairs = vec![
            ("src/main.rs".to_string(), "lib/utils.rs".to_string()),
            ("app/config.rs".to_string(), "test/helper.rs".to_string()),
        ];

        let result = RootFinder::find_common_suffix_between_strings(&path_pairs);
        assert_eq!(result, None);
    }

    /// **What is tested:** Common suffix finding with empty input
    /// **Why it is tested:** Validates edge case handling when no path pairs are provided
    /// **Test conditions:** Calls suffix finding function with empty vector of path pairs
    /// **Expectations:** Should return None for empty input without errors
    #[test]
    fn test_find_common_suffix_between_strings_empty() {
        let path_pairs: Vec<(String, String)> = vec![];
        let result = RootFinder::find_common_suffix_between_strings(&path_pairs);
        assert_eq!(result, None);
    }

    /// **What is tested:** Common suffix finding with single path pair
    /// **Why it is tested:** Ensures algorithm works correctly with minimal input (single pair)
    /// **Test conditions:** Tests suffix finding with only one path pair that has a common suffix
    /// **Expectations:** Should correctly identify the suffix from the single pair
    #[test]
    fn test_find_common_suffix_between_strings_single_pair() {
        let path_pairs = vec![(
            "project/src/main.rs".to_string(),
            "other/src/main.rs".to_string(),
        )];

        let result = RootFinder::find_common_suffix_between_strings(&path_pairs);
        assert_eq!(result, Some("src/main.rs".to_string()));
    }

    /// **What is tested:** Common suffix finding between two relative paths
    /// **Why it is tested:** Tests the helper function for finding suffixes between individual path pairs
    /// **Test conditions:** Tests both cases with common suffixes and without common suffixes
    /// **Expectations:** Should find common suffix when it exists and return None when it doesn't
    #[test]
    fn test_find_common_suffix_between_relative_paths() {
        // Test actual suffix finding
        let result =
            RootFinder::find_common_suffix_between_paths("project/src/main", "other/src/main");
        assert_eq!(result, Some("src/main".to_string()));

        let result2 = RootFinder::find_common_suffix_between_paths("app/main", "lib/utils");
        assert_eq!(result2, None);
    }
}
