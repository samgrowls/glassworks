//! Scanner module integrating glassware-core.
//!
//! This module provides the scanning functionality that uses glassware-core
//! to detect security issues in downloaded packages.

use glassware_core::{
    ScanEngine, Finding,
    DetectionCategory, Severity,
};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Semaphore;
use tracing::{debug, error, info, warn};

use crate::downloader::DownloadedPackage;
use crate::error::{OrchestratorError, Result};

/// LLM verdict for a finding or package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LlmPackageVerdict {
    /// Whether the package is likely malicious.
    pub is_malicious: bool,
    /// Confidence score (0.0-1.0).
    pub confidence: f32,
    /// Explanation from LLM.
    pub explanation: String,
    /// Recommended actions.
    pub recommendations: Vec<String>,
    /// Model used for analysis.
    pub model_used: String,
}

/// Scan result for a single package.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PackageScanResult {
    /// Package name.
    pub package_name: String,
    /// Source type (npm, github, file).
    pub source_type: String,
    /// Version or commit hash.
    pub version: String,
    /// Path to scanned content.
    pub path: String,
    /// Content hash.
    pub content_hash: String,
    /// Scan findings.
    pub findings: Vec<Finding>,
    /// Overall threat score (0.0-10.0).
    pub threat_score: f32,
    /// Whether the package is considered malicious.
    pub is_malicious: bool,
    /// Optional LLM verdict (if LLM analysis was performed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub llm_verdict: Option<LlmPackageVerdict>,
}

/// Configuration for the scanner.
#[derive(Debug, Clone)]
pub struct ScannerConfig {
    /// Maximum concurrent scans.
    pub max_concurrent: usize,
    /// Minimum severity to report.
    pub min_severity: Severity,
    /// Enable semantic analysis (JS/TS only).
    pub enable_semantic: bool,
    /// Enable LLM analysis (requires API key).
    pub enable_llm: bool,
    /// File extensions to scan.
    pub extensions: Vec<String>,
    /// Directories to exclude.
    pub exclude_dirs: Vec<String>,
    /// Threat score threshold for marking as malicious.
    pub threat_threshold: f32,
    /// GlassWorm configuration (scoring, detectors, whitelist, etc.)
    pub glassware_config: glassware_core::GlasswareConfig,
}

impl Default for ScannerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            min_severity: Severity::Low,
            enable_semantic: true,
            enable_llm: false,
            extensions: vec![
                "js".to_string(),
                "mjs".to_string(),
                "cjs".to_string(),
                "ts".to_string(),
                "tsx".to_string(),
                "jsx".to_string(),
                "py".to_string(),
                "rs".to_string(),
                "go".to_string(),
                "rb".to_string(),
                "php".to_string(),
                "java".to_string(),
                "json".to_string(),
                "yaml".to_string(),
                "yml".to_string(),
                "md".to_string(),
                "txt".to_string(),
            ],
            exclude_dirs: vec![
                "node_modules".to_string(),
                ".git".to_string(),
                "dist".to_string(),
                "build".to_string(),
                "target".to_string(),
                "__pycache__".to_string(),
                ".cache".to_string(),
            ],
            threat_threshold: 5.0,  // Updated for better sensitivity
            glassware_config: glassware_core::GlasswareConfig::default(),
        }
    }
}

