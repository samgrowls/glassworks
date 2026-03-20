//! Main orchestrator for coordinating downloads, scans, and caching.
//!
//! This module provides the high-level orchestrator that coordinates
//! package downloads, scanning, and result caching.
//!
//! Phase 2 Features:
//! - GitHub repository downloading
//! - Progress tracking with ETA
//! - Checkpoint/resume support
//! - JSON/SARIF output formatters
//! - LLM analysis integration
//! - Retry logic with exponential backoff
//! - Rate limiting

use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;
use tracing::{debug, error, info, warn};

use crate::cacher::{Cacher, CacheEntry};
use crate::checkpoint::{Checkpoint, CheckpointManager, ScanResult as CheckpointScanResult};
use crate::downloader::{Downloader, DownloaderConfig, DownloadedPackage, PackageSpec};
use crate::error::{OrchestratorError, Result};
use crate::formatters::{JsonFormatter, OutputFormatter, SarifFormatter};
use crate::github::GitHubDownloader;
#[cfg(feature = "llm")]
use crate::llm::{LlmAnalyzer, LlmAnalyzerConfig, LlmVerdict};
use crate::progress::{ProgressStats, ProgressTracker, ProgressTrackerBuilder};
use crate::rate_limiter::ThrottleLimiter;
use crate::retry::{RetryConfig, RetryConfigBuilder};
use crate::scanner::{Scanner, ScannerConfig, PackageScanResult, ScanSummary};

/// Scan progress information.
#[derive(Debug, Clone)]
pub struct ScanProgress {
    /// Total packages to scan.
    pub total: usize,
    /// Packages completed.
    pub completed: usize,
    /// Packages failed.
    pub failed: usize,
    /// Packages cached (skipped).
    pub cached: usize,
    /// Current status message.
    pub status: String,
}

impl ScanProgress {
    /// Create new progress tracker.
    pub fn new(total: usize) -> Self {
        Self {
            total,
            completed: 0,
            failed: 0,
            cached: 0,
            status: "Starting...".to_string(),
        }
    }

    /// Get completion percentage.
    pub fn percentage(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.completed + self.failed + self.cached) as f64 / self.total as f64 * 100.0
        }
    }
}

/// Progress callback type.
pub type ProgressCallback = Arc<dyn Fn(ScanProgress) + Send + Sync>;

/// Configuration for the orchestrator.
#[derive(Debug, Clone)]
pub struct OrchestratorConfig {
    /// Downloader configuration.
    pub downloader: DownloaderConfig,
    /// Scanner configuration.
    pub scanner: ScannerConfig,
    /// Database path for caching (None to disable).
    pub cache_db_path: Option<String>,
    /// Cache TTL in days (default: 7).
    pub cache_ttl_days: i64,
    /// Enable caching.
    pub enable_cache: bool,
    /// GitHub downloader configuration.
    pub github_token: Option<String>,
    /// Enable checkpoint/resume support.
    pub enable_checkpoint: bool,
    /// Checkpoint directory path.
    pub checkpoint_dir: Option<String>,
    /// Auto-save checkpoint interval (packages).
    pub checkpoint_interval: usize,
    /// Retry configuration.
    pub retry_config: RetryConfig,
    /// Rate limiter for npm.
    pub npm_rate_limit: f32,
    /// Rate limiter for GitHub.
    pub github_rate_limit: f32,
    /// Enable LLM analysis.
    #[cfg(feature = "llm")]
    pub enable_llm: bool,
    /// LLM analyzer configuration.
    #[cfg(feature = "llm")]
    pub llm_config: Option<LlmAnalyzerConfig>,
}

impl Default for OrchestratorConfig {
    fn default() -> Self {
        Self {
            downloader: DownloaderConfig::default(),
            scanner: ScannerConfig::default(),
            cache_db_path: Some(".glassware-orchestrator-cache.db".to_string()),
            cache_ttl_days: 7,
            enable_cache: true,
            github_token: None,
            enable_checkpoint: true,
            checkpoint_dir: Some(".glassware-checkpoints".to_string()),
            checkpoint_interval: 10,
            retry_config: RetryConfig::default(),
            npm_rate_limit: 2.0,
            github_rate_limit: 1.0,
            #[cfg(feature = "llm")]
            enable_llm: false,
            #[cfg(feature = "llm")]
            llm_config: None,
        }
    }
}

