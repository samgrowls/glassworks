//! LLM Analysis Layer
//!
//! This module provides LLM-based review of files flagged by static analysis.
//! It is gated behind the `llm` feature flag.

#[cfg(feature = "llm")]
pub mod config;

#[cfg(feature = "llm")]
pub mod analyzer;

#[cfg(feature = "llm")]
pub mod rate_limiter;

// Re-export main types
#[cfg(feature = "llm")]
pub use config::LlmConfig;

#[cfg(feature = "llm")]
pub use analyzer::{LlmError, LlmFileResult, LlmVerdict, OpenAiCompatibleAnalyzer};

#[cfg(feature = "llm")]
pub use rate_limiter::RateLimiter;
