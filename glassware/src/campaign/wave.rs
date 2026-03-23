//! Wave execution logic.
//!
//! This module handles the execution of individual waves, including:
//! - Package source resolution (npm search, categories, explicit lists)
//! - Progress tracking and event publishing
//! - Error handling and retry logic
//! - Integration with glassware-core scanner

use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use glassware_core::{ScanEngine, Finding};

use crate::campaign::config::{WaveConfig, WaveSource, CampaignSettings};
use crate::campaign::event_bus::{EventBus, CampaignEvent, EventBusExt};
use crate::campaign::state_manager::StateManager;
use crate::campaign::types::{WaveStatus, PackageStage, ActivePackage};
use crate::downloader::DownloadedPackage;

/// Wave executor for running individual waves.
pub struct WaveExecutor {
    config: WaveConfig,
    settings: CampaignSettings,
    state: StateManager,
    event_bus: EventBus,
    concurrency_semaphore: Arc<Semaphore>,
    scanner: crate::scanner::Scanner,
}

impl WaveExecutor {
    /// Create a new wave executor.
    pub fn new(
        config: WaveConfig,
        settings: CampaignSettings,
        state: StateManager,
        event_bus: EventBus,
        max_concurrency: usize,
    ) -> Self {
        // Convert campaign whitelist to glassware-core format
        let whitelist_config = glassware_core::config::WhitelistConfig {
            packages: settings.whitelist.packages.clone(),
            crypto_packages: settings.whitelist.crypto_packages.clone(),
            build_tools: settings.whitelist.build_tools.clone(),
            state_management: vec![],
        };

        // Convert campaign scoring to glassware-core format
        let scoring_config = glassware_core::config::ScoringConfig {
            malicious_threshold: settings.scoring.malicious_threshold,
            suspicious_threshold: settings.scoring.suspicious_threshold,
            category_weight: 2.0,
            critical_weight: 3.0,
            high_weight: 1.5,
        };

        let scanner = crate::scanner::Scanner::with_config(
            crate::scanner::ScannerConfig {
                max_concurrent: max_concurrency,
                glassware_config: glassware_core::GlasswareConfig {
                    whitelist: whitelist_config,
                    scoring: scoring_config,
                    ..Default::default()
                },
                ..Default::default()
            }
        );

        Self {
            config,
            settings,
            state,
            event_bus,
            concurrency_semaphore: Arc::new(Semaphore::new(max_concurrency)),
            scanner,
        }
    }

    /// Execute the wave.
    ///
    /// Returns the number of packages scanned, flagged, and malicious.
    pub async fn execute(&self) -> Result<WaveResult, WaveError> {
        let wave_id = self.config.id.clone();
        let wave_name = self.config.name.clone();

        info!("📦 Starting wave '{}' ({})", wave_name, wave_id);

        // Collect all packages from sources
        info!("Collecting packages for wave '{}'...", wave_name);
        let packages = self.collect_packages().await?;
        let total_packages = packages.len();
        info!("Collected {} packages for wave '{}'", total_packages, wave_name);

        // Update state: wave started
        self.state.start_wave(&wave_id, total_packages).await;

        // Execute packages
        let mut scanned = 0;
        let mut flagged = 0;
        let mut malicious = 0;

        for (index, package) in packages.iter().enumerate() {
            debug!("Processing {}/{}: {}", index + 1, total_packages, package);

            // Update active package
            self.state.set_active_package(ActivePackage::new(
                &package.name,
                &package.version,
                &wave_id,
                PackageStage::Scanning,
            )).await;

            // Execute package scan
            match self.scan_package(package).await {
                Ok(result) => {
                    scanned += 1;
                    if result.findings_count > 0 {
                        flagged += 1;
                    }
                    if result.is_malicious {
                        malicious += 1;
                    }

                    // Update progress
                    self.state.update_wave_progress(&wave_id, scanned, flagged, malicious).await;

                    // Publish progress event every 10 packages
                    if scanned % 10 == 0 || scanned == total_packages {
                        self.event_bus.publish(CampaignEvent::WaveProgress {
                            wave_id: wave_id.clone(),
                            scanned,
                            flagged,
                            malicious,
                        });
                    }
                }
                Err(e) => {
                    error!("Failed to scan package {}: {}", package, e);
                    // Continue with next package
                }
            }

            // Clear active package
            self.state.clear_active_package().await;
        }

        // Mark wave as completed
        self.state.complete_wave(&wave_id).await;

        info!(
            "✅ Wave '{}' completed: {} scanned, {} flagged, {} malicious",
            wave_name, scanned, flagged, malicious
        );

        Ok(WaveResult {
            wave_id,
            packages_scanned: scanned,
            packages_flagged: flagged,
            packages_malicious: malicious,
        })
    }

