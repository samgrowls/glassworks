//! Enhanced error types for the orchestrator with better context and categorization.
//!
//! This module defines a comprehensive set of error types used throughout
//! the orchestrator for handling various failure scenarios.
//!
//! Features:
//! - Better error messages with context
//! - Error categorization (retryable vs fatal)
//! - Error recovery suggestions
//! - Source chain tracking
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestrator_core::error::{OrchestratorError, ErrorCategory, Result};
//!
//! fn process_package(name: &str) -> Result<()> {
//!     // Operation that might fail
//!     if name.is_empty() {
//!         return Err(OrchestratorError::validation_error(
//!             "Package name cannot be empty",
//!             Some("package_name"),
//!         ));
//!     }
//!
//!     Ok(())
//! }
//!
//! // Handle errors with context
//! match process_package("") {
//!     Err(e) => {
//!         eprintln!("Error category: {:?}", e.category());
//!         eprintln!("Is retryable: {}", e.is_retryable());
//!         eprintln!("Recovery suggestion: {}", e.recovery_suggestion());
//!     }
//!     Ok(_) => println!("Success!"),
//! }
//! ```

use std::fmt;
use thiserror::Error;

/// Error category for classification and handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCategory {
    /// Network-related errors (HTTP, DNS, connection)
    Network,
    /// File system errors (read, write, permissions)
    FileSystem,
    /// Database errors (SQLite, queries)
    Database,
    /// Parsing/serialization errors (JSON, YAML)
    Parsing,
    /// Validation errors (invalid input)
    Validation,
    /// Authentication/authorization errors
    Authentication,
    /// Rate limiting errors
    RateLimit,
    /// Timeout errors
    Timeout,
    /// Resource not found
    NotFound,
    /// Internal/unknown errors
    Internal,
    /// Cancellation/interruption
    Cancelled,
    /// Configuration errors
    Configuration,
    /// Scanner/detection errors
    Scanner,
    /// Cache errors
    Cache,
}

impl ErrorCategory {
    /// Check if this error category is retryable.
    pub fn is_retryable(self) -> bool {
        matches!(
            self,
            ErrorCategory::Network
                | ErrorCategory::Timeout
                | ErrorCategory::RateLimit
                | ErrorCategory::Database
        )
    }

    /// Check if this error category is fatal (should not be retried).
    pub fn is_fatal(self) -> bool {
        matches!(
            self,
            ErrorCategory::Validation
                | ErrorCategory::Authentication
                | ErrorCategory::NotFound
                | ErrorCategory::Parsing
                | ErrorCategory::Configuration
        )
    }

    /// Get a human-readable description of the category.
    pub fn description(self) -> &'static str {
        match self {
            ErrorCategory::Network => "Network connectivity issue",
            ErrorCategory::FileSystem => "File system operation failed",
            ErrorCategory::Database => "Database operation failed",
            ErrorCategory::Parsing => "Failed to parse data",
            ErrorCategory::Validation => "Invalid input or configuration",
            ErrorCategory::Authentication => "Authentication failed",
            ErrorCategory::RateLimit => "Rate limit exceeded",
            ErrorCategory::Timeout => "Operation timed out",
            ErrorCategory::NotFound => "Resource not found",
            ErrorCategory::Internal => "Internal error occurred",
            ErrorCategory::Cancelled => "Operation was cancelled",
            ErrorCategory::Configuration => "Configuration error",
            ErrorCategory::Scanner => "Scanner operation failed",
            ErrorCategory::Cache => "Cache operation failed",
        }
    }
}

/// Error context for additional information.
#[derive(Debug, Clone, Default)]
pub struct ErrorContext {
    /// Package or resource name involved
    pub package: Option<String>,
    /// File path involved
    pub path: Option<String>,
    /// Operation being performed
    pub operation: Option<String>,
    /// Additional key-value context
    pub extra: Vec<(String, String)>,
}

impl ErrorContext {
    /// Create a new error context.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the package name.
    pub fn with_package(mut self, package: impl Into<String>) -> Self {
        self.package = Some(package.into());
        self
    }