impl From<crate::config::GlasswareConfig> for ScannerConfig {
    fn from(config: crate::config::GlasswareConfig) -> Self {
        // Convert local config to glassware_core config
        let core_config = glassware_core::GlasswareConfig {
            whitelist: glassware_core::WhitelistConfig {
                packages: config.whitelist.packages,
                crypto_packages: config.whitelist.crypto_packages,
                build_tools: config.whitelist.build_tools,
                state_management: vec![],
            },
            scoring: glassware_core::ScoringConfig {
                malicious_threshold: config.scoring.malicious_threshold,
                suspicious_threshold: config.scoring.suspicious_threshold,
                category_weight: config.scoring.category_weight,
                critical_weight: config.scoring.critical_weight,
                high_weight: config.scoring.high_weight,
            },
            detectors: glassware_core::DetectorWeights {
                invisible_char: config.detectors.invisible_char.weight,
                homoglyph: config.detectors.homoglyph.weight,
                bidi: config.detectors.bidi.weight,
                blockchain_c2: config.detectors.blockchain_c2.weight,
                glassware_pattern: config.detectors.glassware_pattern.weight,
                locale_geofencing: if config.detectors.locale_geofencing.enabled { 1.0 } else { 0.0 },
                time_delay: 1.0,  // Default weight
                encrypted_payload: 1.0,  // Default weight
                rdd: 1.0,  // Default weight
                forcememo: 1.0,  // Default weight
                jpd_author: 1.0,  // Default weight
            },
        };

        Self {
            max_concurrent: config.performance.concurrency,
            min_severity: Severity::Low,
            enable_semantic: true,
            enable_llm: false,  // Can be enabled separately
            extensions: vec![
                "js".to_string(), "mjs".to_string(), "cjs".to_string(),
                "ts".to_string(), "tsx".to_string(), "jsx".to_string(),
                "py".to_string(), "json".to_string(),
            ],
            exclude_dirs: vec![
                "node_modules".to_string(), ".git".to_string(),
                "dist".to_string(), "build".to_string(),
            ],
            threat_threshold: config.scoring.malicious_threshold,
            glassware_config: core_config,
        }
    }
}

/// Scanner for glassware-core integration.
#[derive(Clone)]
pub struct Scanner {
    config: ScannerConfig,
    concurrency_semaphore: Arc<Semaphore>,
}

impl Scanner {
    /// Create a new scanner with default configuration.
    pub fn new() -> Self {
        Self::with_config(ScannerConfig::default())
    }

    /// Create a new scanner with custom configuration.
    pub fn with_config(config: ScannerConfig) -> Self {
        let concurrency_semaphore = Arc::new(Semaphore::new(config.max_concurrent));

        Self {
            config,
            concurrency_semaphore,
        }
    }

    /// Enable LLM analysis.
    pub fn with_llm(mut self) -> Self {
        self.config.enable_llm = true;
        self
    }

    /// Scan a downloaded package.
    pub async fn scan_package(&self, package: &DownloadedPackage) -> Result<PackageScanResult> {
        let _permit = self
            .concurrency_semaphore
            .acquire()
            .await
            .map_err(|_| OrchestratorError::cancelled("Semaphore closed".to_string()))?;

        info!("Scanning package: {} ({})", package.name, package.version);

        let findings = self.scan_directory(&package.path).await?;

        let threat_score = self.calculate_threat_score(&findings, &package.name);
        
        // Check whitelist for final malicious determination (defense in depth)
        let is_whitelisted = self.is_package_whitelisted(&package.name);
        let is_malicious = if is_whitelisted {
            // Whitelisted packages are never flagged as malicious regardless of score
            // This prevents false positives for known legitimate packages (i18n libraries, build tools, etc.)
            false
        } else {
            threat_score >= self.config.threat_threshold
        };

        if is_malicious {
            warn!(
                "Package {} flagged as malicious (threat score: {:.2})",
                package.name, threat_score
            );
        } else if is_whitelisted && !findings.is_empty() {
            info!("Package {} is whitelisted ({} findings suppressed)", package.name, findings.len());
        }

        Ok(PackageScanResult {
            package_name: package.name.clone(),
            source_type: package.source_type.clone(),
            version: package.version.clone(),
            path: package.path.clone(),
            content_hash: package.content_hash.clone(),
            findings,
            threat_score,
            is_malicious,
            llm_verdict: None,  // Will be populated by orchestrator if LLM enabled
        })
    }

