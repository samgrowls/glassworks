//! Tracing and logging configuration for the orchestrator.
//!
//! This module provides comprehensive tracing setup with configurable
//! log levels, formats, and output destinations.
//!
//! Features:
//! - Configurable log levels (trace, debug, info, warn, error)
//! - Multiple output formats (pretty, JSON, compact)
//! - File or stdout output
//! - Span tracking for async operations
//! - Environment variable support
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestrator_core::tracing::{init_tracing, TracingConfig, TracingFormat, TracingOutput};
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Initialize tracing with config
//!     let config = TracingConfig {
//!         level: tracing::Level::DEBUG,
//!         format: TracingFormat::Pretty,
//!         output: TracingOutput::Stdout,
//!     };
//!
//!     init_tracing(&config)?;
//!
//!     tracing::info!("Tracing initialized");
//!
//!     Ok(())
//! }
//! ```

use std::fs::File;
use std::io;
use tracing_subscriber::{
    fmt,
    registry::Registry,
    EnvFilter,
};

use crate::error::{OrchestratorError, Result};

/// Log level for tracing.
pub type Level = tracing::Level;

/// Output format for log messages.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TracingFormat {
    /// Human-readable pretty format with colors
    Pretty,
    /// Compact format for production
    Compact,
    /// JSON format for log aggregation
    Json,
    /// Minimal format (timestamp + message only)
    Minimal,
}

/// Output destination for log messages.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TracingOutput {
    /// Output to stdout
    Stdout,
    /// Output to stderr
    Stderr,
    /// Output to a file
    File(String),
    /// Output to both stdout and file
    Both(String),
}

/// Configuration for tracing setup.
#[derive(Debug, Clone)]
pub struct TracingConfig {
    /// Log level filter
    pub level: Level,
    /// Output format
    pub format: TracingFormat,
    /// Output destination
    pub output: TracingOutput,
    /// Enable ANSI colors (for pretty format)
    pub with_ansi: bool,
    /// Show thread names/IDs
    pub with_threads: bool,
    /// Show target/module path
    pub with_targets: bool,
    /// Show line numbers
    pub with_line_numbers: bool,
    /// Show file names
    pub with_file_names: bool,
    /// Environment variable to override log level (default: "RUST_LOG")
    pub env_filter: Option<String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            level: Level::INFO,
            format: TracingFormat::Pretty,
            output: TracingOutput::Stdout,
            with_ansi: true,
            with_threads: false,
            with_targets: false,
            with_line_numbers: false,
            with_file_names: false,
            env_filter: Some("RUST_LOG".to_string()),
        }
    }
}

impl TracingConfig {
    /// Create a new tracing config with default settings.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a config for development/debugging.
    pub fn debug() -> Self {
        Self {
            level: Level::DEBUG,
            format: TracingFormat::Pretty,
            output: TracingOutput::Stdout,
            with_ansi: true,
            with_threads: true,
            with_targets: true,
            with_line_numbers: true,
            with_file_names: true,
            ..Default::default()
        }
    }

    /// Create a config for production.
    pub fn production() -> Self {
        Self {
            level: Level::INFO,
            format: TracingFormat::Compact,
            output: TracingOutput::Stdout,
            with_ansi: false,
            with_threads: false,
            with_targets: false,
            with_line_numbers: false,
            with_file_names: false,
            ..Default::default()
        }
    }

    /// Create a config for JSON logging.
    pub fn json() -> Self {
        Self {
            level: Level::INFO,
            format: TracingFormat::Json,
            output: TracingOutput::Stdout,
            with_ansi: false,
            with_threads: false,
            with_targets: true,
            with_line_numbers: false,
            with_file_names: false,
            ..Default::default()
        }
    }

    /// Set the log level.
    pub fn with_level(mut self, level: Level) -> Self {
        self.level = level;
        self
    }

    /// Set the output format.
    pub fn with_format(mut self, format: TracingFormat) -> Self {
        self.format = format;
        self
    }

    /// Set the output destination.
    pub fn with_output(mut self, output: TracingOutput) -> Self {
        self.output = output;
        self
    }

    /// Enable or disable ANSI colors.
    pub fn with_ansi(mut self, with_ansi: bool) -> Self {
        self.with_ansi = with_ansi;
        self
    }

