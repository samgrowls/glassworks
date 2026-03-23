//! Unicode Scanner Configuration
//!
//! This module provides configuration options for Unicode attack detection.

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use crate::finding::Severity;

/// Unicode scanner configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct UnicodeConfig {
    /// Enable Unicode scanning
    pub enabled: bool,

    /// Sensitivity level: low, medium, high, critical
    pub sensitivity: SensitivityLevel,

    /// Enable/disable individual detectors
    pub detectors: DetectorConfig,

    /// File patterns to include
    pub include_patterns: Vec<String>,

    /// File patterns to exclude
    pub exclude_patterns: Vec<String>,

    /// Allowlist for legitimate i18n usage
    pub allowlist: AllowlistConfig,
}

/// Sensitivity levels for Unicode detection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde", serde(rename_all = "lowercase"))]
pub enum SensitivityLevel {
    /// Report all findings including low-severity issues
    Low,
    /// Report medium and higher severity findings
    Medium,
    /// Report high and critical severity findings (default)
    #[default]
    High,
    /// Report only critical severity findings
    Critical,
}

impl SensitivityLevel {
    /// Get the string representation of the sensitivity level
    pub fn as_str(&self) -> &'static str {
        match self {
            SensitivityLevel::Low => "low",
            SensitivityLevel::Medium => "medium",
            SensitivityLevel::High => "high",
            SensitivityLevel::Critical => "critical",
        }
    }

    /// Parse a sensitivity level from a string (case-insensitive)
    pub fn from_str_val(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "critical" => SensitivityLevel::Critical,
            "high" => SensitivityLevel::High,
            "medium" => SensitivityLevel::Medium,
            _ => SensitivityLevel::Low,
        }
    }
}

/// Configuration for individual detectors
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DetectorConfig {
    /// Detect invisible characters (zero-width, variation selectors)
    pub invisible_chars: bool,

    /// Detect homoglyph/confusable characters
    pub homoglyphs: bool,

    /// Detect bidirectional overrides
    pub bidirectional: bool,

    /// Detect Unicode tags
    pub unicode_tags: bool,

    /// Detect Glassware-specific patterns
    pub glassware: bool,

    /// Detect normalization attacks
    pub normalization: bool,

    /// Detect emoji obfuscation
    pub emoji_obfuscation: bool,
}

impl Default for DetectorConfig {
    fn default() -> Self {
        Self {
            invisible_chars: true,
            homoglyphs: true,
            bidirectional: true,
            unicode_tags: true,
            glassware: true,
            normalization: false,
            emoji_obfuscation: false,
        }
    }
}

/// GlassWare detection configuration
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct GlasswareConfig {
    /// Package whitelist
    #[cfg_attr(feature = "serde", serde(default))]
    pub whitelist: WhitelistConfig,
    /// Scoring configuration
    #[cfg_attr(feature = "serde", serde(default))]
    pub scoring: ScoringConfig,
    /// Detector configuration
    #[cfg_attr(feature = "serde", serde(default))]
    pub detectors: DetectorWeights,
}

/// Package whitelist configuration
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct WhitelistConfig {
    /// Packages to never flag (i18n libraries, etc.)
    #[cfg_attr(feature = "serde", serde(default))]
    pub packages: Vec<String>,
    /// Crypto libraries (blockchain API calls are legitimate)
    #[cfg_attr(feature = "serde", serde(default))]
    pub crypto_packages: Vec<String>,
    /// Build tools (time delays are legitimate)
    #[cfg_attr(feature = "serde", serde(default))]
    pub build_tools: Vec<String>,
    /// State management libraries
    #[cfg_attr(feature = "serde", serde(default))]
    pub state_management: Vec<String>,
}

/// Scoring configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScoringConfig {
    /// Threat score threshold for "malicious" classification
    #[cfg_attr(feature = "serde", serde(default = "default_malicious_threshold"))]
    pub malicious_threshold: f32,
    /// Threat score threshold for "suspicious" classification
    #[cfg_attr(feature = "serde", serde(default = "default_suspicious_threshold"))]
    pub suspicious_threshold: f32,
    /// Weight per attack category present
    #[cfg_attr(feature = "serde", serde(default = "default_category_weight"))]
    pub category_weight: f32,
    /// Weight per critical severity finding
    #[cfg_attr(feature = "serde", serde(default = "default_critical_weight"))]
    pub critical_weight: f32,
    /// Weight per high severity finding
    #[cfg_attr(feature = "serde", serde(default = "default_high_weight"))]
    pub high_weight: f32,
}

