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
        
        let threat_score = self.calculate_threat_score(&findings);
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

    /// Calculate threat score from findings.
    fn calculate_threat_score(&self, findings: &[Finding]) -> f32 {
        if findings.is_empty() {
            return 0.0;
        }

        let mut score = 0.0;

        for finding in findings {
            let severity_score = match finding.severity {
                Severity::Critical => 3.0,
                Severity::High => 2.0,
                Severity::Medium => 1.0,
                Severity::Low => 0.5,
                Severity::Info => 0.1,
            };

            score += severity_score;

            // Bonus for specific detection categories
            match finding.category {
                DetectionCategory::InvisibleCharacter => score += 0.5,
                DetectionCategory::BidirectionalOverride => score += 0.5,
                DetectionCategory::Homoglyph => score += 0.3,
                _ => {}
            }
        }

        // Normalize to 0-10 scale
        (score * 10.0 / (findings.len() as f32 * 3.0)).min(10.0)
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