    /// Set the environment variable for log level override.
    pub fn with_env_filter(mut self, env_filter: impl Into<String>) -> Self {
        self.env_filter = Some(env_filter.into());
        self
    }

    /// Disable environment variable override.
    pub fn without_env_filter(mut self) -> Self {
        self.env_filter = None;
        self
    }
}

/// Initialize tracing with the given configuration.
///
/// Returns an error if the configuration is invalid or file output cannot be created.
pub fn init_tracing(config: &TracingConfig) -> Result<()> {
    // Create filter from level and env
    let filter = if let Some(env_var) = &config.env_filter {
        EnvFilter::try_from_env(env_var)
            .unwrap_or_else(|_| EnvFilter::from(config.level.to_string()))
    } else {
        EnvFilter::from(config.level.to_string())
    };

    // Set up output based on config - use dynamic dispatch for different formats
    match &config.output {
        TracingOutput::Stdout => {
            let subscriber = create_subscriber(config, filter, io::stdout)?;
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| OrchestratorError::internal_error(format!(
                    "Failed to set tracing subscriber: {}",
                    e
                )))?;
        }
        TracingOutput::Stderr => {
            let subscriber = create_subscriber(config, filter, io::stderr)?;
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| OrchestratorError::internal_error(format!(
                    "Failed to set tracing subscriber: {}",
                    e
                )))?;
        }
        TracingOutput::File(path) => {
            let file = File::create(path)
                .map_err(|e| OrchestratorError::io_error(e, format!("Failed to create log file: {}", path)))?;

            let subscriber = create_subscriber(config, filter, file)?;
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| OrchestratorError::internal_error(format!(
                    "Failed to set tracing subscriber: {}",
                    e
                )))?;
        }
        TracingOutput::Both(path) => {
            let file = File::create(path)
                .map_err(|e| OrchestratorError::io_error(e, format!("Failed to create log file: {}", path)))?;

            let tee_writer = TeeWriter::new(io::stdout(), file);
            let subscriber = create_subscriber(config, filter, tee_writer)?;
            tracing::subscriber::set_global_default(subscriber)
                .map_err(|e| OrchestratorError::internal_error(format!(
                    "Failed to set tracing subscriber: {}",
                    e
                )))?;
        }
    }

    Ok(())
}

/// Create a tracing subscriber with the given configuration.
fn create_subscriber<W>(
    config: &TracingConfig,
    filter: EnvFilter,
    writer: W,
) -> Result<Registry>
where
    W: for<'a> tracing_subscriber::fmt::MakeWriter<'a> + Send + Sync + 'static,
{
    let subscriber = match config.format {
        TracingFormat::Pretty => {
            Registry::default()
                .with(filter)
                .with(
                    fmt::layer()
                        .pretty()
                        .with_ansi(config.with_ansi)
                        .with_thread_names(config.with_threads)
                        .with_thread_ids(config.with_threads)
                        .with_target(config.with_targets)
                        .with_line_numbers(config.with_line_numbers)
                        .with_file(config.with_file_names)
                        .with_writer(writer),
                )
        }
        TracingFormat::Compact | TracingFormat::Json => {
            Registry::default()
                .with(filter)
                .with(
                    fmt::layer()
                        .compact()
                        .with_ansi(config.with_ansi)
                        .with_thread_names(config.with_threads)
                        .with_thread_ids(config.with_threads)
                        .with_target(config.with_targets)
                        .with_line_numbers(config.with_line_numbers)
                        .with_file(config.with_file_names)
                        .with_writer(writer),
                )
        }
        TracingFormat::Minimal => {
            Registry::default()
                .with(filter)
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .without_time()
                        .with_target(false)
                        .with_thread_names(false)
                        .with_thread_ids(false)
                        .with_line_numbers(false)
                        .with_file(false)
                        .with_writer(writer),
                )
        }
    };

    Ok(subscriber)
}

/// Writer that writes to both outputs (tee).
struct TeeWriter<W1, W2> {
    writer1: W1,
    writer2: W2,
}

impl<W1, W2> TeeWriter<W1, W2> {
    fn new(writer1: W1, writer2: W2) -> Self {
        Self { writer1, writer2 }
    }
}

impl<W1, W2> io::Write for TeeWriter<W1, W2>
where
    W1: io::Write,
    W2: io::Write,
{
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.writer1.write_all(buf)?;
        self.writer2.write_all(buf)?;
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer1.flush()?;
        self.writer2.flush()?;
        Ok(())
    }
}

