//! Orchestrator Core Library
//!
//! A high-level orchestration library for coordinating security scans across npm packages
//! and GitHub repositories using glassware-core detection engine.
//!
//! ## Features
//!
//! - **Parallel Scanning**: Concurrent downloads and scans with configurable concurrency
//! - **SQLite Caching**: Persistent cache with 7-day TTL for scan results
//! - **Rate Limiting**: Configurable rate limits for npm and GitHub APIs
//! - **Retry Logic**: Exponential backoff with jitter for transient failures
//! - **Progress Tracking**: Real-time progress updates with callbacks
//! - **GitHub Integration**: Repository search and download
//! - **Checkpoint/Resume**: Save and resume long-running scans
//! - **Output Formatters**: JSON and SARIF 2.1.0 output
//! - **LLM Analysis**: OpenAI-compatible API integration (optional)
//!
//! ## Example Usage
//!
//! ```rust,no_run
//! use orchestrator_core::{Orchestrator, OrchestratorConfig};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create orchestrator with default config
//!     let orchestrator = Orchestrator::new().await?;
//!
//!     // Scan npm packages
//!     let packages = vec!["express".to_string(), "lodash".to_string()];
//!     let results = orchestrator.scan_npm_packages(&packages).await;
//!
//!     // Process results
//!     for result in results {
//!         match result {
//!             Ok(scan_result) => {
//!                 println!("Package: {}", scan_result.package_name);
//!                 println!("Threat score: {:.2}", scan_result.threat_score);
//!                 println!("Findings: {}", scan_result.findings.len());
//!             }
//!             Err(e) => eprintln!("Error: {}", e),
//!         }
//!     }
//!
//!     // Export results to SARIF
//!     let sarif = orchestrator.format_sarif(&results, true)?;
//!     std::fs::write("results.sarif", &sarif)?;
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Architecture
//!
//! The orchestrator coordinates multiple components:
//!
//! 1. **Downloader**: Fetches packages from npm registry or GitHub
//! 2. **Scanner**: Uses glassware-core to detect security issues
//! 3. **Cacher**: Stores results in SQLite for faster re-scans
//! 4. **GitHub Downloader**: Repository search and archive download
//! 5. **Progress Tracker**: Real-time progress with ETA
//! 6. **Checkpoint Manager**: Save/resume support
//! 7. **Formatters**: JSON and SARIF output
//! 8. **LLM Analyzer**: AI-powered analysis (optional)
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                         Orchestrator                             │
//! ├─────────────────────────────────────────────────────────────────┤
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │  Downloader  │  │   Scanner    │  │    Cacher    │          │
//! │  │              │  │              │  │              │          │
//! │  │ - npm API    │  │ - glassware  │  │ - SQLite     │          │
//! │  │ - GitHub API │  │ - L1/L2/L3   │  │ - 7-day TTL  │          │
//! │  │ - Rate limit │  │ - Parallel   │  │ - Stats      │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐          │
//! │  │   GitHub     │  │  Progress    │  │ Checkpoint   │          │
//! │  │  Downloader  │  │   Tracker    │  │   Manager    │          │
//! │  └──────────────┘  └──────────────┘  └──────────────┘          │
//! │  ┌──────────────┐  ┌──────────────┐                            │
//! │  │  Formatters  │  │    LLM       │                            │
//! │  │ - JSON       │  │  Analyzer    │                            │
//! │  │ - SARIF      │  │  (optional)  │                            │
//! │  └──────────────┘  └──────────────┘                            │
//! └─────────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Configuration
//!
//! ```rust
//! use orchestrator_core::{OrchestratorConfig, DownloaderConfig, ScannerConfig};
//! use orchestrator_core::retry::RetryConfigBuilder;
//!
//! let config = OrchestratorConfig {
//!     downloader: DownloaderConfig {
//!         max_retries: 5,
//!         npm_rate_limit: 5.0,  // 5 requests/sec
//!         github_rate_limit: 2.0,  // 2 requests/sec
//!         ..Default::default()
//!     },
//!     scanner: ScannerConfig {
//!         max_concurrent: 20,
//!         threat_threshold: 7.0,
//!         ..Default::default()
//!     },
//!     cache_ttl_days: 14,  // 2 weeks
//!     enable_checkpoint: true,
//!     checkpoint_interval: 10,  // Auto-save every 10 packages
//!     retry_config: RetryConfigBuilder::new()
//!         .max_retries(3)
//!         .build(),
//!     npm_rate_limit: 5.0,
//!     github_rate_limit: 2.0,
//!     ..Default::default()
//! };
//! ```
//!
//! ## Progress Tracking
//!
//! ```rust,no_run
//! use orchestrator_core::{Orchestrator, ScanProgress};
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let orchestrator = Orchestrator::new()
//!         .await?
//!         .with_progress_callback(|progress: ScanProgress| {
//!             println!(
//!                 "Progress: {:.1}% - {}",
//!                 progress.percentage(),
//!                 progress.status
//!             );
//!         });
//!
//!     // ... use orchestrator
//!     Ok(())
//! }
//! ```
//!
//! ## Phase 2 Features
//!
//! ### GitHub Repository Downloading
//!
//! ```rust,no_run
//! use orchestrator_core::Orchestrator;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let orchestrator = Orchestrator::new().await?;
//!     
//!     // Search repositories
//!     let repos = orchestrator.search_github_repos("rust security", 10).await?;
//!     
//!     // Download a repository
//!     let path = orchestrator.download_github_repository("owner/repo").await?;
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Checkpoint/Resume
//!
//! ```rust,no_run
//! use orchestrator_core::Orchestrator;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let mut orchestrator = Orchestrator::new().await?;
//!     
//!     // Load checkpoint
//!     if orchestrator.load_checkpoint("npm").await? {
//!         println!("Resuming from checkpoint...");
//!     }
//!     
//!     Ok(())
//! }
//! ```
//!
//! ### Output Formatters
//!
//! ```rust,no_run
//! use orchestrator_core::Orchestrator;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let orchestrator = Orchestrator::new().await?;
//!     
//!     // Scan packages
//!     let packages = vec!["express".to_string()];
//!     let results = orchestrator.scan_npm_packages(&packages).await;
//!     
//!     // Export to JSON
//!     let json = orchestrator.format_json(&results, true)?;
//!     
//!     // Export to SARIF
//!     let sarif = orchestrator.format_sarif(&results, true)?;
//!     std::fs::write("results.sarif", &sarif)?;
//!     
//!     Ok(())
//! }
//! ```