    /// Collect packages from all wave sources.
    async fn collect_packages(&self) -> Result<Vec<PackageSpec>, WaveError> {
        let mut packages = Vec::new();

        for source in &self.config.sources {
            debug!("Collecting packages from source: {}", source.source_type());

            let source_packages = self.collect_from_source(source).await?;
            packages.extend(source_packages);
        }

        // Apply whitelist filter
        if !self.config.whitelist.is_empty() {
            let whitelist = &self.config.whitelist;
            packages.retain(|pkg| {
                !whitelist.iter().any(|entry| {
                    entry.packages.iter().any(|pattern| {
                        pkg.name.contains(pattern) || pkg.name == *pattern
                    })
                })
            });
            debug!("After whitelist filter: {} packages", packages.len());
        }

        // Remove duplicates
        packages.sort_by(|a, b| a.name.cmp(&b.name).then(a.version.cmp(&b.version)));
        packages.dedup_by(|a, b| a.name == b.name && a.version == b.version);

        Ok(packages)
    }

    /// Collect packages from a specific source.
    async fn collect_from_source(&self, source: &WaveSource) -> Result<Vec<PackageSpec>, WaveError> {
        match source {
            WaveSource::Packages { list } => {
                // Parse explicit package list
                let mut packages = Vec::new();
                for spec in list {
                    let (name, version) = parse_package_spec(spec);
                    packages.push(PackageSpec {
                        name,
                        version: version.unwrap_or_else(|| "latest".to_string()),
                    });
                }
                Ok(packages)
            }

            WaveSource::NpmSearch { keywords, samples_per_keyword, days_recent, max_downloads } => {
                // Search npm registry
                let mut packages = Vec::new();
                
                for keyword in keywords {
                    debug!("Searching npm for keyword: {}", keyword);
                    
                    match self.search_npm(keyword, *samples_per_keyword).await {
                        Ok(results) => {
                            for pkg in results {
                                // Apply filters
                                if let Some(max_days) = days_recent {
                                    // TODO: Filter by publish date
                                }
                                if let Some(max_downloads) = max_downloads {
                                    // TODO: Filter by download count
                                }
                                packages.push(pkg);
                            }
                        }
                        Err(e) => {
                            warn!("Failed to search npm for '{}': {}", keyword, e);
                        }
                    }
                }
                
                Ok(packages)
            }

            WaveSource::NpmCategory { category, samples, sort_by } => {
                // Sample from npm category
                // TODO: Implement category sampling
                debug!("Sampling {} packages from category '{}'", samples, category);
                Ok(Vec::new()) // Placeholder
            }

            WaveSource::GitHubSearch { query, max_results, sort_by } => {
                // Search GitHub repositories
                // TODO: Implement GitHub search
                debug!("Searching GitHub for '{}'", query);
                Ok(Vec::new()) // Placeholder
            }
        }
    }

    /// Search npm registry for packages.
    async fn search_npm(&self, keyword: &str, limit: usize) -> Result<Vec<PackageSpec>, WaveError> {
        // TODO: Implement npm search API call
        // For now, return empty list
        debug!("npm search for '{}' (limit: {})", keyword, limit);
        Ok(Vec::new())
    }

    /// Scan a single package.
    async fn scan_package(&self, package: &PackageSpec) -> Result<ScanResult, WaveError> {
        info!("Scanning package: {}@{}", package.name, package.version);
        
        // Acquire concurrency permit
        let _permit = self.concurrency_semaphore
            .acquire()
            .await
            .map_err(|_| WaveError::ExecutorError("Semaphore closed".to_string()))?;

        info!("Acquired semaphore for {}@{}", package.name, package.version);

        // Download the package
        let downloaded = self.download_package(&package.name, &package.version).await?;
        
        // Scan the downloaded package
        let scan_result = self.scanner.scan_package(&downloaded).await
            .map_err(|e| WaveError::ExecutorError(format!("Scan failed: {}", e)))?;
        
        info!(
            "Package {}@{} scanned: {} findings, threat_score={:.2}, malicious={}",
            package.name, package.version,
            scan_result.findings.len(),
            scan_result.threat_score,
            scan_result.is_malicious
        );

        Ok(ScanResult {
            package: package.clone(),
            findings_count: scan_result.findings.len(),
            threat_score: scan_result.threat_score,
            is_malicious: scan_result.is_malicious,
        })
    }