    /// Scan a directory for security issues.
    pub async fn scan_directory(&self, path: &str) -> Result<Vec<Finding>> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(OrchestratorError::invalid_path(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }

        let mut all_findings = Vec::new();
        let mut files_scanned = 0;

        // Create scan engine with all detectors
        let engine = ScanEngine::default_detectors();

        // Walk directory recursively
        let mut entries: Vec<PathBuf> = Vec::new();
        self.collect_files(path, &mut entries)?;

        for entry_path in entries {
            if let Some(content) = self.read_file(&entry_path).await? {
                let relative_path = entry_path
                    .strip_prefix(path)
                    .unwrap_or(&entry_path)
                    .to_string_lossy()
                    .to_string();

                debug!("Scanning file: {}", relative_path);

                let findings = engine.scan(path, &content);

                all_findings.extend(findings);
                files_scanned += 1;
            }
        }

        info!(
            "Scanned {} files, found {} issues",
            files_scanned,
            all_findings.len()
        );

        // Sort findings by severity and location
        all_findings.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.file.cmp(&b.file))
                .then_with(|| a.line.cmp(&b.line))
        });

        Ok(all_findings)
    }

    /// Scan a directory for tarballs (don't skip dist/build directories).
    ///
    /// For tarball scans, we want to scan ALL files including compiled output
    /// in /dist/ and /build/ directories since that's what gets distributed.
    pub async fn scan_directory_for_tarball(&self, path: &str) -> Result<Vec<Finding>> {
        let path = Path::new(path);

        if !path.exists() {
            return Err(OrchestratorError::invalid_path(format!(
                "Path does not exist: {}",
                path.display()
            )));
        }

        let mut all_findings = Vec::new();
        let mut files_scanned = 0;

        // Create scan engine with all detectors
        let engine = ScanEngine::default_detectors();

        // Walk directory recursively (don't skip dist/build for tarballs)
        let mut entries: Vec<PathBuf> = Vec::new();
        self.collect_files_for_tarball(path, &mut entries)?;

        for entry_path in entries {
            if let Some(content) = self.read_file(&entry_path).await? {
                let relative_path = entry_path
                    .strip_prefix(path)
                    .unwrap_or(&entry_path)
                    .to_string_lossy()
                    .to_string();

                debug!("Scanning file: {}", relative_path);

                let findings = engine.scan(path, &content);

                all_findings.extend(findings);
                files_scanned += 1;
            }
        }

        info!(
            "Scanned {} files, found {} issues",
            files_scanned,
            all_findings.len()
        );

        // Sort findings by severity and location
        all_findings.sort_by(|a, b| {
            b.severity
                .cmp(&a.severity)
                .then_with(|| a.file.cmp(&b.file))
                .then_with(|| a.line.cmp(&b.line))
        });

        Ok(all_findings)
    }

    /// Scan a tarball file.
    pub async fn scan_tarball(&self, tarball_path: &str) -> Result<PackageScanResult> {
        let tarball_path = Path::new(tarball_path);

        if !tarball_path.exists() {
            return Err(OrchestratorError::invalid_path(format!(
                "Tarball file not found: {}",
                tarball_path.display()
            )));
        }

        info!("Extracting tarball: {}", tarball_path.display());

        // Extract tarball to temp directory
        let temp_dir = tempfile::tempdir().map_err(|e| {
            OrchestratorError::scan_failed(
                tarball_path.to_string_lossy().to_string(),
                format!("Failed to create temp directory: {}", e)
            )
        })?;

        // Open and extract tarball
        let tarball_file = std::fs::File::open(tarball_path).map_err(|e| {
            OrchestratorError::scan_failed(
                tarball_path.to_string_lossy().to_string(),
                format!("Failed to open tarball: {}", e)
            )
        })?;

        // Handle both .tar.gz and .tar files
        let file_size = tarball_file.metadata().map_err(|e| {
            OrchestratorError::scan_failed(
                tarball_path.to_string_lossy().to_string(),
                format!("Failed to get file metadata: {}", e)
            )
        })?.len();
        if file_size == 0 {
            return Err(OrchestratorError::scan_failed(
                tarball_path.to_string_lossy().to_string(),
                "Tarball file is empty".to_string()
            ));
        }

        // Try gzip decompression first, then plain tar
        let result = if tarball_path.extension().map_or(false, |ext| ext == "gz" || ext == "tgz") {
            // Gzip compressed
            let decoder = flate2::read::GzDecoder::new(tarball_file);
            let mut archive = tar::Archive::new(decoder);
            archive.unpack(temp_dir.path())
        } else {
            // Plain tar
            let mut archive = tar::Archive::new(tarball_file);
            archive.unpack(temp_dir.path())
        };

        result.map_err(|e| {
            OrchestratorError::scan_failed(
                tarball_path.to_string_lossy().to_string(),
                format!("Failed to extract tarball: {}", e)
            )
        })?;

        // Find package directory (usually in package/ subdirectory)
        let package_dir = temp_dir.path().join("package");
        let scan_path = if package_dir.exists() {
            package_dir
        } else {
            temp_dir.path().to_path_buf()
        };

        // Extract package info from tarball name or package.json
        let (name, version) = self.extract_package_info(tarball_path, &scan_path)?;

        info!("Scanning package: {} ({})", name, version);

        // Scan the extracted directory with special config for tarballs
        // Don't skip /dist/ or /build/ for tarballs (these ARE the distributed files)
        let findings = self.scan_directory_for_tarball(scan_path.to_str().unwrap()).await?;

        // Calculate threat score
        let threat_score = self.calculate_threat_score(&findings, &name);
        let is_malicious = threat_score >= self.config.threat_threshold;

        if is_malicious {
            warn!(
                "Package {} flagged as malicious (threat score: {:.2})",
                name, threat_score
            );
        }

        Ok(PackageScanResult {
            package_name: name,
            source_type: "tarball".to_string(),
            version,
            path: scan_path.to_string_lossy().to_string(),
            content_hash: format!("tarball:{}", tarball_path.display()),
            findings,
            threat_score,
            is_malicious,
            llm_verdict: None,
        })
    }

    /// Extract package name and version from tarball or package.json.
    fn extract_package_info(&self, tarball_path: &Path, package_dir: &Path) -> Result<(String, String)> {
        // Try to get info from tarball name (e.g., package-1.0.0.tgz)
        let tarball_name = tarball_path.file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("unknown");

        // Remove .tar if present (e.g., package-1.0.0.tar.gz -> package-1.0.0)
        let tarball_name = tarball_name.strip_suffix(".tar").unwrap_or(tarball_name);

        // Parse name and version from tarball name
        if let Some(last_dash) = tarball_name.rfind('-') {
            let name = &tarball_name[..last_dash];
            let version = &tarball_name[last_dash + 1..];
            if !name.is_empty() && !version.is_empty() {
                return Ok((name.to_string(), version.to_string()));
            }
        }

        // Fallback: read package.json
        let package_json_path = package_dir.join("package.json");
        if package_json_path.exists() {
            if let Ok(content) = std::fs::read_to_string(&package_json_path) {
                if let Ok(json) = serde_json::from_str::<serde_json::Value>(&content) {
                    let name = json["name"].as_str().unwrap_or("unknown").to_string();
                    let version = json["version"].as_str().unwrap_or("0.0.0").to_string();
                    return Ok((name, version));
                }
            }
        }

        // Last resort: use tarball name as name, "unknown" as version
        Ok((tarball_name.to_string(), "unknown".to_string()))
    }

    /// Collect files to scan from a directory.
    fn collect_files(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(|e| {
            OrchestratorError::scan_failed(
                dir.to_string_lossy().to_string(),
                format!("Failed to read directory: {}", e)
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                OrchestratorError::scan_failed(
                    dir.to_string_lossy().to_string(),
                    format!("Failed to read entry: {}", e)
                )
            })?;

            let path = entry.path();

            // Skip excluded directories
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    let dir_name_str = dir_name.to_string_lossy().to_string();
                    if self.config.exclude_dirs.contains(&dir_name_str) {
                        debug!("Skipping excluded directory: {}", dir_name_str);
                        continue;
                    }
                }
                self.collect_files(&path, files)?;
            } else if path.is_file() {
                // Check file extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_string();
                    if self.config.extensions.contains(&ext_str) {
                        files.push(path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Collect files to scan from a directory for tarballs (don't skip dist/build).
    fn collect_files_for_tarball(&self, dir: &Path, files: &mut Vec<PathBuf>) -> Result<()> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = std::fs::read_dir(dir).map_err(|e| {
            OrchestratorError::scan_failed(
                dir.to_string_lossy().to_string(),
                format!("Failed to read directory: {}", e)
            )
        })?;

        for entry in entries {
            let entry = entry.map_err(|e| {
                OrchestratorError::scan_failed(
                    dir.to_string_lossy().to_string(),
                    format!("Failed to read entry: {}", e)
                )
            })?;

            let path = entry.path();

            // For tarballs, only skip node_modules and .git (not dist/build)
            if path.is_dir() {
                if let Some(dir_name) = path.file_name() {
                    let dir_name_str = dir_name.to_string_lossy().to_string();
                    // Only skip node_modules and .git for tarballs
                    if dir_name_str == "node_modules" || dir_name_str == ".git" {
                        debug!("Skipping excluded directory: {}", dir_name_str);
                        continue;
                    }
                }
                self.collect_files_for_tarball(&path, files)?;
            } else if path.is_file() {
                // Check file extension
                if let Some(ext) = path.extension() {
                    let ext_str = ext.to_string_lossy().to_string();
                    if self.config.extensions.contains(&ext_str) {
                        files.push(path);
                    }
                }
            }
        }

        Ok(())
    }

    /// Read a file's content.
    async fn read_file(&self, path: &Path) -> Result<Option<String>> {
        match tokio::fs::read_to_string(path).await {
            Ok(content) => Ok(Some(content)),
            Err(e) if e.kind() == std::io::ErrorKind::InvalidData => {
                // Binary file, skip
                debug!("Skipping binary file: {}", path.display());
                Ok(None)
            }
            Err(e) => {
                warn!("Failed to read file {}: {}", path.display(), e);
                Ok(None)
            }
        }
    }

    /// Calculate threat score from findings using signal stacking with config weights.
    ///
    /// Score = (categories × category_weight) + (critical × critical_weight) + (high × high_weight)
    fn calculate_threat_score(&self, findings: &[Finding], package_name: &str) -> f32 {
        if findings.is_empty() {
            return 0.0;
        }

        let config = &self.config.glassware_config;

        // Check if this is a known legitimate package
        let package_lower = package_name.to_lowercase();

        // Check whitelist using precise matching
        let is_whitelisted = self.is_package_whitelisted(package_name);

        let is_crypto_package = config.whitelist.crypto_packages.iter().any(|p| {
            package_lower.contains(&p.to_lowercase())
        });

        let is_build_tool = config.whitelist.build_tools.iter().any(|p| {
            package_lower.contains(&p.to_lowercase())
        });

        // Track which signal categories are present
        let mut categories = std::collections::HashSet::new();
        let mut critical_hits = 0.0;
        let mut high_hits = 0.0;

        for finding in findings {
            // Get detector weight from config (default 1.0 if not specified)
            let detector_weight = self.get_detector_weight(&finding.category);
            
            // Categorize each finding
            match finding.category {
                // === Obfuscation Category ===
                DetectionCategory::InvisibleCharacter => {
                    categories.insert("obfuscation");
                    if !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }
                DetectionCategory::Homoglyph => {
                    categories.insert("obfuscation");
                    if !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }
                DetectionCategory::BidirectionalOverride => {
                    categories.insert("obfuscation");
                    if !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }

                // === Evasion Category ===
                DetectionCategory::LocaleGeofencing => {
                    categories.insert("evasion");
                    // Skip for whitelisted i18n packages
                    if !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }
                DetectionCategory::TimeDelaySandboxEvasion => {
                    categories.insert("evasion");
                    if !is_build_tool && !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }

                // === C2 Infrastructure Category ===
                DetectionCategory::BlockchainC2 => {
                    // Only count if CRITICAL severity (known wallet/IP)
                    if finding.severity == Severity::Critical {
                        categories.insert("c2");
                        critical_hits += detector_weight;
                    } else if !is_crypto_package {
                        // INFO/MEDIUM severity = just API usage, not C2
                        // Skip for crypto packages where API usage is legitimate
                        categories.insert("c2_weak");
                    }
                }
                DetectionCategory::SocketIOC2 => {
                    categories.insert("c2");
                    if finding.severity == Severity::High || finding.severity == Severity::Critical {
                        high_hits += detector_weight;
                    }
                }

                // === Execution Category ===
                DetectionCategory::GlasswarePattern => {
                    categories.insert("execution");
                    if !is_whitelisted {
                        if finding.severity == Severity::Critical {
                            critical_hits += detector_weight;
                        } else {
                            high_hits += detector_weight;
                        }
                    }
                }
                DetectionCategory::EncryptedPayload => {
                    categories.insert("execution");
                    if !is_whitelisted {
                        high_hits += detector_weight;
                    }
                }
                DetectionCategory::HeaderC2 => {
                    categories.insert("execution");
                    critical_hits += detector_weight;
                }

                // === Persistence Category ===
                DetectionCategory::RddAttack => {
                    categories.insert("persistence");
                    high_hits += detector_weight;
                }
                DetectionCategory::ForceMemoPython => {
                    categories.insert("persistence");
                    critical_hits += detector_weight;
                }
                DetectionCategory::JpdAuthor => {
                    categories.insert("persistence");
                    high_hits += detector_weight;
                }

                // === Binary Detectors ===
                DetectionCategory::XorShiftObfuscation => {
                    categories.insert("obfuscation");
                    critical_hits += detector_weight;
                }
                DetectionCategory::IElevatorCom => {
                    categories.insert("c2");
                    critical_hits += detector_weight;
                }
                DetectionCategory::ApcInjection => {
                    categories.insert("execution");
                    critical_hits += detector_weight;
                }
                DetectionCategory::MemexecLoader => {
                    categories.insert("execution");
                    critical_hits += detector_weight;
                }

                // === Unknown/Other ===
                _ => {
                    // Don't categorize unknown findings
                }
            }
        }

        // Remove weak C2 category if strong C2 is present
        if categories.contains("c2") {
            categories.remove("c2_weak");
        }

        // Calculate score using config weights
        let category_count = categories.len() as f32;
        let score = (category_count * config.scoring.category_weight) +
                    (critical_hits * config.scoring.critical_weight) +
                    (high_hits * config.scoring.high_weight);

        // Cap at 10.0
        score.min(10.0)
    }

    /// Get detector weight from config.
    fn get_detector_weight(&self, category: &DetectionCategory) -> f32 {
        let config = &self.config.glassware_config;
        match category {
            DetectionCategory::InvisibleCharacter => config.detectors.invisible_char,
            DetectionCategory::Homoglyph => config.detectors.homoglyph,
            DetectionCategory::BidirectionalOverride => config.detectors.bidi,
            DetectionCategory::BlockchainC2 => config.detectors.blockchain_c2,
            DetectionCategory::GlasswarePattern => config.detectors.glassware_pattern,
            DetectionCategory::LocaleGeofencing => config.detectors.locale_geofencing,
            DetectionCategory::TimeDelaySandboxEvasion => config.detectors.time_delay,
            DetectionCategory::EncryptedPayload => config.detectors.encrypted_payload,
            DetectionCategory::RddAttack => config.detectors.rdd,
            DetectionCategory::ForceMemoPython => config.detectors.forcememo,
            DetectionCategory::JpdAuthor => config.detectors.jpd_author,
            // Default weight for detectors without specific config
            _ => 1.0,
        }
    }

    /// Check if a package is whitelisted (defense in depth).
    ///
    /// This is checked at scoring time to prevent false positives for known legitimate packages.
    /// Matching rules:
    /// - Exact match: "lodash" matches "lodash"
    /// - Prefix with dash: "webpack-" matches "webpack", "webpack-cli"
    /// - Prefix with slash: "@babel/" matches "@babel/core", "@babel/cli"
    /// - Wildcard: "@metamask/*" matches "@metamask/anything"
    fn is_package_whitelisted(&self, package_name: &str) -> bool {
        // Strip version from package name
        // Examples: "webpack@5.89.0" -> "webpack", "@babel/core@7.23.7" -> "@babel/core"
        let package_base = if let Some(at_pos) = package_name.rfind('@') {
            // Only strip if there's a version after @ (not for scoped packages like @babel/core)
            if at_pos > 0 && !package_name.starts_with('@') {
                // Regular package with version: "name@version"
                &package_name[..at_pos]
            } else if package_name.starts_with('@') && at_pos > 1 {
                // Scoped package with version: "@scope/name@version" -> "@scope/name"
                &package_name[..at_pos]
            } else {
                // No version or invalid format
                package_name
            }
        } else {
            package_name
        };
        let package_lower = package_base.to_lowercase();
        let config = &self.config.glassware_config;

        // Helper to check if package matches a whitelist entry
        let matches_entry = |entry: &str| -> bool {
            let entry_lower = entry.to_lowercase();

            // Exact match
            if package_lower == entry_lower {
                return true;
            }

            // Wildcard match (@metamask/* matches @metamask/anything)
            if entry_lower.ends_with("/*") {
                let prefix = &entry_lower[..entry_lower.len()-2]; // "@metamask/"
                return package_lower.starts_with(prefix);
            }

            // Prefix match with dash (webpack- matches webpack-cli)
            if entry_lower.ends_with('-') {
                return package_lower.starts_with(&entry_lower);
            }

            // Prefix match with slash (@babel/ matches @babel/core)
            if entry_lower.ends_with('/') {
                return package_lower.starts_with(&entry_lower);
            }

            false
        };

        // Check all whitelist categories
        config.whitelist.packages.iter().any(|p| matches_entry(p))
            || config.whitelist.crypto_packages.iter().any(|p| matches_entry(p))
            || config.whitelist.build_tools.iter().any(|p| matches_entry(p))
            || config.whitelist.state_management.iter().any(|p| matches_entry(p))
    }

    /// Check if a file is a locale or data file (where invisible chars are legitimate)
    fn is_locale_or_data_file(&self, file_path: &str) -> bool {
        let path_lower = file_path.to_lowercase();
        
        // Check for locale/i18n directories
        path_lower.contains("/locale/") ||
        path_lower.contains("/locales/") ||
        path_lower.contains("/i18n/") ||
        path_lower.contains("/lang/") ||
        path_lower.contains("/languages/") ||
        path_lower.contains("moment") ||  // moment.js has unicode in locale data
        path_lower.contains("date") ||
        path_lower.contains("global") ||  // Global locale files
        path_lower.contains("min/") ||    // Minified files often have unicode
        path_lower.ends_with(".json") ||
        path_lower.ends_with(".min.js")
    }

    /// Scan multiple packages in parallel.
    pub async fn scan_packages(
        &self,
        packages: &[DownloadedPackage],
    ) -> Vec<Result<PackageScanResult>> {
        let mut tasks = Vec::new();

        for package in packages {
            let scanner = self.clone();
            let package = package.clone();
            tasks.push(tokio::spawn(async move {
                scanner.scan_package(&package).await
            }));
        }

        let mut results = Vec::new();
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(OrchestratorError::cancelled(format!(
                    "Task failed: {}",
                    e
                )))),
            }
        }

        results
    }

    /// Get the scanner configuration.
    pub fn config(&self) -> &ScannerConfig {
        &self.config
    }

    /// Scan content string for security issues.
    pub async fn scan_content(&self, content: &str) -> Vec<Finding> {
        let engine = ScanEngine::default_detectors();
        engine.scan(Path::new("<content>"), content)
    }
}