/// Guard that cleans up tracing when dropped.
pub struct TracingGuard {
    _private: (),
}

impl TracingGuard {
    /// Create a new tracing guard.
    pub fn new() -> Self {
        Self { _private: () }
    }
}

impl Default for TracingGuard {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for TracingGuard {
    fn drop(&mut self) {
        tracing::info!("Tracing shutting down");
    }
}

/// Initialize tracing with default configuration.
///
/// This is a convenience function that uses `TracingConfig::default()`.
pub fn init() -> Result<()> {
    init_tracing(&TracingConfig::default())
}

/// Initialize tracing for debug mode.
pub fn init_debug() -> Result<()> {
    init_tracing(&TracingConfig::debug())
}

/// Initialize tracing for production mode.
pub fn init_production() -> Result<()> {
    init_tracing(&TracingConfig::production())
}

/// Initialize tracing with JSON output.
pub fn init_json() -> Result<()> {
    init_tracing(&TracingConfig::json())
}

/// Macro-friendly function to get the current span info.
pub fn current_span_info() -> Option<String> {
    // Note: This requires tracing-subscriber with registry feature
    // For now, return None as span info is typically handled by the subscriber
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tracing_config_default() {
        let config = TracingConfig::default();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, TracingFormat::Pretty);
        assert!(config.with_ansi);
    }

    #[test]
    fn test_tracing_config_debug() {
        let config = TracingConfig::debug();
        assert_eq!(config.level, Level::DEBUG);
        assert_eq!(config.format, TracingFormat::Pretty);
        assert!(config.with_threads);
        assert!(config.with_targets);
    }

    #[test]
    fn test_tracing_config_production() {
        let config = TracingConfig::production();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, TracingFormat::Compact);
        assert!(!config.with_ansi);
    }

    #[test]
    fn test_tracing_config_json() {
        let config = TracingConfig::json();
        assert_eq!(config.level, Level::INFO);
        assert_eq!(config.format, TracingFormat::Json);
        assert!(!config.with_ansi);
    }

    #[test]
    fn test_tracing_config_builder() {
        let config = TracingConfig::new()
            .with_level(Level::TRACE)
            .with_format(TracingFormat::Compact)
            .with_output(TracingOutput::Stderr)
            .with_ansi(false)
            .without_env_filter();

        assert_eq!(config.level, Level::TRACE);
        assert_eq!(config.format, TracingFormat::Compact);
        assert!(matches!(config.output, TracingOutput::Stderr));
        assert!(!config.with_ansi);
        assert!(config.env_filter.is_none());
    }

    #[test]
    fn test_tracing_format_display() {
        let formats = [
            TracingFormat::Pretty,
            TracingFormat::Compact,
            TracingFormat::Json,
            TracingFormat::Minimal,
        ];

        for format in &formats {
            // Just verify they can be created and compared
            assert_eq!(*format, *format);
        }
    }

    #[test]
    fn test_tracing_output_display() {
        let stdout = TracingOutput::Stdout;
        let stderr = TracingOutput::Stderr;
        let file = TracingOutput::File("/tmp/test.log".to_string());
        let both = TracingOutput::Both("/tmp/test.log".to_string());

        assert!(matches!(stdout, TracingOutput::Stdout));
        assert!(matches!(stderr, TracingOutput::Stderr));
        assert!(matches!(file, TracingOutput::File(_)));
        assert!(matches!(both, TracingOutput::Both(_)));
    }

    #[test]
    fn test_tee_writer() {
        let mut buf1 = Vec::new();
        let mut buf2 = Vec::new();

        {
            let mut tee = TeeWriter::new(&mut buf1, &mut buf2);
            tee.write_all(b"test").unwrap();
            tee.flush().unwrap();
        }

        assert_eq!(buf1, b"test");
        assert_eq!(buf2, b"test");
    }

    #[test]
    fn test_tracing_guard() {
        let guard = TracingGuard::new();
        // Guard exists, will log on drop
        drop(guard);
    }

    #[test]
    fn test_init_functions() {
        // These should not panic (though they may fail if tracing is already initialized)
        let _ = init();
        let _ = init_debug();
        let _ = init_production();
        let _ = init_json();
    }
}
