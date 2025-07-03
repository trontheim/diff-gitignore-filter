//! Error handling module
//!
//! This module provides unified error handling for the diff-gitignore-filter application.

use std::fmt;

/// Result type alias for the application
pub type Result<T> = std::result::Result<T, Error>;

/// Main error type for the application
#[derive(Debug)]
pub enum Error {
    /// IO-related errors
    Io(std::io::Error),
    /// Processing errors with custom messages
    Processing(String),
    /// Configuration errors
    Config(crate::config::ConfigError),
    /// Downstream command spawn failed
    DownstreamSpawnFailed(String),
    /// Downstream process failed
    DownstreamProcessFailed(String),
}

impl Error {
    /// Create a processing error with a custom message
    pub fn processing_error(message: String) -> Self {
        Error::Processing(message)
    }

    /// Chain this error with a function for functional composition
    ///
    /// This allows for functional error handling patterns where errors can be
    /// transformed or handled in a composable way.
    pub fn chain<F, T>(self, f: F) -> Result<T>
    where
        F: FnOnce(Self) -> Result<T>,
    {
        f(self)
    }

    /// Apply an alternative error handling function if this error occurs
    ///
    /// This provides a functional way to handle errors by allowing transformation
    /// or recovery strategies to be applied.
    pub fn or_else<F>(self, f: F) -> Self
    where
        F: FnOnce(Self) -> Self,
    {
        f(self)
    }

    /// Map the context/message of this error using a transformation function
    ///
    /// This allows functional transformation of error messages while preserving
    /// the error type structure.
    pub fn map_context<F>(self, f: F) -> Self
    where
        F: FnOnce(String) -> String,
    {
        match self {
            Error::Processing(msg) => Error::Processing(f(msg)),
            Error::DownstreamSpawnFailed(msg) => Error::DownstreamSpawnFailed(f(msg)),
            Error::DownstreamProcessFailed(msg) => Error::DownstreamProcessFailed(f(msg)),
            other => other,
        }
    }

    /// Combine multiple errors into a single error with aggregated context
    ///
    /// This functional combinator allows collecting multiple errors and presenting
    /// them as a single processing error with combined context.
    pub fn combine_errors(errors: Vec<Self>) -> Self {
        if errors.is_empty() {
            return Error::Processing("No errors to combine".to_string());
        }

        if errors.len() == 1 {
            return errors.into_iter().next().unwrap();
        }

        let combined_message = errors
            .iter()
            .enumerate()
            .map(|(i, err)| format!("Error {}: {}", i + 1, err))
            .collect::<Vec<_>>()
            .join("; ");

        Error::Processing(format!("Multiple errors occurred: {combined_message}"))
    }

    /// Collect errors from an iterator, returning the first error or success
    ///
    /// This functional helper processes an iterator of Results and either returns
    /// the first error encountered or indicates success.
    pub fn collect_errors<I, T>(results: I) -> Result<Vec<T>>
    where
        I: IntoIterator<Item = Result<T>>,
    {
        results.into_iter().collect()
    }