impl Default for Scanner {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of scan results.
#[derive(Debug, Clone, Default)]
pub struct ScanSummary {
    /// Total packages scanned.
    pub total_packages: usize,
    /// Packages flagged as malicious.
    pub malicious_packages: usize,
    /// Total findings.
    pub total_findings: usize,
    /// Findings by severity.
    pub findings_by_severity: std::collections::HashMap<String, usize>,
    /// Findings by category.
    pub findings_by_category: std::collections::HashMap<String, usize>,
    /// Average threat score.
    pub average_threat_score: f32,
}

impl ScanSummary {
    /// Create a summary from multiple package scan results.
    pub fn from_results(results: &[PackageScanResult]) -> Self {
        let mut summary = Self {
            total_packages: results.len(),
            ..Default::default()
        };

        let mut total_threat_score = 0.0;

        for result in results {
            if result.is_malicious {
                summary.malicious_packages += 1;
            }

            total_threat_score += result.threat_score;

            for finding in &result.findings {
                summary.total_findings += 1;
                *summary
                    .findings_by_severity
                    .entry(format!("{:?}", finding.severity))
                    .or_insert(0) += 1;
                *summary
                    .findings_by_category
                    .entry(format!("{:?}", finding.category))
                    .or_insert(0) += 1;
            }
        }

        summary.average_threat_score = if results.is_empty() {
            0.0
        } else {
            total_threat_score / results.len() as f32
        };

        summary
    }
}

impl std::fmt::Display for ScanSummary {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Scan Summary:")?;
        writeln!(f, "  Total packages: {}", self.total_packages)?;
        writeln!(f, "  Malicious packages: {}", self.malicious_packages)?;
        writeln!(f, "  Total findings: {}", self.total_findings)?;
        writeln!(f, "  Average threat score: {:.2}", self.average_threat_score)?;
        writeln!(f, "  Findings by severity:")?;
        for (severity, count) in &self.findings_by_severity {
            writeln!(f, "    {:?}: {}", severity, count)?;
        }
        writeln!(f, "  Findings by category:")?;
        for (category, count) in &self.findings_by_category {
            writeln!(f, "    {:?}: {}", category, count)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scanner_config_default() {
        let config = ScannerConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.min_severity, Severity::Low);
        assert!(config.enable_semantic);
        assert!(!config.enable_llm);
        assert!(!config.extensions.is_empty());
        assert!(!config.exclude_dirs.is_empty());
    }