impl Default for ScoringConfig {
    fn default() -> Self {
        Self {
            malicious_threshold: default_malicious_threshold(),
            suspicious_threshold: default_suspicious_threshold(),
            category_weight: default_category_weight(),
            critical_weight: default_critical_weight(),
            high_weight: default_high_weight(),
        }
    }
}

fn default_malicious_threshold() -> f32 { 5.0 }  // Lowered from 7.0 for better sensitivity
fn default_suspicious_threshold() -> f32 { 2.0 }  // Lowered from 3.0
fn default_category_weight() -> f32 { 3.0 }  // Increased from 2.0 - more weight per category
fn default_critical_weight() -> f32 { 4.0 }  // Increased from 3.0
fn default_high_weight() -> f32 { 2.0 }  // Increased from 1.5

/// Detector weights configuration
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DetectorWeights {
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub invisible_char: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub homoglyph: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub bidi: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub blockchain_c2: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_heavy_weight"))]
    pub glassware_pattern: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub locale_geofencing: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_weight"))]
    pub time_delay: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_heavy_weight"))]
    pub encrypted_payload: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_heavy_weight"))]
    pub rdd: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_heavy_weight"))]
    pub forcememo: f32,
    #[cfg_attr(feature = "serde", serde(default = "default_heavy_weight"))]
    pub jpd_author: f32,
}

fn default_weight() -> f32 { 1.0 }
fn default_heavy_weight() -> f32 { 3.0 }

/// Allowlist configuration for legitimate i18n usage
#[derive(Debug, Clone, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct AllowlistConfig {
    /// Files to always allow (e.g., i18n resource files)
    pub files: HashSet<String>,

    /// Patterns to allow (e.g., emoji with variation selectors)
    pub patterns: HashSet<String>,

    /// Allow emoji variation selectors
    pub allow_emoji_variants: bool,

    /// Allow CJK character variants
    pub allow_cjk_variants: bool,
}

/// Performance tuning configuration
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct PerformanceConfig {
    /// Maximum file size to scan (bytes)
    pub max_file_size: u64,

    /// Skip binary files
    pub skip_binary: bool,

    /// Parallel scanning enabled
    pub parallel: bool,
}

impl Default for PerformanceConfig {
    /// Create default performance settings (10MB max, skip binary files, parallel enabled)
    fn default() -> Self {
        Self {
            max_file_size: 10 * 1024 * 1024, // 10MB
            skip_binary: true,
            parallel: true,
        }
    }
}

/// Scan configuration for decoupling CLI from engine
///
/// This struct provides a unified configuration interface that can be used
/// by both the CLI and programmatic embeddings of the scan engine.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct ScanConfig {
    /// File extensions to include (e.g., "js", "ts", "py")
    pub extensions: Vec<String>,

    /// Directory/file patterns to exclude (e.g., ".git", "node_modules")
    pub exclude_patterns: Vec<String>,

    /// Maximum file size to scan in bytes
    pub max_file_size: u64,

    /// Enable parallel scanning
    pub enable_parallel: bool,

    /// Number of parallel workers (defaults to num_cpus::get())
    pub parallel_workers: usize,

    /// Enable deduplication of findings
    pub enable_dedup: bool,

    /// Minimum severity level to report
    pub min_severity: Severity,

    /// Enable LLM analysis (requires LLM feature and config)
    #[cfg(feature = "llm")]
    pub enable_llm: bool,
}

impl Default for ScanConfig {
    fn default() -> Self {
        Self {
            extensions: vec![
                "js".into(),
                "mjs".into(),
                "cjs".into(),
                "ts".into(),
                "tsx".into(),
                "jsx".into(),
                "py".into(),
                "rs".into(),
                "go".into(),
                "java".into(),
                "rb".into(),
                "php".into(),
                "sh".into(),
                "bash".into(),
                "zsh".into(),
                "yml".into(),
                "yaml".into(),
                "toml".into(),
                "json".into(),
                "xml".into(),
                "md".into(),
                "txt".into(),
            ],
            exclude_patterns: vec![
                ".git".into(),
                "node_modules".into(),
                "target".into(),
                "__pycache__".into(),
                ".venv".into(),
                "vendor".into(),
            ],
            max_file_size: 5 * 1024 * 1024, // 5MB
            enable_parallel: true,
            parallel_workers: num_cpus::get(),
            enable_dedup: true,
            min_severity: Severity::Low,
            #[cfg(feature = "llm")]
            enable_llm: false,
        }
    }
}