    /// Set the file path.
    pub fn with_path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }

    /// Set the operation.
    pub fn with_operation(mut self, operation: impl Into<String>) -> Self {
        self.operation = Some(operation.into());
        self
    }

    /// Add extra context.
    pub fn with_extra(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.extra.push((key.into(), value.into()));
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut parts = Vec::new();

        if let Some(ref package) = self.package {
            parts.push(format!("package: {}", package));
        }

        if let Some(ref path) = self.path {
            parts.push(format!("path: {}", path));
        }

        if let Some(ref operation) = self.operation {
            parts.push(format!("operation: {}", operation));
        }

        for (key, value) in &self.extra {
            parts.push(format!("{}: {}", key, value));
        }

        if parts.is_empty() {
            Ok(())
        } else {
            write!(f, " [{}]", parts.join(", "))
        }
    }
}

/// Main error type for orchestrator operations with enhanced context.
#[derive(Error, Debug)]
pub enum OrchestratorError {
    /// Error during HTTP requests or network operations.
    #[error("HTTP request failed: {message}")]
    Http {
        /// Underlying reqwest error
        #[source]
        source: reqwest::Error,
        /// Additional context message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during database operations.
    #[error("Database error: {message}")]
    Database {
        /// Underlying SQLx error
        #[source]
        source: sqlx::Error,
        /// Additional context message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during JSON serialization/deserialization.
    #[error("JSON error: {message}")]
    Json {
        /// Underlying serde_json error
        #[source]
        source: serde_json::Error,
        /// Additional context message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error from glassware-core operations.
    #[error("Scanner error: {message}")]
    Scanner {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when a package or repository is not found.
    #[error("Package not found: {package}")]
    NotFound {
        /// Package name or identifier
        package: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when rate limit is exceeded.
    #[error("Rate limit exceeded. Retry after {retry_after} seconds")]
    RateLimitExceeded {
        /// Seconds to wait before retrying
        retry_after: u64,
        /// Error context
        context: ErrorContext,
    },

    /// Error during file system operations.
    #[error("IO error: {message}")]
    Io {
        /// Underlying IO error
        #[source]
        source: std::io::Error,
        /// Additional context message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when maximum retries are exceeded.
    #[error("Maximum retries exceeded for operation: {operation}")]
    MaxRetriesExceeded {
        /// Operation that failed
        operation: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during package download.
    #[error("Download failed for {package}: {message}")]
    DownloadFailed {
        /// Package name or identifier
        package: String,
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during scanning operation.
    #[error("Scan failed for {path}: {message}")]
    ScanFailed {
        /// Path to the file or directory
        path: String,
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when cache operations fail.
    #[error("Cache error: {message}")]
    Cache {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for invalid configuration.
    #[error("Configuration error: {message}")]
    Config {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when operation is cancelled or interrupted.
    #[error("Operation cancelled: {reason}")]
    Cancelled {
        /// Reason for cancellation
        reason: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for invalid package name or identifier.
    #[error("Invalid package name: {package}")]
    InvalidPackageName {
        /// Invalid package name
        package: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for unsupported source type.
    #[error("Unsupported source type: {source}")]
    UnsupportedSource {
        /// Source type
        source: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during GitHub API operations.
    #[error("GitHub API error: {message}")]
    GitHub {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error during npm API operations.
    #[error("npm API error: {message}")]
    Npm {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error when a timeout occurs.
    #[error("Operation timed out: {operation}")]
    Timeout {
        /// Operation that timed out
        operation: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for invalid UTF-8 sequences.
    #[error("Invalid UTF-8: {message}")]
    Utf8 {
        /// Underlying UTF-8 error
        #[source]
        source: std::string::FromUtf8Error,
        /// Additional context message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for invalid path.
    #[error("Invalid path: {path}")]
    InvalidPath {
        /// Invalid path
        path: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for validation failures.
    #[error("Validation error: {message}")]
    Validation {
        /// Error message
        message: String,
        /// Field that failed validation
        field: Option<String>,
        /// Error context
        context: ErrorContext,
    },

    /// Error for authentication failures.
    #[error("Authentication failed: {message}")]
    Authentication {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },

    /// Error for internal/unexpected failures.
    #[error("Internal error: {message}")]
    Internal {
        /// Error message
        message: String,
        /// Error context
        context: ErrorContext,
    },
}

impl OrchestratorError {
    /// Get the error category.
    pub fn category(&self) -> ErrorCategory {
        match self {
            OrchestratorError::Http { .. } => ErrorCategory::Network,
            OrchestratorError::Database { .. } => ErrorCategory::Database,
            OrchestratorError::Json { .. } => ErrorCategory::Parsing,
            OrchestratorError::Scanner { .. } => ErrorCategory::Scanner,
            OrchestratorError::NotFound { .. } => ErrorCategory::NotFound,
            OrchestratorError::RateLimitExceeded { .. } => ErrorCategory::RateLimit,
            OrchestratorError::Io { .. } => ErrorCategory::FileSystem,
            OrchestratorError::MaxRetriesExceeded { .. } => ErrorCategory::Internal,
            OrchestratorError::DownloadFailed { .. } => ErrorCategory::Network,
            OrchestratorError::ScanFailed { .. } => ErrorCategory::Scanner,
            OrchestratorError::Cache { .. } => ErrorCategory::Cache,
            OrchestratorError::Config { .. } => ErrorCategory::Configuration,
            OrchestratorError::Cancelled { .. } => ErrorCategory::Cancelled,
            OrchestratorError::InvalidPackageName { .. } => ErrorCategory::Validation,
            OrchestratorError::UnsupportedSource { .. } => ErrorCategory::Validation,
            OrchestratorError::GitHub { .. } => ErrorCategory::Network,
            OrchestratorError::Npm { .. } => ErrorCategory::Network,
            OrchestratorError::Timeout { .. } => ErrorCategory::Timeout,
            OrchestratorError::Utf8 { .. } => ErrorCategory::Parsing,
            OrchestratorError::InvalidPath { .. } => ErrorCategory::Validation,
            OrchestratorError::Validation { .. } => ErrorCategory::Validation,
            OrchestratorError::Authentication { .. } => ErrorCategory::Authentication,
            OrchestratorError::Internal { .. } => ErrorCategory::Internal,
        }
    }

    /// Check if this error is retryable.
    pub fn is_retryable(&self) -> bool {
        self.category().is_retryable()
    }

    /// Check if this error is fatal (should not be retried).
    pub fn is_fatal(&self) -> bool {
        self.category().is_fatal()
    }

    /// Get a recovery suggestion for this error.
    pub fn recovery_suggestion(&self) -> &'static str {
        match self {
            OrchestratorError::Http { .. } => {
                "Check your network connection and try again"
            }
            OrchestratorError::Database { .. } => {
                "Verify database file is accessible and not corrupted"
            }
            OrchestratorError::Json { .. } => {
                "Check the input data format"
            }
            OrchestratorError::Scanner { .. } => {
                "Verify the file is readable and not corrupted"
            }
            OrchestratorError::NotFound { .. } => {
                "Verify the package name or repository exists"
            }
            OrchestratorError::RateLimitExceeded { .. } => {
                "Wait before retrying or reduce request frequency"
            }
            OrchestratorError::Io { .. } => {
                "Check file permissions and disk space"
            }
            OrchestratorError::MaxRetriesExceeded { .. } => {
                "The operation failed after multiple attempts. Check logs for details"
            }
            OrchestratorError::DownloadFailed { .. } => {
                "Check network connectivity and package availability"
            }
            OrchestratorError::ScanFailed { .. } => {
                "Verify the file is a valid source code file"
            }
            OrchestratorError::Cache { .. } => {
                "Try clearing the cache and retrying"
            }
            OrchestratorError::Config { .. } => {
                "Review your configuration settings"
            }
            OrchestratorError::Cancelled { .. } => {
                "Restart the operation"
            }
            OrchestratorError::InvalidPackageName { .. } => {
                "Use a valid package name format (e.g., 'package' or 'owner/repo')"
            }
            OrchestratorError::UnsupportedSource { .. } => {
                "Use a supported source type (npm or github)"
            }
            OrchestratorError::GitHub { .. } => {
                "Verify GitHub token is valid and has required permissions"
            }
            OrchestratorError::Npm { .. } => {
                "Verify npm registry is accessible"
            }
            OrchestratorError::Timeout { .. } => {
                "Increase timeout or check network latency"
            }
            OrchestratorError::Utf8 { .. } => {
                "Ensure the file is valid UTF-8 encoded text"
            }
            OrchestratorError::InvalidPath { .. } => {
                "Verify the path exists and is accessible"
            }
            OrchestratorError::Validation { .. } => {
                "Review input validation requirements"
            }
            OrchestratorError::Authentication { .. } => {
                "Verify credentials are correct and have required permissions"
            }
            OrchestratorError::Internal { .. } => {
                "This is an internal error. Please report it with the full error message"
            }
        }
    }

    /// Get the error context.
    pub fn context(&self) -> &ErrorContext {
        match self {
            OrchestratorError::Http { context, .. } => context,
            OrchestratorError::Database { context, .. } => context,
            OrchestratorError::Json { context, .. } => context,
            OrchestratorError::Scanner { context, .. } => context,
            OrchestratorError::NotFound { context, .. } => context,
            OrchestratorError::RateLimitExceeded { context, .. } => context,
            OrchestratorError::Io { context, .. } => context,
            OrchestratorError::MaxRetriesExceeded { context, .. } => context,
            OrchestratorError::DownloadFailed { context, .. } => context,
            OrchestratorError::ScanFailed { context, .. } => context,
            OrchestratorError::Cache { context, .. } => context,
            OrchestratorError::Config { context, .. } => context,
            OrchestratorError::Cancelled { context, .. } => context,
            OrchestratorError::InvalidPackageName { context, .. } => context,
            OrchestratorError::UnsupportedSource { context, .. } => context,
            OrchestratorError::GitHub { context, .. } => context,
            OrchestratorError::Npm { context, .. } => context,
            OrchestratorError::Timeout { context, .. } => context,
            OrchestratorError::Utf8 { context, .. } => context,
            OrchestratorError::InvalidPath { context, .. } => context,
            OrchestratorError::Validation { context, .. } => context,
            OrchestratorError::Authentication { context, .. } => context,
            OrchestratorError::Internal { context, .. } => context,
        }
    }

    /// Add context to this error.
    pub fn with_context(mut self, context: ErrorContext) -> Self {
        match &mut self {
            OrchestratorError::Http { context: c, .. } => *c = context,
            OrchestratorError::Database { context: c, .. } => *c = context,
            OrchestratorError::Json { context: c, .. } => *c = context,
            OrchestratorError::Scanner { context: c, .. } => *c = context,
            OrchestratorError::NotFound { context: c, .. } => *c = context,
            OrchestratorError::RateLimitExceeded { context: c, .. } => *c = context,
            OrchestratorError::Io { context: c, .. } => *c = context,
            OrchestratorError::MaxRetriesExceeded { context: c, .. } => *c = context,
            OrchestratorError::DownloadFailed { context: c, .. } => *c = context,
            OrchestratorError::ScanFailed { context: c, .. } => *c = context,
            OrchestratorError::Cache { context: c, .. } => *c = context,
            OrchestratorError::Config { context: c, .. } => *c = context,
            OrchestratorError::Cancelled { context: c, .. } => *c = context,
            OrchestratorError::InvalidPackageName { context: c, .. } => *c = context,
            OrchestratorError::UnsupportedSource { context: c, .. } => *c = context,
            OrchestratorError::GitHub { context: c, .. } => *c = context,
            OrchestratorError::Npm { context: c, .. } => *c = context,
            OrchestratorError::Timeout { context: c, .. } => *c = context,
            OrchestratorError::Utf8 { context: c, .. } => *c = context,
            OrchestratorError::InvalidPath { context: c, .. } => *c = context,
            OrchestratorError::Validation { context: c, .. } => *c = context,
            OrchestratorError::Authentication { context: c, .. } => *c = context,
            OrchestratorError::Internal { context: c, .. } => *c = context,
        }
        self
    }

    // Convenience constructors with context

    /// Create a validation error.
    pub fn validation_error(message: impl Into<String>, field: Option<&str>) -> Self {
        Self::Validation {
            message: message.into(),
            field: field.map(String::from),
            context: ErrorContext::new(),
        }
    }

    /// Create an HTTP error with context.
    pub fn http_error(source: reqwest::Error, message: impl Into<String>) -> Self {
        Self::Http {
            source,
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    /// Create a database error with context.
    pub fn database_error(source: sqlx::Error, message: impl Into<String>) -> Self {
        Self::Database {
            source,
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    /// Create a JSON error with context.
    pub fn json_error(source: serde_json::Error, message: impl Into<String>) -> Self {
        Self::Json {
            source,
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    /// Create an IO error with context.
    pub fn io_error(source: std::io::Error, message: impl Into<String>) -> Self {
        Self::Io {
            source,
            message: message.into(),
            context: ErrorContext::new(),
        }
    }

    /// Create an internal error with context.
    pub fn internal_error(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a cache error with context.
    pub fn cache_error(message: impl Into<String>) -> Self {
        Self::Cache {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a GitHub error with context.
    pub fn github_error(message: impl Into<String>) -> Self {
        Self::GitHub {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a config error with context.
    pub fn config_error(message: impl Into<String>) -> Self {
        Self::Config {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a download failed error.
    pub fn download_failed(package: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DownloadFailed {
            package: package.into(),
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a scan failed error.
    pub fn scan_failed(path: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ScanFailed {
            path: path.into(),
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a scanner error.
    pub fn scanner_error(message: impl Into<String>) -> Self {
        Self::Scanner {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a not found error.
    pub fn not_found(package: impl Into<String>) -> Self {
        Self::NotFound {
            package: package.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a rate limit exceeded error.
    pub fn rate_limit_exceeded(retry_after: u64) -> Self {
        Self::RateLimitExceeded {
            retry_after,
            context: ErrorContext::new(),
        }
    }
    
    /// Create a max retries exceeded error.
    pub fn max_retries_exceeded(operation: impl Into<String>) -> Self {
        Self::MaxRetriesExceeded {
            operation: operation.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a timeout error.
    pub fn timeout(message: impl Into<String>) -> Self {
        Self::Timeout {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create an authentication error.
    pub fn auth_error(message: impl Into<String>) -> Self {
        Self::Authentication {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a parsing error.
    pub fn parse_error(message: impl Into<String>) -> Self {
        Self::Parsing {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create a cancellation error.
    pub fn cancelled(message: impl Into<String>) -> Self {
        Self::Cancelled {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create an invalid package name error.
    pub fn invalid_package_name(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Self::InvalidPackageName {
            name: name.into(),
            reason: reason.into(),
            context: ErrorContext::new(),
        }
    }
    
    /// Create an npm error.
    pub fn npm_error(message: impl Into<String>) -> Self {
        Self::Npm {
            message: message.into(),
            context: ErrorContext::new(),
        }
    }
}

/// Result type alias for orchestrator operations.
pub type Result<T> = std::result::Result<T, OrchestratorError>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_category_retryable() {
        assert!(ErrorCategory::Network.is_retryable());
        assert!(ErrorCategory::Timeout.is_retryable());
        assert!(ErrorCategory::RateLimit.is_retryable());
        assert!(!ErrorCategory::Validation.is_retryable());
        assert!(!ErrorCategory::NotFound.is_retryable());
    }

    #[test]
    fn test_error_category_fatal() {
        assert!(ErrorCategory::Validation.is_fatal());
        assert!(ErrorCategory::Authentication.is_fatal());
        assert!(ErrorCategory::NotFound.is_fatal());
        assert!(!ErrorCategory::Network.is_fatal());
        assert!(!ErrorCategory::Timeout.is_fatal());
    }

    #[test]
    fn test_error_category_description() {
        assert_eq!(
            ErrorCategory::Network.description(),
            "Network connectivity issue"
        );
        assert_eq!(
            ErrorCategory::FileSystem.description(),
            "File system operation failed"
        );
    }

    #[test]
    fn test_error_context_display() {
        let context = ErrorContext::new()
            .with_package("test-pkg")
            .with_path("/path/to/file")
            .with_operation("scan");

        let display = format!("{}", context);
        assert!(display.contains("package: test-pkg"));
        assert!(display.contains("path: /path/to/file"));
        assert!(display.contains("operation: scan"));
    }

    #[test]
    fn test_error_category_detection() {
        let err = OrchestratorError::NotFound {
            package: "test".to_string(),
            context: ErrorContext::new(),
        };
        assert_eq!(err.category(), ErrorCategory::NotFound);
        assert!(err.is_fatal());
        assert!(!err.is_retryable());

        let err = OrchestratorError::Http {
            source: reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::ConnectionRefused,
                "connection refused"
            )),
            message: "test".to_string(),
            context: ErrorContext::new(),
        };
        assert_eq!(err.category(), ErrorCategory::Network);
        assert!(err.is_retryable());
    }

    #[test]
    fn test_recovery_suggestions() {
        let err = OrchestratorError::NotFound {
            package: "test".to_string(),
            context: ErrorContext::new(),
        };
        assert!(!err.recovery_suggestion().is_empty());

        let err = OrchestratorError::RateLimitExceeded {
            retry_after: 60,
            context: ErrorContext::new(),
        };
        assert!(err.recovery_suggestion().contains("Wait"));
    }

    #[test]
    fn test_validation_error_constructor() {
        let err = OrchestratorError::validation_error("Invalid input", Some("field_name"));
        assert_eq!(err.category(), ErrorCategory::Validation);
        assert!(err.is_fatal());
    }

    #[test]
    fn test_error_with_context() {
        let err = OrchestratorError::validation_error("test", None)
            .with_context(ErrorContext::new().with_package("pkg"));

        assert_eq!(err.context().package, Some("pkg".to_string()));
    }
}
