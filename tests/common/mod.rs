//! Unified Test Framework for diff-gitignore-filter
//!
//! This module provides a clean, consistent testing framework with unified design patterns.
//! All components follow the same architectural principles and naming conventions.

// New unified framework modules
pub mod framework;
pub mod test_utilities;

// Re-export the main framework API
#[allow(unused_imports)]
pub use framework::*;

// Framework tests have been moved to tests/framework_tests.rs to avoid duplication
// across multiple test files that import the common module