    /// Return the first error from a collection of Results
    ///
    /// This functional combinator finds the first error in a collection of Results,
    /// useful for error prioritization and early termination patterns.
    pub fn first_error<I, T>(results: I) -> Option<Self>
    where
        I: IntoIterator<Item = Result<T>>,
    {
        results.into_iter().find_map(|result| result.err())
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Io(err) => write!(f, "IO error: {err}"),
            Error::Processing(msg) => write!(f, "Processing error: {msg}"),
            Error::Config(err) => write!(f, "Configuration error: {err}"),
            Error::DownstreamSpawnFailed(msg) => write!(f, "DownstreamSpawnFailed: {msg}"),
            Error::DownstreamProcessFailed(msg) => write!(f, "DownstreamProcessFailed: {msg}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(err) => Some(err),
            Error::Processing(_) => None,
            Error::Config(err) => Some(err),
            Error::DownstreamSpawnFailed(_) => None,
            Error::DownstreamProcessFailed(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<crate::config::ConfigError> for Error {
    fn from(err: crate::config::ConfigError) -> Self {
        Error::Config(err)
    }
}

/// Functional extensions for Result types to enable better composition
///
/// This trait provides functional programming patterns for Result handling,
/// enabling more expressive and composable error handling code.
pub trait ResultExt<T> {
    /// Add context to an error using a closure
    ///
    /// This allows adding contextual information to errors in a functional way,
    /// useful for providing more detailed error messages.
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;

    /// Chain error handling with another operation
    ///
    /// This provides a functional way to chain operations that might fail,
    /// similar to flatMap in other functional languages.
    fn chain_error<F, U>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>;

    /// Map errors to a different error type while preserving success values
    ///
    /// This functional combinator allows transformation of error types
    /// while leaving successful results unchanged.
    fn map_error<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(Error) -> Error;

    /// Apply a side effect function to the error without changing the Result
    ///
    /// This is useful for logging or other side effects during error handling
    /// while maintaining the functional chain.
    fn inspect_error<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(&Error);
}

impl<T> ResultExt<T> for Result<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| e.map_context(|msg| format!("{}: {}", f(), msg)))
    }

    fn chain_error<F, U>(self, f: F) -> Result<U>
    where
        F: FnOnce(T) -> Result<U>,
    {
        self.and_then(f)
    }

    fn map_error<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(Error) -> Error,
    {
        self.map_err(f)
    }

    fn inspect_error<F>(self, f: F) -> Result<T>
    where
        F: FnOnce(&Error),
    {
        if let Err(ref e) = self {
            f(e);
        }
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::ConfigError;

    /// **What is tested:** Error display formatting for different error variants
    /// **Why it is tested:** Ensures that error messages are properly formatted and contain expected content for user-facing error reporting
    /// **Test conditions:** Creates different error types (IO, Processing, Config) with specific messages and error kinds
    /// **Expectations:** Each error's display format should contain the appropriate prefix and original error message
    #[test]
    fn test_error_display() {
        let io_error = Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "file not found",
        ));
        assert!(format!("{io_error}").contains("IO error"));
        assert!(format!("{io_error}").contains("file not found"));

        let processing_error = Error::processing_error("custom message".to_string());
        assert!(format!("{processing_error}").contains("Processing error"));
        assert!(format!("{processing_error}").contains("custom message"));

        let config_error = Error::Config(ConfigError::IoError {
            source: "test error".to_string(),
        });
        assert!(format!("{config_error}").contains("Configuration error"));
    }

    /// **What is tested:** Conversion from std::io::Error to application Error type
    /// **Why it is tested:** Verifies that the From trait implementation correctly wraps IO errors in the application's error type
    /// **Test conditions:** Creates a std::io::Error with PermissionDenied kind and converts it using From trait
    /// **Expectations:** The resulting error should be wrapped in Error::Io variant
    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let error = Error::from(io_err);