impl ScanConfig {
    /// Create a new ScanConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Builder method to set extensions
    pub fn with_extensions(mut self, extensions: Vec<String>) -> Self {
        self.extensions = extensions;
        self
    }

    /// Builder method to set exclude patterns
    pub fn with_exclude_patterns(mut self, exclude_patterns: Vec<String>) -> Self {
        self.exclude_patterns = exclude_patterns;
        self
    }

    /// Builder method to set max file size
    pub fn with_max_file_size(mut self, max_file_size: u64) -> Self {
        self.max_file_size = max_file_size;
        self
    }

    /// Builder method to enable/disable parallel scanning
    pub fn with_parallel(mut self, enable: bool) -> Self {
        self.enable_parallel = enable;
        self
    }

    /// Builder method to set number of parallel workers
    pub fn with_parallel_workers(mut self, workers: usize) -> Self {
        self.parallel_workers = workers;
        self
    }

    /// Builder method to enable/disable deduplication
    pub fn with_deduplication(mut self, enable: bool) -> Self {
        self.enable_dedup = enable;
        self
    }

    /// Builder method to set minimum severity
    pub fn with_min_severity(mut self, severity: Severity) -> Self {
        self.min_severity = severity;
        self
    }

    /// Builder method to enable/disable LLM analysis
    #[cfg(feature = "llm")]
    pub fn with_llm(mut self, enable: bool) -> Self {
        self.enable_llm = enable;
        self
    }
}

impl Default for UnicodeConfig {
    /// Create default configuration (enabled, high sensitivity, all detectors on)
    fn default() -> Self {
        Self {
            enabled: true,
            sensitivity: SensitivityLevel::default(),
            detectors: DetectorConfig::default(),
            include_patterns: vec![],
            exclude_patterns: vec![".git".to_string(), "node_modules".to_string()],
            allowlist: AllowlistConfig::default(),
        }
    }
}

impl UnicodeConfig {
    /// Create config for i18n projects (more permissive)
    pub fn for_i18n_project() -> Self {
        Self {
            sensitivity: SensitivityLevel::Medium,
            allowlist: AllowlistConfig {
                allow_emoji_variants: true,
                allow_cjk_variants: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }

    /// Create config for high-security projects (stricter)
    pub fn for_high_security() -> Self {
        Self {
            sensitivity: SensitivityLevel::Critical,
            detectors: DetectorConfig {
                normalization: true,
                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_config_default() {
        let config = ScanConfig::default();
        
        // Verify default extensions
        assert!(config.extensions.contains(&"js".to_string()));
        assert!(config.extensions.contains(&"ts".to_string()));
        assert!(config.extensions.contains(&"py".to_string()));
        
        // Verify default exclude patterns
        assert!(config.exclude_patterns.contains(&".git".to_string()));
        assert!(config.exclude_patterns.contains(&"node_modules".to_string()));
        
        // Verify default settings
        assert_eq!(config.max_file_size, 5 * 1024 * 1024);
        assert!(config.enable_parallel);
        assert!(config.enable_dedup);
        assert_eq!(config.min_severity, Severity::Low);
        #[cfg(feature = "llm")]
        assert!(!config.enable_llm);
    }

    #[test]
    fn test_scan_config_builder() {
        let config = ScanConfig::default()
            .with_extensions(vec!["js".to_string(), "ts".to_string()])
            .with_exclude_patterns(vec!["dist".to_string()])
            .with_max_file_size(1024 * 1024)
            .with_parallel(false)
            .with_parallel_workers(4)
            .with_deduplication(false)
            .with_min_severity(Severity::High);
        
        #[cfg(feature = "llm")]
        let config = config.with_llm(true);
        
        assert_eq!(config.extensions, vec!["js".to_string(), "ts".to_string()]);
        assert_eq!(config.exclude_patterns, vec!["dist".to_string()]);
        assert_eq!(config.max_file_size, 1024 * 1024);
        assert!(!config.enable_parallel);
        assert_eq!(config.parallel_workers, 4);
        assert!(!config.enable_dedup);
        assert_eq!(config.min_severity, Severity::High);
        #[cfg(feature = "llm")]
        assert!(config.enable_llm);
    }

    #[test]
    fn test_scan_config_new() {
        let config1 = ScanConfig::new();
        let config2 = ScanConfig::default();
        
        // Both should have the same extensions
        assert_eq!(config1.extensions.len(), config2.extensions.len());
    }
}