/// Main orchestrator for coordinating security scans.
pub struct Orchestrator {
    config: OrchestratorConfig,
    downloader: Downloader,
    scanner: Scanner,
    cacher: Option<Cacher>,
    github_downloader: Option<GitHubDownloader>,
    checkpoint_manager: Option<CheckpointManager>,
    npm_rate_limiter: ThrottleLimiter,
    github_rate_limiter: ThrottleLimiter,
    #[cfg(feature = "llm")]
    llm_analyzer: Option<LlmAnalyzer>,
    progress_tracker: Arc<ProgressTracker>,
    progress_callback: Option<ProgressCallback>,
}

impl Orchestrator {
    /// Create a new orchestrator with default configuration.
    pub async fn new() -> Result<Self> {
        Self::with_config(OrchestratorConfig::default()).await
    }

    /// Create a new orchestrator with custom configuration.
    pub async fn with_config(config: OrchestratorConfig) -> Result<Self> {
        let downloader = Downloader::with_config(config.downloader.clone())?;
        let scanner = Scanner::with_config(config.scanner.clone());

        // Initialize cache
        let cacher = if config.enable_cache {
            let default_cache_path = ".glassware-orchestrator-cache.db".to_string();
            let cache_path = config
                .cache_db_path
                .as_ref()
                .unwrap_or(&default_cache_path);

            Some(
                Cacher::with_path_and_ttl(cache_path, config.cache_ttl_days)
                    .await
                    .map_err(|e| {
                        warn!("Failed to initialize cache: {}. Caching disabled.", e);
                        e
                    })
                    .unwrap_or_else(|_| {
                        warn!("Cache initialization failed, continuing without cache");
                        panic!("Cache initialization failed")
                    }),
            )
        } else {
            None
        };

        // Initialize GitHub downloader
        let github_downloader = if let Some(ref token) = config.github_token {
            use crate::github::GitHubDownloaderConfig;
            let gh_config = GitHubDownloaderConfig {
                token: Some(token.clone()),
                rate_limit: config.github_rate_limit,
                ..Default::default()
            };
            Some(GitHubDownloader::with_config(gh_config)?)
        } else {
            None
        };

        // Initialize checkpoint manager
        let checkpoint_manager = if config.enable_checkpoint {
            let checkpoint_dir = config.checkpoint_dir
                .as_ref()
                .map(|s| PathBuf::from(s))
                .unwrap_or_else(|| PathBuf::from(".glassware-checkpoints"));
            
            let mut manager = CheckpointManager::new(&checkpoint_dir)?
                .with_auto_save_interval(config.checkpoint_interval);
            
            Some(manager)
        } else {
            None
        };

        // Initialize rate limiters
        let npm_rate_limiter = ThrottleLimiter::new_per_second(config.npm_rate_limit);
        let github_rate_limiter = ThrottleLimiter::new_per_second(config.github_rate_limit);

        // Initialize LLM analyzer
        #[cfg(feature = "llm")]
        let llm_analyzer = if config.enable_llm {
            if let Some(llm_config) = config.llm_config.clone() {
                Some(LlmAnalyzer::with_config(llm_config)?)
            } else {
                warn!("LLM enabled but no configuration provided");
                None
            }
        } else {
            None
        };

        // Initialize progress tracker
        let progress_tracker = Arc::new(ProgressTracker::new(0));

        Ok(Self {
            config,
            downloader,
            scanner,
            cacher,
            github_downloader,
            checkpoint_manager,
            npm_rate_limiter,
            github_rate_limiter,
            #[cfg(feature = "llm")]
            llm_analyzer,
            progress_tracker,
            progress_callback: None,
        })
    }

    /// Set a progress callback.
    pub fn with_progress_callback<F>(mut self, callback: F) -> Self
    where
        F: Fn(ScanProgress) + Send + Sync + 'static,
    {
        self.progress_callback = Some(Arc::new(callback));
        self
    }

    /// Get the progress tracker.
    pub fn progress_tracker(&self) -> &ProgressTracker {
        &self.progress_tracker
    }