    #[test]
    fn test_threat_score_calculation() {
        let scanner = Scanner::new();

        // Empty findings
        assert_eq!(scanner.calculate_threat_score(&[], "test-pkg"), 0.0);

        // Single critical finding
        let findings = vec![Finding {
            file: "test.js".to_string(),
            line: 1,
            column: 1,
            code_point: 0xFE00,
            character: "\u{FE00}".to_string(),
            raw_bytes: None,
            severity: Severity::Critical,
            category: DetectionCategory::InvisibleCharacter,
            description: "Test".to_string(),
            remediation: "Remove invisible character".to_string(),
            cwe_id: None,
            references: vec![],
            context: None,
            decoded_payload: None,
            confidence: None,
        }];
        let score = scanner.calculate_threat_score(&findings, "test-pkg");
        assert!(score > 0.0);
    }

    #[test]
    fn test_scan_summary() {
        let results = vec![
            PackageScanResult {
                package_name: "pkg1".to_string(),
                source_type: "npm".to_string(),
                version: "1.0.0".to_string(),
                path: "/path1".to_string(),
                content_hash: "hash1".to_string(),
                findings: vec![],
                threat_score: 0.0,
                is_malicious: false,
                llm_verdict: None,
            },
            PackageScanResult {
                package_name: "pkg2".to_string(),
                source_type: "npm".to_string(),
                version: "1.0.0".to_string(),
                path: "/path2".to_string(),
                content_hash: "hash2".to_string(),
                findings: vec![Finding {
                    file: "test.js".to_string(),
                    line: 1,
                    column: 1,
                    code_point: 0xFE00,
                    character: "\u{FE00}".to_string(),
                    raw_bytes: None,
                    severity: Severity::High,
                    category: DetectionCategory::InvisibleCharacter,
                    description: "Test".to_string(),
                    remediation: "Remove invisible character".to_string(),
                    cwe_id: None,
                    references: vec![],
                    context: None,
                    decoded_payload: None,
                    confidence: None,
                }],
                threat_score: 5.0,
                is_malicious: true,
                llm_verdict: None,
            },
        ];

        let summary = ScanSummary::from_results(&results);
        assert_eq!(summary.total_packages, 2);
        assert_eq!(summary.malicious_packages, 1);
        assert_eq!(summary.total_findings, 1);
    }
}