        match error {
            Error::Io(_) => (),
            _ => panic!("Expected IO error"),
        }
    }

    /// **What is tested:** Conversion from ConfigError to application Error type
    /// **Why it is tested:** Ensures that configuration errors are properly wrapped in the main error type for unified error handling
    /// **Test conditions:** Creates a ConfigError::IoError and converts it using From trait
    /// **Expectations:** The resulting error should be wrapped in Error::Config variant
    #[test]
    fn test_error_from_config() {
        let config_err = ConfigError::IoError {
            source: "test".to_string(),
        };
        let error = Error::from(config_err);

        match error {
            Error::Config(_) => (),
            _ => panic!("Expected Config error"),
        }
    }

    /// **What is tested:** Creation of processing errors with custom messages
    /// **Why it is tested:** Validates that the processing_error helper function correctly creates Error::Processing variants
    /// **Test conditions:** Creates a processing error with a specific test message
    /// **Expectations:** The error should be of Processing variant and contain the exact message provided
    #[test]
    fn test_processing_error() {
        let error = Error::processing_error("test message".to_string());

        match error {
            Error::Processing(msg) => assert_eq!(msg, "test message"),
            _ => panic!("Expected Processing error"),
        }
    }

    /// **What is tested:** Error source chain functionality for nested error handling
    /// **Why it is tested:** Ensures that the std::error::Error::source() method works correctly for error chaining and debugging
    /// **Test conditions:** Creates errors with and without underlying sources (IO error with source, Processing error without)
    /// **Expectations:** IO errors should have a source, Processing errors should not have a source
    #[test]
    fn test_error_source() {
        use std::error::Error as StdError;

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "not found");
        let error = crate::Error::Io(io_err);

        assert!(StdError::source(&error).is_some());

        let processing_error = crate::Error::processing_error("test".to_string());
        assert!(StdError::source(&processing_error).is_none());
    }

    /// **What is tested:** Functional error chaining with the chain method
    /// **Why it is tested:** Validates that errors can be functionally composed and transformed
    /// **Test conditions:** Creates an error and chains it with a transformation function
    /// **Expectations:** The chain function should apply the transformation correctly
    #[test]
    fn test_error_chain() {
        let error = Error::processing_error("initial".to_string());
        let result = error
            .chain(|e| -> Result<String> { Err(Error::processing_error(format!("chained: {e}"))) });

        match result {
            Err(Error::Processing(msg)) => {
                assert!(msg.contains("chained: Processing error: initial"))
            }
            _ => panic!("Expected chained processing error"),
        }
    }

    /// **What is tested:** Functional error context mapping
    /// **Why it is tested:** Ensures that error messages can be functionally transformed
    /// **Test conditions:** Creates errors and maps their context using transformation functions
    /// **Expectations:** Context should be transformed while preserving error type
    #[test]
    fn test_error_map_context() {
        let error = Error::processing_error("original".to_string());
        let mapped = error.map_context(|msg| format!("mapped: {msg}"));

        match mapped {
            Error::Processing(msg) => assert_eq!(msg, "mapped: original"),
            _ => panic!("Expected processing error"),
        }

        // Test that non-message errors are preserved
        let io_error = Error::Io(std::io::Error::new(std::io::ErrorKind::NotFound, "test"));
        let mapped_io = io_error.map_context(|msg| format!("mapped: {msg}"));

        match mapped_io {
            Error::Io(_) => (), // Should remain unchanged
            _ => panic!("Expected IO error to remain unchanged"),
        }
    }

    /// **What is tested:** Functional error combination
    /// **Why it is tested:** Validates that multiple errors can be combined into a single error
    /// **Test conditions:** Creates multiple errors and combines them using combine_errors
    /// **Expectations:** Should create a single error with aggregated context
    #[test]
    fn test_error_combine_errors() {
        let errors = vec![
            Error::processing_error("first".to_string()),
            Error::processing_error("second".to_string()),
            Error::processing_error("third".to_string()),
        ];

        let combined = Error::combine_errors(errors);
        match combined {
            Error::Processing(msg) => {
                assert!(msg.contains("Multiple errors occurred"));
                assert!(msg.contains("Error 1: Processing error: first"));
                assert!(msg.contains("Error 2: Processing error: second"));
                assert!(msg.contains("Error 3: Processing error: third"));
            }
            _ => panic!("Expected combined processing error"),
        }

        // Test single error case
        let single_error = vec![Error::processing_error("single".to_string())];
        let result = Error::combine_errors(single_error);
        match result {
            Error::Processing(msg) => assert_eq!(msg, "single"),
            _ => panic!("Expected single processing error"),
        }

        // Test empty case
        let empty_result = Error::combine_errors(vec![]);
        match empty_result {
            Error::Processing(msg) => assert_eq!(msg, "No errors to combine"),
            _ => panic!("Expected empty processing error"),
        }
    }

    /// **What is tested:** Functional error collection from iterator
    /// **Why it is tested:** Validates that errors can be collected from Result iterators
    /// **Test conditions:** Creates iterators of Results and collects them
    /// **Expectations:** Should collect successful results or return first error
    #[test]
    fn test_error_collect_errors() {
        // Test successful collection
        let successful_results: Vec<Result<i32>> = vec![Ok(1), Ok(2), Ok(3)];
        let collected = Error::collect_errors(successful_results).unwrap();
        assert_eq!(collected, vec![1, 2, 3]);

        // Test error collection
        let mixed_results: Vec<Result<i32>> = vec![
            Ok(1),
            Err(Error::processing_error("error".to_string())),
            Ok(3),
        ];
        let result = Error::collect_errors(mixed_results);
        assert!(result.is_err());
    }

    /// **What is tested:** Finding first error in Result collection
    /// **Why it is tested:** Validates that the first error can be extracted from Results
    /// **Test conditions:** Creates collections with and without errors
    /// **Expectations:** Should return first error or None if no errors exist
    #[test]
    fn test_error_first_error() {
        // Test with no errors
        let successful_results: Vec<Result<i32>> = vec![Ok(1), Ok(2), Ok(3)];
        let first_error = Error::first_error(successful_results);
        assert!(first_error.is_none());

        // Test with errors
        let mixed_results: Vec<Result<i32>> = vec![
            Ok(1),
            Err(Error::processing_error("first_error".to_string())),
            Err(Error::processing_error("second_error".to_string())),
        ];
        let first_error = Error::first_error(mixed_results).unwrap();
        match first_error {
            Error::Processing(msg) => assert_eq!(msg, "first_error"),
            _ => panic!("Expected first processing error"),
        }
    }

    /// **What is tested:** ResultExt trait with_context functionality
    /// **Why it is tested:** Validates that context can be added to Results functionally
    /// **Test conditions:** Creates Results and adds context using with_context
    /// **Expectations:** Error messages should include the added context
    #[test]
    fn test_result_ext_with_context() {
        use super::ResultExt;

        let result: Result<i32> = Err(Error::processing_error("original".to_string()));
        let with_context = result.with_context(|| "additional context".to_string());

        match with_context {
            Err(Error::Processing(msg)) => {
                assert!(msg.contains("additional context"));
                assert!(msg.contains("original"));
            }
            _ => panic!("Expected processing error with context"),
        }

        // Test successful case
        let success: Result<i32> = Ok(42);
        let success_with_context = success.with_context(|| "context".to_string());
        assert_eq!(success_with_context.unwrap(), 42);
    }

    /// **What is tested:** ResultExt trait error mapping functionality
    /// **Why it is tested:** Validates that errors can be transformed while preserving success
    /// **Test conditions:** Creates Results and maps errors using map_error
    /// **Expectations:** Errors should be transformed, success values preserved
    #[test]
    fn test_result_ext_map_error() {
        use super::ResultExt;

        let result: Result<i32> = Err(Error::processing_error("original".to_string()));
        let mapped = result.map_error(|e| Error::processing_error(format!("mapped: {e}")));

        match mapped {
            Err(Error::Processing(msg)) => {
                assert!(msg.contains("mapped: Processing error: original"))
            }
            _ => panic!("Expected mapped processing error"),
        }

        // Test successful case
        let success: Result<i32> = Ok(42);
        let success_mapped = success.map_error(|e| Error::processing_error(format!("mapped: {e}")));
        assert_eq!(success_mapped.unwrap(), 42);
    }

    /// **What is tested:** ResultExt trait error inspection functionality
    /// **Why it is tested:** Validates that side effects can be applied to errors without changing Results
    /// **Test conditions:** Creates Results and inspects errors using inspect_error
    /// **Expectations:** Side effects should be applied, Results should remain unchanged
    #[test]
    fn test_result_ext_inspect_error() {
        use super::ResultExt;
        use std::sync::{Arc, Mutex};

        let inspected = Arc::new(Mutex::new(String::new()));
        let inspected_clone = Arc::clone(&inspected);

        let result: Result<i32> = Err(Error::processing_error("test".to_string()));
        let inspected_result = result.inspect_error(|e| {
            *inspected_clone.lock().unwrap() = format!("inspected: {e}");
        });

        // Result should be unchanged
        match inspected_result {
            Err(Error::Processing(msg)) => assert_eq!(msg, "test"),
            _ => panic!("Expected unchanged processing error"),
        }

        // Side effect should have occurred
        assert_eq!(
            *inspected.lock().unwrap(),
            "inspected: Processing error: test"
        );

        // Test successful case - no side effect
        let success_tracker = Arc::new(Mutex::new(String::new()));
        let success_tracker_clone = Arc::clone(&success_tracker);

        let success: Result<i32> = Ok(42);
        let success_result = success.inspect_error(|e| {
            *success_tracker_clone.lock().unwrap() = format!("should not happen: {e}");
        });

        assert_eq!(success_result.unwrap(), 42);
        assert_eq!(*success_tracker.lock().unwrap(), ""); // No side effect for success
    }
}