    /// Update progress status.
    async fn update_progress(&self, status: String) {
        self.progress_tracker.set_status(status);
        
        if let Some(ref callback) = self.progress_callback {
            let stats = self.progress_tracker.get_stats();
            callback(ScanProgress {
                total: stats.total,
                completed: stats.current,
                failed: stats.errors,
                cached: 0, // Not tracked in new progress system
                status: stats.status,
            });
        }
    }

    /// Increment completed count.
    async fn increment_completed(&self) {
        self.progress_tracker.update();
        
        if let Some(ref callback) = self.progress_callback {
            let stats = self.progress_tracker.get_stats();
            callback(ScanProgress {
                total: stats.total,
                completed: stats.current,
                failed: stats.errors,
                cached: 0,
                status: stats.status,
            });
        }
    }

    /// Increment cached count (legacy support).
    async fn increment_cached(&self) {
        self.progress_tracker.update();
    }

    /// Format results to JSON.
    pub fn format_json(&self, results: &[PackageScanResult], pretty: bool) -> Result<String> {
        let formatter = JsonFormatter::new().with_pretty(pretty);
        formatter.format(results)
    }

    /// Format results to SARIF.
    pub fn format_sarif(&self, results: &[PackageScanResult], pretty: bool) -> Result<String> {
        let formatter = SarifFormatter::new().with_pretty(pretty);
        formatter.format(results)
    }

    /// Save results to a file.
    pub fn save_results(&self, results: &[PackageScanResult], path: &Path, format: &str) -> Result<()> {
        match format {
            "json" => {
                let formatter = JsonFormatter::new();
                formatter.format_to_file(results, path)
            }
            "sarif" => {
                let formatter = SarifFormatter::new();
                formatter.format_to_file(results, path)
            }
            _ => Err(OrchestratorError::config_error(format!("Unknown format: {}", format))),
        }
    }

    /// Analyze findings with LLM.
    #[cfg(feature = "llm")]
    pub async fn analyze_with_llm(&self, results: &[PackageScanResult]) -> Result<Vec<LlmVerdict>> {
        if let Some(ref analyzer) = self.llm_analyzer {
            let mut verdicts = Vec::new();
            
            for result in results {
                for finding in &result.findings {
                    let llm_finding = crate::llm::LlmFinding::from(finding);
                    let verdict = analyzer.analyze_finding(finding).await?;
                    verdicts.push(verdict);
                }
            }
            
            Ok(verdicts)
        } else {
            Err(OrchestratorError::config_error("LLM analyzer not configured".to_string()))
        }
    }

    /// Search GitHub repositories.
    pub async fn search_github_repos(&self, query: &str, max_results: usize) -> Result<Vec<String>> {
        if let Some(ref gh) = self.github_downloader {
            gh.search_repos(query, max_results).await
        } else {
            Err(OrchestratorError::github_error("GitHub downloader not configured (no token)".to_string()))
        }
    }

    /// Download a GitHub repository.
    pub async fn download_github_repository(&self, repo: &str) -> Result<PathBuf> {
        if let Some(ref gh) = self.github_downloader {
            gh.download_repo(repo).await
        } else {
            // Use the built-in downloader without token
            let parts: Vec<&str> = repo.split('/').collect();
            if parts.len() != 2 {
                return Err(OrchestratorError::invalid_package_name("package", format!("Invalid repo: {}", repo)));
            }
            
            self.downloader.download_github_repo(parts[0], parts[1], None).await
                .map(|pkg| PathBuf::from(pkg.path))
        }
    }

    /// Save a checkpoint.
    pub fn save_checkpoint(&self, source: &str) -> Result<()> {
        if let Some(ref manager) = self.checkpoint_manager {
            manager.save_checkpoint()
        } else {
            Err(OrchestratorError::config_error("Checkpoint manager not enabled".to_string()))
        }
    }

    /// Load a checkpoint.
    pub fn load_checkpoint(&mut self, source: &str) -> Result<bool> {
        if let Some(ref mut manager) = self.checkpoint_manager {
            manager.load_checkpoint(source)
        } else {
            Err(OrchestratorError::config_error("Checkpoint manager not enabled".to_string()))
        }
    }