    /// Download a package from npm.
    async fn download_package(&self, name: &str, version: &str) -> Result<DownloadedPackage, WaveError> {
        use crate::downloader::{Downloader, DownloaderConfig};
        
        let downloader = Downloader::with_config(DownloaderConfig::default())
            .map_err(|e| WaveError::CollectionError(format!("Downloader init failed: {}", e)))?;
        let package_spec = format!("{}@{}", name, version);
        
        downloader.download_npm_package(&package_spec).await
            .map_err(|e| WaveError::CollectionError(format!("Download failed: {}", e)))
    }
}

/// Package specification.
#[derive(Debug, Clone)]
pub struct PackageSpec {
    pub name: String,
    pub version: String,
}

impl std::fmt::Display for PackageSpec {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}@{}", self.name, self.version)
    }
}

/// Scan result for a single package.
#[derive(Debug, Clone)]
pub struct ScanResult {
    pub package: PackageSpec,
    pub findings_count: usize,
    pub threat_score: f32,
    pub is_malicious: bool,
}

/// Wave execution result.
#[derive(Debug, Clone)]
pub struct WaveResult {
    pub wave_id: String,
    pub packages_scanned: usize,
    pub packages_flagged: usize,
    pub packages_malicious: usize,
}

/// Wave execution errors.
#[derive(Debug, thiserror::Error)]
pub enum WaveError {
    #[error("Package collection error: {0}")]
    CollectionError(String),

    #[error("Executor error: {0}")]
    ExecutorError(String),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Parse package spec string (name@version or just name).
fn parse_package_spec(spec: &str) -> (String, Option<String>) {
    if let Some(at_pos) = spec.rfind('@') {
        if at_pos > 0 {
            let name = spec[..at_pos].to_string();
            let version = spec[at_pos + 1..].to_string();
            (name, Some(version))
        } else {
            // Starts with @, treat as scoped package without version
            (spec.to_string(), None)
        }
    } else {
        // No version specified
        (spec.to_string(), None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::campaign::types::WaveMode;

    #[test]
    fn test_parse_package_spec() {
        // name@version
        let (name, version) = parse_package_spec("express@4.19.2");
        assert_eq!(name, "express");
        assert_eq!(version, Some("4.19.2".to_string()));

        // Just name
        let (name, version) = parse_package_spec("express");
        assert_eq!(name, "express");
        assert_eq!(version, None);

        // Scoped package@version
        let (name, version) = parse_package_spec("@scope/package@1.0.0");
        assert_eq!(name, "@scope/package");
        assert_eq!(version, Some("1.0.0".to_string()));

        // Scoped package without version
        let (name, version) = parse_package_spec("@scope/package");
        assert_eq!(name, "@scope/package");
        assert_eq!(version, None);
    }

    #[test]
    fn test_package_spec_display() {
        let pkg = PackageSpec {
            name: "express".to_string(),
            version: "4.19.2".to_string(),
        };
        assert_eq!(format!("{}", pkg), "express@4.19.2");
    }

    #[tokio::test]
    async fn test_wave_executor_creation() {
        let event_bus = EventBus::new(16);
        let state = StateManager::new("test", "Test", event_bus.clone());
        
        let config = WaveConfig {
            id: "wave1".to_string(),
            name: "Test Wave".to_string(),
            description: String::new(),
            depends_on: Vec::new(),
            mode: WaveMode::Hunt,
            sources: vec![WaveSource::Packages { 
                list: vec!["express".to_string()] 
            }],
            whitelist: Vec::new(),
            expectations: None,
            reporting: None,
        };

        let executor = WaveExecutor::new(config, state, event_bus, 10);
        
        // Verify executor created successfully
        assert_eq!(executor.config.id, "wave1");
    }
}