pub mod adversarial;
pub mod campaign;
pub mod cacher;
pub mod checkpoint;
pub mod cli;
pub mod cli_validator;
pub mod config;
pub mod downloader;
pub mod error;
pub mod formatters;
pub mod github;
pub mod llm;
pub mod orchestrator;
pub mod progress;
pub mod rate_limiter;
pub mod retry;
pub mod sampler;
pub mod scanner;
pub mod scan_registry;
pub mod scoring;
pub mod scoring_config;
pub mod package_context;
pub mod streaming;
pub mod tracing;
pub mod version_checkpoint;
pub mod version_scanner;

// Re-export main types for convenience
pub use adversarial::{
    AdversarialTester, AdversarialReport, MutationEngine, FuzzerEngine,
    MutationEngineConfig, FuzzerEngineConfig, MutationStrategy, FuzzStrategy,
    MutationTestResult, FuzzTestResult, RiskLevel,
};

pub use campaign::{
    CampaignStatus, CampaignState, CampaignStats, CampaignEvent, CampaignCommand,
    EventBus, StateManager, CommandChannel, CommandSender,
    WaveState, WaveMode, WaveStatus, Priority, CaseId, PackageStage,
};

pub use cacher::{CacheEntry, CacheStats, Cacher};

pub use checkpoint::{Checkpoint, CheckpointManager, ScanResult as CheckpointScanResult};

pub use config::GlasswareConfig;

pub use downloader::{
    Downloader, DownloaderConfig, DownloadedPackage, GitHubRepoInfo, NpmDist, NpmPackageInfo,
    NpmRepository, PackageSpec,
};

pub use error::{OrchestratorError, Result};

pub use formatters::{JsonFormatter, OutputFormatter, SarifFormatter};

pub use github::{GitHubDownloader, GitHubDownloaderConfig};

#[cfg(feature = "llm")]
pub use llm::{LlmAnalyzer, LlmAnalyzerConfig, LlmVerdict};

pub use orchestrator::{Orchestrator, OrchestratorConfig, ScanProgress};

pub use progress::{ProgressStats, ProgressTracker, ProgressTrackerBuilder};

pub use rate_limiter::{MultiThrottleLimiter, ThrottleLimiter};

pub use retry::{RetryConfig, RetryConfigBuilder, RetryState, RetryableError, with_retry};

pub use scanner::{PackageScanResult, Scanner, ScannerConfig, ScanSummary};

pub use scoring::ScoringEngine;

pub use scoring_config::ScoringConfig;

pub use package_context::PackageContext;

pub use streaming::{StreamingWriter, StreamingWriterBuilder, OutputFormat as StreamOutputFormat};

pub use tracing::{
    TracingConfig, TracingFormat, TracingOutput,
    init_tracing, init, init_debug, init_production, init_json, TracingGuard,
};

/// Library version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version() {
        assert!(!VERSION.is_empty());
    }

    #[tokio::test]
    async fn test_orchestrator_creation() {
        let orchestrator = Orchestrator::new().await;
        assert!(orchestrator.is_ok());
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = Scanner::new();
        assert!(scanner.config().max_concurrent > 0);
    }

    #[tokio::test]
    async fn test_cacher_creation() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.db");
        let cacher = Cacher::with_path(&db_path).await;
        assert!(cacher.is_ok());
    }

    #[test]
    fn test_github_downloader_creation() {
        let downloader = GitHubDownloader::new();
        assert!(downloader.is_ok());
    }

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(100);
        assert_eq!(tracker.get_stats().total, 100);
    }

    #[test]
    fn test_checkpoint_creation() {
        use tempfile::TempDir;
        let temp_dir = TempDir::new().unwrap();
        let _manager = CheckpointManager::new(temp_dir.path());
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = ThrottleLimiter::new_per_second(10.0);
        assert!(limiter.check());
    }

    #[test]
    fn test_retry_config_creation() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
    }

    #[test]
    fn test_json_formatter_creation() {
        let formatter = JsonFormatter::new();
        assert!(formatter.pretty);
    }

    #[test]
    fn test_sarif_formatter_creation() {
        let formatter = SarifFormatter::new();
        assert!(formatter.pretty);
    }

    #[cfg(feature = "llm")]
    #[test]
    fn test_llm_analyzer_config() {
        let config = LlmAnalyzerConfig::default();
        assert!(!config.base_url.is_empty());
    }
}