    /// Get remaining packages from checkpoint.
    pub fn get_remaining_from_checkpoint(&self, source: &str) -> Result<Vec<String>> {
        if let Some(ref manager) = self.checkpoint_manager {
            let checkpoint_path = PathBuf::from(".glassware-checkpoints")
                .join(format!("{}.checkpoint", source.replace('/', "_")));
            
            if checkpoint_path.exists() {
                let checkpoint = Checkpoint::load(&checkpoint_path)?;
                Ok(checkpoint.remaining)
            } else {
                Err(OrchestratorError::not_found("Checkpoint not found".to_string()))
            }
        } else {
            Err(OrchestratorError::config_error("Checkpoint manager not enabled".to_string()))
        }
    }

    /// Scan npm packages.
    pub async fn scan_npm_packages(
        &self,
        packages: &[String],
    ) -> Vec<Result<PackageScanResult>> {
        info!("Scanning {} npm packages", packages.len());

        // Initialize progress tracker
        self.progress_tracker.set_status(format!("Starting scan of {} packages", packages.len()));
        
        let mut results = Vec::new();

        for (i, package) in packages.iter().enumerate() {
            self.update_progress(format!("Processing {}/{}: {}", i + 1, packages.len(), package))
                .await;

            match self.scan_npm_package(package).await {
                Ok(result) => results.push(Ok(result)),
                Err(e) => {
                    error!("Failed to scan package {}: {}", package, e);
                    results.push(Err(e));
                }
            }

            self.increment_completed().await;
        }

        results
    }

    /// Scan a single npm package.
    pub async fn scan_npm_package(&self, package: &str) -> Result<PackageScanResult> {
        // Check cache
        if let Some(ref cacher) = self.cacher {
            if let Some(entry) = cacher.get(package).await? {
                debug!("Cache hit for package: {}", package);
                self.increment_cached().await;

                // Parse cached result
                let cached_result: PackageScanResult =
                    serde_json::from_str(&entry.result).map_err(|e| {
                        OrchestratorError::cache_error(format!("Failed to parse cached result: {}", e))
                    })?;

                return Ok(cached_result);
            }
        }

        // Download package
        let downloaded = self.downloader.download_npm_package(package).await?;

        // Scan package
        let scan_result = self.scanner.scan_package(&downloaded).await?;

        // Cache result
        if let Some(ref cacher) = self.cacher {
            let result_json = serde_json::to_string(&scan_result).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize scan result: {}", e))
            })?;

            let entry = cacher.create_entry(
                package.to_string(),
                "npm".to_string(),
                result_json,
                Some(scan_result.content_hash.clone()),
            );

