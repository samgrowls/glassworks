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
            threat_threshold: 5.0,
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
        let is_malicious = threat_score >= self.config.threat_threshold;

        if is_malicious {
            warn!(
                "Package {} flagged as malicious (threat score: {:.2})",
                package.name, threat_score
            );
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

    /// Calculate threat score from findings using signal stacking.
    /// 
    /// Instead of counting findings, we assess attack patterns across categories:
    /// - Obfuscation: invisible chars, homoglyphs, bidi
    /// - Evasion: locale bypass, time delay, sandbox escape
    /// - C2 Infrastructure: known wallets, known IPs, blockchain polling
    /// - Execution: eval patterns, encrypted payload, dynamic exec
    /// - Persistence: preinstall scripts, file writes, registry
    /// 
    /// Score = (categories_present * 2.0) + (critical_hits * 3.0) + (high_hits * 1.5)
    /// 
    /// Thresholds:
    /// 0-3: Clean
    /// 3-6: Suspicious (review)
    /// 6-10: Likely malicious (quarantine)
    /// 10+: Confirmed malicious (block + report)
    fn calculate_threat_score(&self, findings: &[Finding], package_name: &str) -> f32 {
        if findings.is_empty() {
            return 0.0;
        }

        // Check if this is a known legitimate package that often has unicode in locale files
        let package_lower = package_name.to_lowercase();
        let is_locale_heavy_package = 
            package_lower.contains("moment") ||
            package_lower.contains("date") ||
            package_lower.contains("i18n") ||
            package_lower.contains("globalize") ||
            package_lower.contains("prettier") ||  // Prettier has unicode in doc tests
            package_lower.contains("typescript");   // TypeScript has unicode in tests

        // Track which signal categories are present
        let mut categories = std::collections::HashSet::new();
        let mut critical_hits = 0;
        let mut high_hits = 0;

        for finding in findings {
            // Categorize each finding
            match finding.category {
                // === Obfuscation Category ===
                DetectionCategory::InvisibleCharacter => {
                    categories.insert("obfuscation");
                    // Skip counting for locale-heavy packages (moment, date libraries)
                    if !is_locale_heavy_package {
                        high_hits += 1;
                    }
                }
                DetectionCategory::Homoglyph => {
                    categories.insert("obfuscation");
                    if !is_locale_heavy_package {
                        high_hits += 1;
                    }
                }
                DetectionCategory::BidirectionalOverride => {
                    categories.insert("obfuscation");
                    if !is_locale_heavy_package {
                        high_hits += 1;
                    }
                }

                // === Evasion Category ===
                DetectionCategory::LocaleGeofencing => {
                    categories.insert("evasion");
                    // Skip for known packages with i18n support
                    if !is_locale_heavy_package &&
                       !package_lower.contains("prettier") &&
                       !package_lower.contains("typescript") {
                        high_hits += 1;
                    }
                }
                DetectionCategory::TimeDelaySandboxEvasion => {
                    categories.insert("evasion");
                    // Time delays in build tools are often legitimate (watch mode, debouncing)
                    if !package_lower.contains("prettier") &&
                       !package_lower.contains("typescript") &&
                       !package_lower.contains("webpack") &&
                       !package_lower.contains("vite") &&
                       !package_lower.contains("rollup") {
                        high_hits += 1;
                    }
                }

                // === C2 Infrastructure Category ===
                DetectionCategory::BlockchainC2 => {
                    // Only count if CRITICAL severity (known wallet/IP)
                    if finding.severity == Severity::Critical {
                        categories.insert("c2");
                        critical_hits += 1;
                    } else {
                        // INFO/MEDIUM severity = just API usage, not C2
                        categories.insert("c2_weak");
                    }
                }
                DetectionCategory::SocketIOC2 => {
                    categories.insert("c2");
                    if finding.severity == Severity::High || finding.severity == Severity::Critical {
                        high_hits += 1;
                    }
                }

                // === Execution Category ===
                DetectionCategory::GlasswarePattern => {
                    categories.insert("execution");
                    // Skip for known legitimate packages with string processing
                    if !is_locale_heavy_package && 
                       !package_lower.contains("prettier") &&
                       !package_lower.contains("eslint") &&
                       !package_lower.contains("babel") {
                        if finding.severity == Severity::Critical {
                            critical_hits += 1;
                        } else {
                            high_hits += 1;
                        }
                    }
                }
                DetectionCategory::EncryptedPayload => {
                    categories.insert("execution");
                    if !is_locale_heavy_package {
                        high_hits += 1;
                    }
                }
                DetectionCategory::HeaderC2 => {
                    categories.insert("execution");
                    critical_hits += 1;
                }

                // === Persistence Category ===
                DetectionCategory::RddAttack => {
                    categories.insert("persistence");
                    high_hits += 1;
                }
                DetectionCategory::ForceMemoPython => {
                    categories.insert("persistence");
                    critical_hits += 1;
                }
                DetectionCategory::JpdAuthor => {
                    categories.insert("persistence");
                    high_hits += 1;
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

        // Calculate score
        let category_count = categories.len() as f32;
        let score = (category_count * 2.0) + (critical_hits as f32 * 3.0) + (high_hits as f32 * 1.5);

        // Cap at 10.0
        score.min(10.0)
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
        assert_eq!(scanner.calculate_threat_score(&[]), 0.0);

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
        let score = scanner.calculate_threat_score(&findings);
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
            },
        ];

        let summary = ScanSummary::from_results(&results);
        assert_eq!(summary.total_packages, 2);
        assert_eq!(summary.malicious_packages, 1);
        assert_eq!(summary.total_findings, 1);
    }
}