            cacher.set(entry).await?;
        }

        Ok(scan_result)
    }

    /// Scan GitHub repositories.
    pub async fn scan_github_repos(
        &self,
        repos: &[(String, String)], // (owner, repo)
        ref_name: Option<&str>,
    ) -> Vec<Result<PackageScanResult>> {
        info!("Scanning {} GitHub repositories", repos.len());

        // Initialize progress tracker
        self.progress_tracker.set_status(format!("Starting scan of {} repositories", repos.len()));

        let mut results = Vec::new();

        for (i, (owner, repo)) in repos.iter().enumerate() {
            let package_spec = format!("{}/{}", owner, repo);
            self.update_progress(format!(
                "Processing {}/{}: {}",
                i + 1,
                repos.len(),
                package_spec
            ))
            .await;

            match self.scan_github_repo(owner, repo, ref_name).await {
                Ok(result) => results.push(Ok(result)),
                Err(e) => {
                    error!("Failed to scan repository {}/{}: {}", owner, repo, e);
                    results.push(Err(e));
                }
            }

            self.increment_completed().await;
        }

        results
    }

    /// Scan a single GitHub repository.
    async fn scan_github_repo(
        &self,
        owner: &str,
        repo: &str,
        ref_name: Option<&str>,
    ) -> Result<PackageScanResult> {
        let package_key = format!("{}/{}", owner, repo);

        // Check cache
        if let Some(ref cacher) = self.cacher {
            if let Some(entry) = cacher.get(&package_key).await? {
                debug!("Cache hit for repository: {}", package_key);
                self.increment_cached().await;

                let cached_result: PackageScanResult =
                    serde_json::from_str(&entry.result).map_err(|e| {
                        OrchestratorError::cache_error(format!("Failed to parse cached result: {}", e))
                    })?;

                return Ok(cached_result);
            }
        }

        // Download repository
        let downloaded = self
            .downloader
            .download_github_repo(owner, repo, ref_name)
            .await?;

        // Scan repository
        let scan_result = self.scanner.scan_package(&downloaded).await?;

        // Cache result
        if let Some(ref cacher) = self.cacher {
            let result_json = serde_json::to_string(&scan_result).map_err(|e| {
                OrchestratorError::cache_error(format!("Failed to serialize scan result: {}", e))
            })?;

            let entry = cacher.create_entry(
                package_key,
                "github".to_string(),
                result_json,
                Some(scan_result.content_hash.clone()),
            );

            cacher.set(entry).await?;
        }

        Ok(scan_result)
    }

    /// Scan packages from a file list.
    pub async fn scan_file_list(&self, file_path: &str) -> Result<Vec<Result<PackageScanResult>>> {
        let path = Path::new(file_path);

        if !path.exists() {
            return Err(OrchestratorError::InvalidPath(format!(
                "File not found: {}",
                file_path
            )));
        }

        let content = tokio::fs::read_to_string(path)
            .await
            .map_err(|e| OrchestratorError::io_error(e))?;

        let packages: Vec<String> = content
            .lines()
            .filter(|line| !line.trim().is_empty() && !line.trim().starts_with('#'))
            .map(|s| s.trim().to_string())
            .collect();

        info!("Loaded {} packages from file: {}", packages.len(), file_path);

        let mut results = Vec::new();

        for package in &packages {
            let spec = Downloader::parse_package_spec(package)?;

            match spec {
                PackageSpec::Npm { name } => {
                    match self.scan_npm_package(&name).await {
                        Ok(result) => results.push(Ok(result)),
                        Err(e) => results.push(Err(e)),
                    }
                }
                PackageSpec::GitHub { owner, repo } => {
                    match self.scan_github_repo(&owner, &repo, None).await {
                        Ok(result) => results.push(Ok(result)),
                        Err(e) => results.push(Err(e)),
                    }
                }
            }
        }

        Ok(results)
    }

    /// Get the orchestrator configuration.
    pub fn config(&self) -> &OrchestratorConfig {
        &self.config
    }

    /// Get the scanner.
    pub fn scanner(&self) -> &Scanner {
        &self.scanner
    }

    /// Get the downloader.
    pub fn downloader(&self) -> &Downloader {
        &self.downloader
    }
}

impl Default for Orchestrator {
    fn default() -> Self {
        // Use tokio runtime to create default instance
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(Self::new()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_orchestrator_config_default() {
        let config = OrchestratorConfig::default();
        assert!(config.enable_cache);
        assert_eq!(config.cache_ttl_days, 7);
        assert!(config.cache_db_path.is_some());
        assert!(config.enable_checkpoint);
        assert_eq!(config.checkpoint_interval, 10);
        assert_eq!(config.npm_rate_limit, 2.0);
        assert_eq!(config.github_rate_limit, 1.0);
    }

    #[test]
    fn test_scan_progress() {
        let mut progress = ScanProgress::new(10);
        assert_eq!(progress.percentage(), 0.0);

        progress.completed = 5;
        assert_eq!(progress.percentage(), 50.0);

        progress.failed = 2;
        progress.cached = 3;
        assert_eq!(progress.percentage(), 100.0);
    }

    #[test]
    fn test_retry_config_builder() {
        let retry_config = RetryConfigBuilder::new()
            .max_retries(5)
            .base_delay(std::time::Duration::from_millis(100))
            .build();
        
        assert_eq!(retry_config.max_retries, 5);
    }

    #[test]
    fn test_rate_limiter_creation() {
        let limiter = ThrottleLimiter::new_per_second(10.0);
        assert!(limiter.check());
    }

    #[test]
    fn test_progress_tracker_creation() {
        let tracker = ProgressTracker::new(100);
        assert_eq!(tracker.get_stats().total, 100);
    }

    #[tokio::test]
    async fn test_json_formatter() {
        let formatter = JsonFormatter::new();
        let results: Vec<PackageScanResult> = vec![];
        let json = formatter.format(&results).unwrap();
        assert!(json.contains("total_packages"));
    }

    #[tokio::test]
    async fn test_sarif_formatter() {
        let formatter = SarifFormatter::new();
        let results: Vec<PackageScanResult> = vec![];
        let sarif = formatter.format(&results).unwrap();
        assert!(sarif.contains("\"version\": \"2.1.0\""));
    }
}
